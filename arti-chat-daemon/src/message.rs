//! Logic for different types of messages (text, image,...)

/// Content type of message.
#[non_exhaustive]
#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(tag = "type", content = "content")]
pub enum MessageContent {
    /// Plain text message.
    Text {
        /// Content of the text.
        text: String,
    },
    /// Image.
    Image {
        /// Image bytes.
        data: Vec<u8>,
    },
    /// Display error in chat.
    Error {
        /// Error message.
        message: String,
    },
}
