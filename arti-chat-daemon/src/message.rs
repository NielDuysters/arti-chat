//! Types representing ChatMessages send between peers.

/// Payload of message.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MessagePayload {
    /// Onion HsId of the sender.
    pub onion_id: String,

    /// Text content of message.
    pub text: String,
}

