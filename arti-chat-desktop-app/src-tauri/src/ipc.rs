//! Logic to make the desktop UI communicate with the daemon using Inter-process communication.

use crate::error;

pub struct SocketPaths;
impl SocketPaths {
    pub const BROADCAST: &str = "/tmp/arti-chat.broadcast.sock";
    pub const RPC: &str = "/tmp/arti-chat.rpc.sock";
}

pub async fn get_socket_stream(
    path: &str,
    retries: u8,
    delay: tokio::time::Duration,
) -> anyhow::Result<tokio::net::UnixStream> {
    for _ in 0..retries {
        match tokio::net::UnixStream::connect(path).await {
            Ok(stream) => return Ok(stream),
            Err(_) => tokio::time::sleep(delay).await,
        }
    }

    anyhow::bail!(error::DesktopUiError::SocketTimeout);
}

pub async fn launch_daemon() -> anyhow::Result<()> {
    match get_socket_stream(SocketPaths::BROADCAST, 5, tokio::time::Duration::from_millis(2000)).await {
        Ok(_) => return Ok(()),
        Err(_) => {
            match std::process::Command::new("arti-chat-daemon-bin").spawn() {
                Ok(_) => return Ok(()),
                Err(_) => anyhow::bail!(error::DesktopUiError::DaemonStartFailure),
            }
        }
    }
}
