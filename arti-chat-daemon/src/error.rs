//! Used error types.

use thiserror::Error;

/// Errors related to daemon.
#[derive(Error, Clone, Debug)]
pub enum DaemonError {
    /// Error running daemon.
    #[error("Error running daemon.")]
    RunError(String),
}

/// Errors related to client.
#[derive(Error, Clone, Debug)]
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
}
