//! Logic to communicate between the daemon and the desktop app using Inter-process communication.

use crate::{client, db::{self, DbModel}, error::IpcError, rpc::{self, SendRpcReply}};
use futures::io::WriteHalf;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    net::unix::{OwnedReadHalf, OwnedWriteHalf},
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    time::{timeout, Duration},
};
use tokio::sync::Mutex as TokioMutex;

type DatabaseConnection = std::sync::Arc<TokioMutex<rusqlite::Connection>>; 

/// Possible paths for UnixSockets.
pub struct SocketPaths;
impl SocketPaths {
    /// Socket for RPC (send + reply).
    pub const RPC: &str = "/tmp/arti-chat.rpc.sock";
    
    /// Socket for broadcasting (fire and forget).
    pub const BROADCAST: &str = "/tmp/arti-chat.broadcast.sock";
}

/// Type of message to UI.
pub enum MessageToUI {
    // Broadcast.
    Broadcast(String),
    // RPC command.
    Rpc(String),
}

/// Run our IPC server.
pub async fn run_ipc_server(
    mut message_rx: tokio::sync::mpsc::UnboundedReceiver<String>,   // Receives incoming chat messages
    // from client.
    client: std::sync::Arc<client::Client>,
) -> Result<(), IpcError> {

    // Bind broadcast socket.
    // Used for fire and forget-messages for when we do not expect a reply from the UI.
    // E.g: When the daemon wants to push an incoming message to the UI.
    let _ = std::fs::remove_file(SocketPaths::BROADCAST);
    let broadcast_listener = UnixListener::bind(SocketPaths::BROADCAST)?;
    tracing::info!("Broadcast IPC listening at: {}", SocketPaths::BROADCAST);

    // Bind RPC socket.
    // Used for RPC commands for which the UI expects a reply.
    // E.g: UI requests info of contact, we receive the RPC command and send a reply.
    let _ = std::fs::remove_file(SocketPaths::RPC);
    let rpc_listener = UnixListener::bind(SocketPaths::RPC)?;
    tracing::info!("RPC IPC listening at: {}", SocketPaths::RPC);

    // List of outgoing channels to subscribed UI.
    let broadcast_writers = std::sync::Arc::new(TokioMutex::new(
        std::vec::Vec::<UnboundedSender<MessageToUI>>::new()
    ));

    loop {
        tokio::select! {
            // Incoming chat message.
            Some(message) = message_rx.recv() => {
                // Send to all broadcast listeners and remove dead listeners.
                broadcast_writers.lock().await.retain(|tx| {
                    tx.send(MessageToUI::Broadcast(message.to_string() + "\n")).is_ok()
                });
            }

            // New broadcast subscriber.
            Ok((stream, _)) = broadcast_listener.accept() => {
                tracing::debug!("UI subscribed to IPC broadcast channel.");

                // Write_half is pipe back to the UI.
                let (_, write_half) = stream.into_split();

                // - tx_broadcast: Allows daemon to transmit message to UI.
                // - rx_broadcast: Receives incoming messages from daemon and through `ui_writer_loop` will
                //              write to `write_half`.
                let (tx_broadcast, rx_broadcast) = mpsc::unbounded_channel();

                // Push tx_broadcast writer to broadcast_writers so this UI also receives messages.
                broadcast_writers.lock().await.push(tx_broadcast.clone());

                // Start ui_write_loop with write_half and rx_broadcast.
                // When daemon uses tx_broadcast (via broadcast_writers) to write
                // a message to the UI, rx_writer will read it and forward it
                // to write_half which is the pipe back to the UI.
                tokio::spawn(ui_write_loop(rx_broadcast, write_half));
            }

            // New RPC request.
            Ok((stream, _)) = rpc_listener.accept() => {
                tracing::debug!("UI sent a RPC request.");

                // Write_half is pipe back to the UI.
                // Read_half is required because we want to read the RPC command and it's
                // arguments.
                let (read_half, write_half) = stream.into_split();

                // - tx_rpc: Allows daemon to transmit message to UI.
                // - rx_rpc: Receives incoming messages from daemon and through `ui_writer_loop` will
                //              write to `write_half`.
                let (tx_rpc, rx_rpc) = mpsc::unbounded_channel();

                // Get latest broadcast_writer to send broadcast messages to UI.
                let tx_broadcast = broadcast_writers
                    .lock()
                    .await
                    .last()
                    .cloned();

                // Start ui_write_loop with write_half and rx_rpc.
                // When daemon uses tx_rpc to write
                // a message to the UI, rx_rpc will read it and forward it
                // to write_half which is the pipe back to the UI.
                tokio::spawn(ui_write_loop(rx_rpc, write_half));

                // Handle incoming RPC request.
                tokio::spawn(handle_rpc_call(read_half, tx_rpc, tx_broadcast, client.clone()));
            }
        }
    }
} 

// Writes to UI whenever daemon demands it.
async fn ui_write_loop(
    mut rx: mpsc::UnboundedReceiver<MessageToUI>,   // Receives messages pushed by daemon.
    mut write_half: OwnedWriteHalf,                 // Writer handle to UI.
) {
    while let Some(msg) = rx.recv().await {
        let msg = match msg {
            MessageToUI::Broadcast(m) => m,
            MessageToUI::Rpc(m) => m,
        };

        if let Err(e) = write_half.write_all(msg.as_bytes()).await {
            tracing::error!("IPC error writing to UI: {}", e);
            return; // Broken pipe.
        }
    }
}

// Handle a RPC call coming from the UI.
async fn handle_rpc_call(
    read_half: OwnedReadHalf,                               // Read incoming RPC call.
    tx_rpc: UnboundedSender<MessageToUI>,                // Reply to current RPC call.
    tx_broadcast: Option<UnboundedSender<MessageToUI>>,     // Write to UI.
    client: std::sync::Arc<client::Client>,
) {

    // Convert incoming RPC call to lines.
    let mut lines = BufReader::new(read_half).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue; // Skip empty line.
        }

        tracing::debug!("Imcoming RPC command from UI: {}", line);
        match serde_json::from_str::<rpc::RpcCommand>(&line) {
            Ok(cmd) => {
                if let Err(e) = cmd.route(&tx_rpc, &client).await {
                    rpc::reply_rpc_error(&tx_rpc, &e);
                }
            }
            Err(e) => rpc::reply_rpc_error(&tx_rpc, &e.into()),
        }
    }
}
