//! Used error types.

use thiserror::Error;

/// Errors related to daemon.
#[derive(Error, Clone, Debug)]
pub enum DaemonError {
    /// Error running daemon.
    #[error("Error running daemon.")]
    RunError(String),
}

