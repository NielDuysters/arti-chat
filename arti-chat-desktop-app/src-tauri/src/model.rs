use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Contact {
    pub onion_id: String,
    pub nickname: String,
    pub public_key: String,
    pub last_message_at: i32,
    pub last_viewed_at: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    /// Column sender_onion_id.
    pub sender_onion_id: String,

    /// Column body.
    pub body: String,

    /// Column timestamp.
    pub timestamp: i32,

    /// Column is_incoming.
    pub is_incoming: bool,
    
    /// Column sent_status.
    pub sent_status: bool,

    /// Column verified_status.
    pub verified_status: bool,
}

