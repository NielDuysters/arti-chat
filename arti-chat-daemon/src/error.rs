//! Used error types.

use image::ImageError;
use thiserror::Error;

use crate::ipc::MessageToUI;

/// Errors related to daemon.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum DaemonError {
    /// Error running daemon.
    #[error("Error running daemon.")]
    RunError(String),

    /// Error in client.
    #[error("Arti Chat Client error: {0}")]
    ClientError(#[from] ClientError),

    /// I/O Error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Database Error.
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
}

/// Errors related to client.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ClientError {
    /// Error in Arti TorClient.
    #[error("Tor Client error: {0}")]
    TorClientError(#[from] arti_client::Error),

    /// Specified onion service is disabled in config.
    #[error("Specified onion service is disabled in config")]
    OnionServiceDisabled,

    /// Invalid nickname for onion service.
    #[error("Invalid nickname for onion service: {0}")]
    OnionServiceInvalidNickname(#[from] tor_persist::hsnickname::InvalidNickname),

    /// Failed to build Arti config.
    #[error("Failed to build Arti config: {0}")]
    ArtiConfigBuildError(#[from] arti_client::config::ConfigBuildError),

    /// Empty HsId.
    #[error("Empty Hsid.")]
    EmptyHsid,

    /// Database Error.
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    /// Hex decode error.
    #[error("Hex decode error: {0}")]
    HexDecodeError(#[from] hex::FromHexError),

    /// Invalid key length.
    #[error("Key length is not 32 bytes.")]
    InvalidKeyLength,

    /// Ed25519 error.
    #[error("ed25519 error: {0}")]
    Ed25519Error(#[from] ed25519_dalek::ed25519::Error),

    /// Internal Arti bug.
    #[error("Internal Arti bug")]
    ArtiBug,

    /// HiddenServiceClientError.
    #[error("Arti HiddenServiceClientError: {0}")]
    ArtiHiddenServiceClientError(#[from] tor_hsservice::ClientError),

    /// Serde Json Error.
    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    /// I/O Error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error related to message.
    #[error("Message error: {0}")]
    MessageError(#[from] MessageError),

    /// Invalid config key.
    #[error("Invalid config key.")]
    InvalidConfigKey,
    
    /// Ratchet Error.
    #[error("Ratchet error: {0}")]
    RatchetError(#[from] RatchetError),
    
    /// Attachment Error.
    #[error("Attachment error: {0}")]
    AttachmentError(#[from] AttachmentError),
}

/// Errors related to database.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// I/O Error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Rusqlite error.
    #[error("rusqlite error: {0}")]
    RusqliteError(#[from] rusqlite::Error),

    /// Invalid primary key type.
    #[error("Invalid primary key type.")]
    InvalidPrimaryKeyType,

    /// Error with OS keyring.
    #[error("Keyring error: {0}")]
    KeyringError(#[from] keyring::Error),
}

/// Errors related to IPC server.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum IpcError {
    /// I/O Error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Rusqlite error.
    #[error("rusqlite error: {0}")]
    RusqliteError(#[from] rusqlite::Error),
}

/// Errors related to RPC.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum RpcError {
    /// I/O Error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serde Json Error.
    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    /// Mpsc send error.
    #[error("mpsc send error: {0}")]
    MpscSendError(#[from] tokio::sync::mpsc::error::SendError<MessageToUI>),

    /// Database error.
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    /// Error in client.
    #[error("Arti Chat Client error: {0}")]
    ClientError(#[from] ClientError),
    
    /// Attachment Error.
    #[error("Attachment error: {0}")]
    AttachmentError(#[from] AttachmentError),
}

/// Errors related to message.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum MessageError {
    /// Serde Json Error.
    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    /// Hex decode error.
    #[error("Hex decode error: {0}")]
    HexDecodeError(#[from] hex::FromHexError),

    /// Invalid key length.
    #[error("Key length is not 32 bytes.")]
    InvalidKeyLength,

    /// Ed25519 error.
    #[error("ed25519 error: {0}")]
    Ed25519Error(#[from] ed25519_dalek::ed25519::Error),
}

/// Errors related to ratchet algorithm and message encryption logic.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum RatchetError {
    /// Handshake target does not match self.
    #[error("Invalid handshake target.")]
    InvalidHandshakeTarget,
    
    /// Ed25519 error.
    #[error("ed25519 error: {0}")]
    Ed25519Error(#[from] ed25519_dalek::ed25519::Error),
    
    /// HKDF invalid length.
    #[error("HKDF invalid length.")]
    HkdfInvalidLength,

    /// Error decrypting message.
    #[error("Failed to decrypt message.")]
    MessageDecryptError,
    
    /// Invalid key length.
    #[error("Key length is not 32 bytes.")]
    InvalidKeyLength,

    /// Hex decode error.
    #[error("Hex decode error: {0}")]
    HexDecodeError(#[from] hex::FromHexError),
    
    /// I/O Error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Errors related to attachments.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum AttachmentError {
    /// Image error.
    #[error("Image error: {0}")]
    ImageError(#[from] ImageError),
    
    /// Image dimensions too big.
    #[error("Image dimensions exceed limit of {0}.")]
    ImageDimensionsExceedsLimit(String),
    
    /// File size exceeds limit.
    #[error("File size exceeds limit of {0}.")]
    FileSizeExceedsLimit(String),
    
    /// Unsupported format.
    #[error("Unsupported format")]
    FileUnsupportedFormat,
    
    /// I/O Error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
