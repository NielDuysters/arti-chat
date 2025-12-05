//! Logic to communicate between the daemon and the desktop app using Inter-process communication.

use crate::error::IpcError;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    net::unix::{OwnedReadHalf, OwnedWriteHalf},
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    time::{timeout, Duration},
};
use tokio::sync::Mutex as TokioMutex;

type DatabaseConnection = std::sync::Arc<TokioMutex<rusqlite::Connection>>; 

const RPC_SOCK: &str = "/tmp/arti-chat.rpc.sock";
const BROADCAST_SOCK: &str = "/tmp/arti-chat.broadcast.sock";

/// Run our IPC server.
pub async fn run_ipc_server() -> Result<(), IpcError> {
    
    // Bind broadcast socket.
    // Used for fire and forget-messages for when we do not expect a reply from the UI.
    // E.g: When the daemon wants to push an incoming message to the UI.
    let _ = std::fs::remove_file(BROADCAST_SOCK);
    let broadcast_listener = UnixListener::bind(BROADCAST_SOCK)?;
    tracing::info!("Broadcast IPC listening at: {}", BROADCAST_SOCK);

    // Bind RPC socket.
    // Used for RPC commands for which the UI expects a reply.
    // E.g: UI requests info of contact, we receive the RPC command and send a reply.
    let _ = std::fs::remove_file(RPC_SOCK);
    let rpc_listener = UnixListener::bind(RPC_SOCK)?;
    tracing::info!("RPC IPC listening at: {}", RPC_SOCK);

    Ok(())
} 
