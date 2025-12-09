use tokio::io::{AsyncBufReadExt, BufReader};

pub mod commands;
pub mod error;
pub mod ipc;
pub mod model;
pub mod rpc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            // Separate async task to receive messages from broadcast.
            tauri::async_runtime::spawn(async move {
                let broadcast_stream = ipc::get_socket_stream(
                    ipc::SocketPaths::BROADCAST,
                    20,
                    tokio::time::Duration::from_millis(1000),
                )
                .await
                .expect("Failed to get broadcast stream.");

                let mut lines = BufReader::new(broadcast_stream).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::info!("Received message: {}", line);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_contacts,
            commands::load_chat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
