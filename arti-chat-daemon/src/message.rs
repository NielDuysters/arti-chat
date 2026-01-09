//! Logic for different types of messages (text, image,...)

/// Content type of message.
#[non_exhaustive]
#[derive(serde::Deserialize, serde::Serialize)]
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
    }
}
