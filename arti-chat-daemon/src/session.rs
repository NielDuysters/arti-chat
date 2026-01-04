//! Minimal authenticated session with forward secrecy (single ratchet),
//! now with out-of-order message support via a skipped-message-key cache.

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::{SigningKey, VerifyingKey, Signer};
use hkdf::Hkdf;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

use std::collections::BTreeMap;

use crate::error::ClientError;

const MAX_SKIP: u32 = 2000; // cap to limit DoS/memory growth; tune as you like

//
// ===== Session state =====
//

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub send_chain: [u8; 32],
    pub recv_chain: [u8; 32],
    pub send_count: u32,
    pub recv_count: u32,

    /// Cached message keys for skipped counters (out-of-order support).
    /// Keyed by message counter.
    pub skipped: BTreeMap<u32, [u8; 32]>,
}

impl Drop for Session {
    fn drop(&mut self) {
        self.send_chain.zeroize();
        self.recv_chain.zeroize();
        for (_k, v) in self.skipped.iter_mut() {
            v.zeroize();
        }
        self.skipped.clear();
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

/// Build transcript for handshake signature (domain separated & versioned).
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

/// Initiator: create handshake and ephemeral secret.
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

/// Responder: verify handshake and reply.
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

/// Finalize handshake and derive session keys.
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

    let t = transcript(&reply.from, &reply.to, &reply.eph_pub);
    let sig: ed25519_dalek::Signature = reply.sig.parse()?;
    peer_verify.verify_strict(&t, &sig)?;

    let peer_pub = PublicKey::from(reply.eph_pub);
    let shared = my_eph.diffie_hellman(&peer_pub);

    let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());

    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    hk.expand(b"arti-chat/chain-a", &mut a).map_err(|_| ClientError::ArtiBug)?;
    hk.expand(b"arti-chat/chain-b", &mut b).map_err(|_| ClientError::ArtiBug)?;

    let (send, recv) = if initiator { (a, b) } else { (b, a) };

    Ok(Session {
        send_chain: send,
        recv_chain: recv,
        send_count: 0,
        recv_count: 0,
        skipped: BTreeMap::new(),
    })
}

//
// ===== Ratchet + encryption =====
//

#[derive(Serialize, Deserialize)]
pub struct Encrypted {
    pub from_onion_id: String, // keep in your wire format for lookup
    pub counter: u32,
    pub nonce: [u8; 12],
    pub data: Vec<u8>,
}

/// One symmetric-ratchet step: derive message key and next chain key.
fn ratchet_step(chain: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let hk = Hkdf::<Sha256>::from_prk(chain).expect("valid PRK");

    let mut msg = [0u8; 32];
    let mut next = [0u8; 32];

    hk.expand(b"msg", &mut msg).unwrap();
    hk.expand(b"next", &mut next).unwrap();

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

    let msg = Encrypted {
        from_onion_id,
        counter: session.send_count,
        nonce,
        data,
    };

    session.send_count = session.send_count.wrapping_add(1);
    msg
}

/// Decrypt ciphertext and advance receive ratchet.
/// Supports out-of-order delivery via skipped-key cache.
pub fn decrypt(session: &mut Session, msg: &Encrypted) -> Result<Vec<u8>, ClientError> {
    // Case 1: message is from the past — only OK if we cached its key
    if msg.counter < session.recv_count {
        if let Some(msg_key) = session.skipped.remove(&msg.counter) {
            let cipher = ChaCha20Poly1305::new(Key::from_slice(&msg_key));
            let pt = cipher.decrypt(Nonce::from_slice(&msg.nonce), msg.data.as_ref()).map_err(|_| ClientError::ArtiBug)?;
            return Ok(pt);
        }
        return Err(ClientError::ArtiBug);
    }

    // Case 2: message is too far in the future — reject (DoS protection)
    let gap = msg.counter.saturating_sub(session.recv_count);
    if gap > MAX_SKIP {
        return Err(ClientError::ArtiBug);
    }

    // Case 3: message is in the future — derive and cache skipped keys
    while session.recv_count < msg.counter {
        let (mk, next) = ratchet_step(&session.recv_chain);
        session.recv_chain = next;

        // Cache key for this skipped counter
        session.skipped.insert(session.recv_count, mk);

        session.recv_count = session.recv_count.wrapping_add(1);

        // Optional: enforce cache size (BTreeMap makes eviction easy)
        while session.skipped.len() as u32 > MAX_SKIP {
            // remove smallest counter
            if let Some((&k, _)) = session.skipped.iter().next() {
                if let Some(mut v) = session.skipped.remove(&k) {
                    v.zeroize();
                }
            } else {
                break;
            }
        }
    }

    // Now msg.counter == session.recv_count → decrypt normally
    let (msg_key, next) = ratchet_step(&session.recv_chain);
    session.recv_chain = next;

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&msg_key));
    let plaintext = cipher.decrypt(Nonce::from_slice(&msg.nonce), msg.data.as_ref()).map_err(|_| ClientError::ArtiBug)?;

    session.recv_count = session.recv_count.wrapping_add(1);
    Ok(plaintext)
}

//
// ===== Utilities =====
//

pub fn verifying_key_from_hex(hex_pk: &str) -> Result<VerifyingKey, ClientError> {
    let bytes = hex::decode(hex_pk)?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| ClientError::InvalidKeyLength)?;
    Ok(VerifyingKey::from_bytes(&arr)?)
}

/// Read null-terminated frame from Tor stream.
pub async fn read_null_terminated<S: tokio::io::AsyncRead + Unpin>(
    stream: &mut S,
) -> Result<String, ClientError> {
    use tokio::io::AsyncReadExt;

    let mut buf = Vec::new();
    let mut byte = [0u8; 1];

    while stream.read_exact(&mut byte).await.is_ok() {
        if byte[0] == 0 {
            break;
        }
        buf.push(byte[0]);
    }

    Ok(String::from_utf8_lossy(&buf).to_string())
}

