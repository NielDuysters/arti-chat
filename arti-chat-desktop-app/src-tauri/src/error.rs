//! Error types for desktop UI app.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DesktopUiError {
    #[error("Socket timeout.")]
    SocketTimeout,
    
    #[error("Daemon couldn't launch.")]
    DaemonStartFailure,
    
    #[error("Unsupported operating system.")]
    UnsupportedOperatingSystem,
}
