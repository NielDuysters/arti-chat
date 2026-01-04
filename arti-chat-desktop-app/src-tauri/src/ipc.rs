//! Logic to make the desktop UI communicate with the daemon using Inter-process communication.

 use interprocess::local_socket::{
            tokio::{prelude::*, Stream},
            GenericFilePath, GenericNamespaced};
use crate::error;

/// Names for sockets.
#[non_exhaustive]
pub struct SocketNames;

impl SocketNames {
    pub fn rpc() -> interprocess::local_socket::Name<'static> {
        if GenericNamespaced::is_supported() {
            "arti-chat.rpc.sock".to_ns_name::<GenericNamespaced>().expect("Failed to convert to filesystem path-type local socket name.")
        } else {
            "/tmp/arti-chat.rpc.sock".to_fs_name::<GenericFilePath>().expect("Failed to convert to namespaced local socket name.")
        }
    }

    pub fn broadcast() -> interprocess::local_socket::Name<'static> {
        if GenericNamespaced::is_supported() {
            "arti-chat.broadcast.sock".to_ns_name::<GenericNamespaced>().expect("Failed to convert to filesystem path-type local socket name.")
        } else {
            "/tmp/arti-chat.broadcast.sock".to_fs_name::<GenericFilePath>().expect("Failed to convert to namespaced local socket name.")
        }
    }
}

/// Connect to a local IPC socket with retry logic.
pub async fn get_socket_stream(
    name: interprocess::local_socket::Name<'static>,
    retries: u8,
    delay: tokio::time::Duration,
) -> anyhow::Result<Stream> {
    for _ in 0..retries {
        match Stream::connect(name.clone()).await {
            Ok(stream) => return Ok(stream),
            Err(_) => tokio::time::sleep(delay).await,
        }
    }

    anyhow::bail!(error::DesktopUiError::SocketTimeout);
}

pub async fn launch_daemon() -> anyhow::Result<()> {
    if get_socket_stream(
        SocketNames::broadcast(),
        5,
        tokio::time::Duration::from_millis(2000),
    )
    .await
    .is_ok()
    {
        return Ok(());
    }

    let status = start_daemon()?;

    if !status.success() {
        anyhow::bail!(error::DesktopUiError::DaemonStartFailure);
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn start_daemon() -> anyhow::Result<std::process::ExitStatus> {
    let uid = nix::unistd::Uid::current();

    Ok(std::process::Command::new("launchctl")
        .args([
            "kickstart",
            "-k",
            &format!("gui/{}/com.arti-chat.daemon", uid.as_raw()),
        ])
        .status()?)
}

#[cfg(target_os = "linux")]
fn start_daemon() -> anyhow::Result<std::process::ExitStatus> {
    Ok(std::process::Command::new("systemctl")
        .args(["--user", "start", "com.arti-chat.daemon.service"])
        .status()?)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn start_daemon() -> anyhow::Result<std::process::ExitStatus> {
    anyhow::bail!(error::DesktopUiError::UnsupportedOperatingSystem)
}
