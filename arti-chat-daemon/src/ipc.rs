//! Logic to communicate between the daemon and the desktop app using Inter-process communication.

use crate::{client, error::IpcError, rpc};
use tokio::sync::Mutex as TokioMutex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, WriteHalf, ReadHalf},
    sync::mpsc::{self, UnboundedSender},
};
 use interprocess::local_socket::{
            tokio::{prelude::*, Stream},
            GenericFilePath, GenericNamespaced, ListenerOptions};

/// Names for sockets.
#[non_exhaustive]
pub struct SocketNames;

impl SocketNames {
    const BROADCAST_FS: &'static str = "/tmp/arti-chat.broadcast.sock";
    const RPC_FS: &'static str = "/tmp/arti-chat.rpc.sock";

    /// Broadcast socket name.
    pub fn broadcast() -> interprocess::local_socket::Name<'static> {
        if GenericNamespaced::is_supported() {
            "arti-chat.broadcast.sock"
                .to_ns_name::<GenericNamespaced>()
                .unwrap()
        } else {
            Self::BROADCAST_FS
                .to_fs_name::<GenericFilePath>()
                .unwrap()
        }
    }

    /// RPC socket name.
    pub fn rpc() -> interprocess::local_socket::Name<'static> {
        if GenericNamespaced::is_supported() {
            "arti-chat.rpc.sock"
                .to_ns_name::<GenericNamespaced>()
                .unwrap()
        } else {
            Self::RPC_FS
                .to_fs_name::<GenericFilePath>()
                .unwrap()
        }
    }

    /// Cleanup zombie socket files for Unix systems.
    pub fn cleanup_filesystem_sockets() {
        let _ = std::fs::remove_file(Self::BROADCAST_FS);
        let _ = std::fs::remove_file(Self::RPC_FS);
    }
}

/// Type of message to UI.
#[non_exhaustive]
pub enum MessageToUI {
    /// Broadcast.
    Broadcast(String),
    /// RPC command.
    Rpc(String),
}

/// Run our IPC server.
pub async fn run_ipc_server(
    mut message_rx: tokio::sync::mpsc::UnboundedReceiver<String>, // Receives incoming chat messages
    // from client.
    client: std::sync::Arc<client::Client>,
) -> Result<(), IpcError> {
    SocketNames::cleanup_filesystem_sockets();

    // Bind broadcast socket.
    // Used for fire and forget-messages for when we do not expect a reply from the UI.
    // E.g: When the daemon wants to push an incoming message to the UI.
    let opts_broadcast = ListenerOptions::new().name(SocketNames::broadcast());
    let broadcast_listener = opts_broadcast.create_tokio()?;
    tracing::info!("Broadcast IPC listening at: {:?}", SocketNames::broadcast());

    // Bind RPC socket.
    // Used for RPC commands for which the UI expects a reply.
    // E.g: UI requests info of contact, we receive the RPC command and send a reply.
    //let _ = std::fs::remove_file(SocketPaths::RPC);
    let opts_rpc = ListenerOptions::new().name(SocketNames::rpc());
    let rpc_listener = opts_rpc.create_tokio()?;
    tracing::info!("RPC IPC listening at: {:?}", SocketNames::rpc());

    // List of outgoing channels to subscribed UI.
    let broadcast_writers = std::sync::Arc::new(TokioMutex::new(std::vec::Vec::<
        UnboundedSender<MessageToUI>,
    >::new()));

    // Spawn task to retry failed messages.
    let bw_clone = broadcast_writers.clone();
    let client_clone = client.clone();
    tokio::spawn(async move {
        let _ = client_clone.retry_failed_messages(bw_clone).await;
    });

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
            Ok(conn) = broadcast_listener.accept() => {
                tracing::debug!("UI subscribed to IPC broadcast channel.");

                // Write_half is pipe back to the UI.
                let (_, write_half) = tokio::io::split(conn);

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
            Ok(conn) = rpc_listener.accept() => {
                tracing::debug!("UI sent a RPC request.");

                // Write_half is pipe back to the UI.
                // Read_half is required because we want to read the RPC command and it's
                // arguments.
                let (read_half, write_half) = tokio::io::split(conn);

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

/// Writes to UI whenever daemon demands it.
async fn ui_write_loop(
    mut rx: mpsc::UnboundedReceiver<MessageToUI>, // Receives messages pushed by daemon.
    mut write_half: WriteHalf<Stream>,               // Writer handle to UI.
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

/// Handle a RPC call coming from the UI.
async fn handle_rpc_call(
    read_half: ReadHalf<Stream>,             // Read incoming RPC call.
    tx_rpc: UnboundedSender<MessageToUI>, // Reply to current RPC call.
    tx_broadcast: Option<UnboundedSender<MessageToUI>>, // Write to UI.
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
                if let Err(e) = cmd.route(&tx_rpc, &tx_broadcast, &client).await {
                    rpc::reply_rpc_error(&tx_rpc, &e);
                }
            }
            Err(e) => rpc::reply_rpc_error(&tx_rpc, &e.into()),
        }
    }
}
