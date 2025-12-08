use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Contact {
    pub onion: String,
    pub nickname: String,
    pub unread_messages: i32,
}

