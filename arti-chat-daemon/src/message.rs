//! Types representing ChatMessages send between peers.

use ed25519_dalek::ed25519::signature::SignerMut;

use crate::error::MessageError;

/// Payload of message.
#[non_exhaustive]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct MessagePayload {
    /// Onion HsId of the sender.
    pub onion_id: String,

    /// Text content of message.
    pub text: String,

    /// Timestamp.
    pub timestamp: i64,
}

/// Signed message payload.
#[non_exhaustive]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SignedMessagePayload {
    /// Message payload.
    pub payload: MessagePayload,

    /// Signature.
    pub signature: String,
}

impl MessagePayload {
    /// Sign message.
    pub fn sign_message(
        &self,
        private_key: &mut ed25519_dalek::SigningKey,
    ) -> Result<SignedMessagePayload, MessageError> {
        let json_payload = serde_json::to_string(&self)?;
        let signature = private_key.sign(json_payload.as_bytes()).to_string();

        Ok(SignedMessagePayload {
            payload: self.clone(),
            signature,
        })
    }
}

impl SignedMessagePayload {
    /// Verify message.
    pub fn verify_message(&self, public_key_str: &str) -> Result<bool, MessageError> {
        // Get public key.
        let public_key = hex::decode(public_key_str)?;
        if public_key.len() != ed25519_dalek::PUBLIC_KEY_LENGTH {
            return Err(MessageError::InvalidKeyLength);
        }
        let mut public_key_array = [0_u8; ed25519_dalek::PUBLIC_KEY_LENGTH];
        public_key_array.copy_from_slice(&public_key);
        let public_key = ed25519_dalek::VerifyingKey::from_bytes(&public_key_array)?;

        // Get signature + payload.
        let signature: ed25519_dalek::Signature = self.signature.parse()?;
        let payload = serde_json::to_vec(&self.payload)?;

        // Verify signature.
        Ok(public_key.verify_strict(&payload, &signature).is_ok())
    }
}
