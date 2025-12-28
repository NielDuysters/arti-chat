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
    if get_socket_stream(SocketPaths::BROADCAST, 5, tokio::time::Duration::from_millis(2000)).await.is_ok() {
        return Ok(());
    }

    let launch_status = if cfg!(target_os = "macos") {
        std::process::Command::new("launchctl").args(["start", "com.arti-chat.daemon"]).spawn()
    } else if cfg!(target_os = "linux") {
        std::process::Command::new("systemctl").args(["--user", "start", "com.arti-chat.daemon.service"]).spawn()
    } else {
        anyhow::bail!(error::DesktopUiError::UnsupportedOperatingSystem);
    };

    match launch_status {
        Ok(_) => return Ok(()),
        Err(_) => anyhow::bail!(error::DesktopUiError::DaemonStartFailure),
    }
}
