//! Used error types.

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
