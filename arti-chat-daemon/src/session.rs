//! Minimal authenticated session with forward secrecy (single ratchet).
//!
//! Simplified version:
//! - No counters
//! - No out-of-order handling
//! - No skipped-key cache
//!
//! Assumption: messages are processed in-order and never resent.

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use hkdf::Hkdf;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

use crate::error::ClientError;

//
// ===== Session state =====
//

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub send_chain: [u8; 32],
    pub recv_chain: [u8; 32],
}

impl Drop for Session {
    fn drop(&mut self) {
        self.send_chain.zeroize();
        self.recv_chain.zeroize();
    }
}

//
// ===== Handshake =====
//

#[derive(Serialize, Deserialize)]
pub struct Handshake {
    pub from: String,
    pub to: String,
    pub eph_pub: [u8; 32],
    pub sig: String,
}

/// Domain-separated transcript for signing the ephemeral X25519 pubkey.
/// (Keep this stable and versioned.)
fn transcript(from: &str, to: &str, eph_pub: &[u8; 32]) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(b"arti-chat/handshake/v1");
    v.push(0);
    v.extend_from_slice(from.as_bytes());
    v.push(0);
    v.extend_from_slice(to.as_bytes());
    v.push(0);
    v.extend_from_slice(eph_pub);
    v
}

/// Initiator: create Handshake + return ephemeral secret.
pub fn initiate_handshake(
    my_onion: &str,
    peer_onion: &str,
    id_key: &SigningKey,
) -> (Handshake, StaticSecret) {
    let eph = StaticSecret::new(OsRng);
    let eph_pub = PublicKey::from(&eph);

    let t = transcript(my_onion, peer_onion, &eph_pub.to_bytes());
    let sig = id_key.sign(&t).to_string();

    (
        Handshake {
            from: my_onion.into(),
            to: peer_onion.into(),
            eph_pub: eph_pub.to_bytes(),
            sig,
        },
        eph,
    )
}

/// Responder: verify initiator handshake + create reply handshake.
/// Returns (reply_handshake, responder_ephemeral_secret).
pub fn accept_handshake(
    h: &Handshake,
    my_onion: &str,
    peer_verify: &VerifyingKey,
    id_key: &SigningKey,
) -> Result<(Handshake, StaticSecret), ClientError> {
    if h.to != my_onion {
        return Err(ClientError::ArtiBug);
    }

    let t = transcript(&h.from, &h.to, &h.eph_pub);
    let sig: ed25519_dalek::Signature = h.sig.parse()?;
    peer_verify.verify_strict(&t, &sig)?;

    let eph = StaticSecret::new(OsRng);
    let eph_pub = PublicKey::from(&eph);

    let t_reply = transcript(my_onion, &h.from, &eph_pub.to_bytes());
    let sig_reply = id_key.sign(&t_reply).to_string();

    Ok((
        Handshake {
            from: my_onion.into(),
            to: h.from.clone(),
            eph_pub: eph_pub.to_bytes(),
            sig: sig_reply,
        },
        eph,
    ))
}

/// Finalize handshake and derive the session (send_chain, recv_chain).
///
/// IMPORTANT:
/// - Initiator uses (send=a, recv=b)
/// - Responder uses (send=b, recv=a)
pub fn complete_handshake(
    reply: &Handshake,
    my_onion: &str,
    peer_verify: &VerifyingKey,
    my_eph: StaticSecret,
    initiator: bool,
) -> Result<Session, ClientError> {
    if reply.to != my_onion {
        return Err(ClientError::ArtiBug);
    }

    // Verify reply signature
    let t = transcript(&reply.from, &reply.to, &reply.eph_pub);
    let sig: ed25519_dalek::Signature = reply.sig.parse()?;
    peer_verify.verify_strict(&t, &sig)?;

    // Compute shared secret
    let peer_pub = PublicKey::from(reply.eph_pub);
    let shared = my_eph.diffie_hellman(&peer_pub);

    // Derive two chain keys
    let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    hk.expand(b"arti-chat/chain-a/v1", &mut a)
        .map_err(|_| ClientError::ArtiBug)?;
    hk.expand(b"arti-chat/chain-b/v1", &mut b)
        .map_err(|_| ClientError::ArtiBug)?;

    let (send, recv) = if initiator { (a, b) } else { (b, a) };

    Ok(Session {
        send_chain: send,
        recv_chain: recv,
    })
}

//
// ===== Ratchet + encryption =====
//

#[derive(Serialize, Deserialize)]
pub struct Encrypted {
    pub from_onion_id: String,
    pub nonce: [u8; 12],
    pub data: Vec<u8>,
}

/// One ratchet step:
/// - input: chain_key
/// - output: (msg_key, next_chain_key)
fn ratchet_step(chain: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    // Use chain key as PRK; derive two outputs with different labels.
    let hk = Hkdf::<Sha256>::from_prk(chain).expect("valid PRK");

    let mut msg = [0u8; 32];
    let mut next = [0u8; 32];
    hk.expand(b"arti-chat/msg-key/v1", &mut msg).unwrap();
    hk.expand(b"arti-chat/next-chain/v1", &mut next).unwrap();

    (msg, next)
}

/// Encrypt plaintext and advance send ratchet.
pub fn encrypt(session: &mut Session, plaintext: &[u8], from_onion_id: String) -> Encrypted {
    let (msg_key, next) = ratchet_step(&session.send_chain);
    session.send_chain = next;

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&msg_key));
    let nonce = rand::random::<[u8; 12]>();
    let data = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .expect("encryption failed");

    Encrypted {
        from_onion_id,
        nonce,
        data,
    }
}

/// Decrypt ciphertext and advance receive ratchet.
///
/// NOTE: This requires in-order delivery. If messages are lost/reordered,
/// decryption will fail and the session will be out of sync.
pub fn decrypt(session: &mut Session, msg: &Encrypted) -> Result<Vec<u8>, ClientError> {
    let (msg_key, next) = ratchet_step(&session.recv_chain);
    session.recv_chain = next;

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&msg_key));
    match cipher
        .decrypt(Nonce::from_slice(&msg.nonce), msg.data.as_ref()) {
        Ok(c) => return Ok(c),
        Err(e) => {
            println!("Error decrypting: {:?}", e);
            return Err(ClientError::ArtiBug);
        }
    }
}

//
// ===== Utilities =====
//

pub fn verifying_key_from_hex(hex_pk: &str) -> Result<VerifyingKey, ClientError> {
    let bytes = hex::decode(hex_pk)?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| ClientError::InvalidKeyLength)?;
    Ok(VerifyingKey::from_bytes(&arr)?)
}

/// Read null-terminated frame from Tor stream.
pub async fn read_null_terminated<S: tokio::io::AsyncRead + Unpin>(
    stream: &mut S,
) -> Result<String, ClientError> {
    use tokio::io::AsyncReadExt;

    let mut buf = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        match stream.read(&mut byte).await {
            Ok(0) => break, // EOF
            Ok(1) => {
                if byte[0] == 0 {
                    break;
                }
                buf.push(byte[0]);
            }
            Ok(_) => unreachable!(),
            Err(e) => return Err(e.into()),
        }
    }

    Ok(String::from_utf8_lossy(&buf).to_string())
}

