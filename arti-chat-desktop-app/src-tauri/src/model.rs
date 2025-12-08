use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Contact {
    pub onion_id: String,
    pub nickname: String,
    pub public_key: String,
    pub last_message_at: i32,
    pub last_viewed_at: i32,
}

