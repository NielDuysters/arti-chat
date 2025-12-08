//! Error types for desktop UI app.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DesktopUiError {
    #[error("Socket timeout.")]
    SocketTimeout,
}
