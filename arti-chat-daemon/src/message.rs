//! Types representing ChatMessages send between peers.

/// Payload of message.
#[derive(serde::Serialize)]
pub struct MessagePayload {
    /// Onion HsId of the sender.
    pub sender_onion_id: String,

    /// Text content of message.
    pub text: String,
}

