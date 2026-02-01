//! Logic to encrypt a chat session with a simple single ratchet algorithm.
//! This to provide forward secrecy.

use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use tokio::io::AsyncReadExt;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

use crate::error::RatchetError;
use crate::message::MessageContent;

/// Send and receive chain.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct RatchetChain {
    /// To encrypt sending messages.
    pub send_chain: [u8; 32],
    /// To decrypt receiving messages.
    pub recv_chain: [u8; 32],
}

impl RatchetChain {
    /// Next step in ratchet.
    fn next_step(chain: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
        let hk = hkdf::Hkdf::<sha2::Sha256>::from_prk(chain).expect("valid PRK");

        let mut msg = [0u8; 32];
        let mut next = [0u8; 32];
        hk.expand(&[], &mut msg).unwrap();
        hk.expand(&[], &mut next).unwrap();

        (msg, next)
    }

    /// Encrypt plaintext data + do next step in send chain.
    pub fn encrypt(&mut self, plaintext: &[u8], self_onion_id: String) -> EncryptedMessage {
        let (current_key, next_key) = Self::next_step(&self.send_chain);
        self.send_chain = next_key;

        let cipher = ChaCha20Poly1305::new(Key::from_slice(&current_key));
        let nonce = rand::random::<[u8; 12]>();
        let data = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .expect("encryption failed");

        EncryptedMessage {
            from: self_onion_id,
            nonce,
            data,
        }
    }

    /// Decrypt encrypted message + do next step in receive chain.
    pub fn decrypt(&mut self, msg: &EncryptedMessage) -> Result<Vec<u8>, RatchetError> {
        let (current_key, next_key) = Self::next_step(&self.recv_chain);
        self.recv_chain = next_key;

        let ciphertext = ChaCha20Poly1305::new(Key::from_slice(&current_key));
        ciphertext
            .decrypt(Nonce::from_slice(&msg.nonce), msg.data.as_ref())
            .map_err(|_| RatchetError::MessageDecryptError)
    }
}

/// Securely zero memory.
impl Drop for RatchetChain {
    fn drop(&mut self) {
        self.send_chain.zeroize();
        self.recv_chain.zeroize();
    }
}

/// Handshake to establish or accept a session.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Handshake {
    /// Origin of handshake.
    pub from: String,
    /// Receiver of handshake.
    pub to: String,
    /// Ephemeral public key to establish session.
    pub ephemeral_pub_key: [u8; 32],
    /// Signature to verify handshake.
    pub signature: String,
}

impl Handshake {
    /// Create transcipt so we can sign public key for handshake.
    fn transcript(from: &str, to: &str, ephemeral_pub_key: &[u8; 32]) -> Vec<u8> {
        let mut t = Vec::new();
        t.extend_from_slice(from.as_bytes());
        t.push(0);
        t.extend_from_slice(to.as_bytes());
        t.push(0);
        t.extend_from_slice(ephemeral_pub_key);
        t
    }

    /// Create handshake and return ephemeral secret.
    /// Send by initiator.
    pub fn initiate(
        self_onion_id: &str,
        peer_onion_id: &str,
        self_private_key: &SigningKey,
    ) -> (Self, StaticSecret) {
        let ephemeral_priv_key = StaticSecret::random_from_rng(rand_core::OsRng);
        let ephemeral_pub_key = PublicKey::from(&ephemeral_priv_key);
        let t = Self::transcript(self_onion_id, peer_onion_id, ephemeral_pub_key.as_bytes());
        let signature = self_private_key.sign(&t).to_string();

        (
            Self {
                from: self_onion_id.into(),
                to: peer_onion_id.into(),
                ephemeral_pub_key: ephemeral_pub_key.to_bytes(),
                signature,
            },
            ephemeral_priv_key,
        )
    }

    /// Accept incoming handshake + create response handshake and return ephemeral secret.
    /// Done by responder.
    pub fn accept(
        &self,
        self_onion_id: &str,
        peer_public_key: &VerifyingKey,
        self_private_key: &SigningKey,
    ) -> Result<(Self, StaticSecret), RatchetError> {
        if self.to != self_onion_id {
            return Err(RatchetError::InvalidHandshakeTarget);
        }

        // Verify incoming handshake.
        let t = Self::transcript(&self.from, &self.to, &self.ephemeral_pub_key);
        let signature: ed25519_dalek::Signature = self.signature.parse()?;
        peer_public_key.verify_strict(&t, &signature)?;

        // Create reply.
        let ephemeral_priv_key = StaticSecret::random_from_rng(rand_core::OsRng);
        let ephemeral_pub_key = PublicKey::from(&ephemeral_priv_key);
        let t_reply = Self::transcript(self_onion_id, &self.from, ephemeral_pub_key.as_bytes());
        let signature_reply = self_private_key.sign(&t_reply).to_string();

        Ok((
            Self {
                from: self_onion_id.into(),
                to: self.from.to_string(),
                ephemeral_pub_key: ephemeral_pub_key.to_bytes(),
                signature: signature_reply,
            },
            ephemeral_priv_key,
        ))
    }

    /// Complete handshake and derive ratchet chains.
    pub fn complete(
        &self,
        self_onion_id: &str,
        peer_public_key: &VerifyingKey,
        self_ephemeral_priv_key: StaticSecret,
        is_initiator: bool,
    ) -> Result<RatchetChain, RatchetError> {
        if self.to != self_onion_id {
            return Err(RatchetError::InvalidHandshakeTarget);
        }

        // Verify reply.
        let t = Self::transcript(&self.from, &self.to, &self.ephemeral_pub_key);
        let signature: ed25519_dalek::Signature = self.signature.parse()?;
        peer_public_key.verify_strict(&t, &signature)?;

        // Shared DH secret.
        let shared_secret =
            self_ephemeral_priv_key.diffie_hellman(&PublicKey::from(self.ephemeral_pub_key));

        // Derive key chains for sending and receiving.
        let hk = hkdf::Hkdf::<sha2::Sha256>::new(None, shared_secret.as_bytes());
        let mut a = [0_u8; 32];
        let mut b = [0_u8; 32];
        hk.expand(&[], &mut a)
            .map_err(|_| RatchetError::HkdfInvalidLength)?;
        hk.expand(&[], &mut b)
            .map_err(|_| RatchetError::HkdfInvalidLength)?;

        let (send_chain, recv_chain) = if is_initiator { (a, b) } else { (b, a) };
        Ok(RatchetChain {
            send_chain,
            recv_chain,
        })
    }
}

/// Encrypted ciphertext.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct EncryptedMessage {
    /// Origin of message.
    pub from: String,
    /// Nonce for unique keystream.
    pub nonce: [u8; 12],
    /// Message data / payload.
    pub data: Vec<u8>,
}

/// Unencrypted message.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PlaintextPayload {
    /// Sender or receiver.
    pub onion_id: String,
    /// Timestamp from sending or receiving.
    pub timestamp: i64,
    /// Content of the message (can be of different types).
    pub message: MessageContent,
}

// --- Helpers ---

/// Get VerifyingKey from hex string.
pub fn verifying_key_from_hex(hex_pk: &str) -> Result<VerifyingKey, RatchetError> {
    let bytes = hex::decode(hex_pk)?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| RatchetError::InvalidKeyLength)?;
    Ok(VerifyingKey::from_bytes(&arr)?)
}

/// Read null-terminated frame from Tor stream.
pub async fn read_null_terminated<S: tokio::io::AsyncRead + Unpin>(
    stream: &mut S,
) -> Result<String, RatchetError> {
    let mut buffer = Vec::new();
    let mut byte = [0_u8; 1];

    loop {
        match stream.read(&mut byte).await {
            Ok(0) => break, // EOF
            Ok(1) => {
                if byte[0] == 0 {
                    break;
                }
                buffer.push(byte[0]);
            }
            Ok(_) => unreachable!(),
            Err(e) => return Err(e.into()),
        }
    }

    Ok(String::from_utf8_lossy(&buffer).to_string())
}
