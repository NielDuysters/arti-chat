//! Logic to communicate between the daemon and the desktop app using Inter-process communication.

use crate::error::IpcError;
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

const RPC_SOCK: &str = "/tmp/arti-chat.rpc.sock";
const BROADCAST_SOCK: &str = "/tmp/arti-chat.broadcast.sock";

// Type of message to UI.
enum MessageToUI {
    // Broadcast.
    Broadcast(String),
    // RPC command.
    Rpc(String),
}

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

    // List of outgoing channels to subscribed UI.
    let broadcast_writers = std::sync::Arc::new(TokioMutex::new(
        std::vec::Vec::<UnboundedSender<MessageToUI>>::new()
    ));

    loop {
        tokio::select! {
            
            // New broadcast subscriber.
            Ok((stream, _)) = broadcast_listener.accept() => {
                tracing::debug!("UI subscribed to IPC broadcast channel.");

                // Write_half is pipe back to the UI.
                let (_, write_half) = stream.into_split();
               
                // - tx_writer: Allows daemon to transmit message to UI.
                // - rx_writer: Receives incoming messages from daemon and through `ui_writer_loop` will
                //              write to `write_half`.
                let (tx_writer, rx_writer) = mpsc::unbounded_channel();

                // Push tx_writer writer to broadcast_writers so this UI also receives messages.
                broadcast_writers.lock().await.push(tx_writer.clone());

                // Start ui_write_loop with write_half and rx_writer.
                // When daemon uses tx_writer (via broadcast_writers) to write
                // a message to the UI, rx_writer will read it and forward it
                // to write_half which is the pipe back to the UI.
                tokio::spawn(ui_write_loop(rx_writer, write_half));
            }

            // New RPC request.
            Ok((stream, _)) = rpc_listener.accept() => {
                tracing::debug!("UI sent a RPC request.");

                // Write_half is pipe back to the UI.
                // Read_half is required because we want to read the RPC command and it's
                // arguments.
                let (read_half, write_half) = stream.into_split();
                
                // - tx_writer: Allows daemon to transmit message to UI.
                // - rx_writer: Receives incoming messages from daemon and through `ui_writer_loop` will
                //              write to `write_half`.
                let (tx_writer, rx_writer) = mpsc::unbounded_channel();

                // Get latest broadcast_writer to send broadcast messages to UI.
                let tx_broadcast = broadcast_writers
                    .lock()
                    .await
                    .last()
                    .cloned();

                // Start ui_write_loop with write_half and rx_writer.
                // When daemon uses tx_writer (via broadcast_writers) to write
                // a message to the UI, rx_writer will read it and forward it
                // to write_half which is the pipe back to the UI.
                tokio::spawn(ui_write_loop(rx_writer, write_half));

                // Handle incoming RPC request.
                tokio::spawn(handle_rpc_call(read_half, tx_writer, tx_broadcast));
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
    tx_writer: UnboundedSender<MessageToUI>,                // Reply to current RPC call.
    tx_broadcast: Option<UnboundedSender<MessageToUI>>,     // Write to UI.
) {

    // Convert incoming RPC call to lines.
    let mut lines = BufReader::new(read_half).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue; // Skip empty line.
        }

        tracing::debug!("Imcoming RPC command from UI: {}", line);
    }
}
