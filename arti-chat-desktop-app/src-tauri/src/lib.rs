use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};

pub mod commands;
pub mod error;
pub mod ipc;
pub mod model;
pub mod rpc;

// Atomic bool indicating if the user has the app open and focussed.
static APP_FOCUSED: AtomicBool = AtomicBool::new(true);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|_, event| match event {
            // Update focus state when user changes focus or closes app.
             tauri::WindowEvent::Focused(focused) => {
                APP_FOCUSED.store(*focused, Ordering::Relaxed);
            }
            tauri::WindowEvent::CloseRequested { .. } => {
                tauri::async_runtime::spawn(async move {
                    let _ = commands::send_focus_state(false).await; 
                });
            }
            _ => {}
        })
        .setup(|app| {
            let app_handle = app.handle().clone();

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
                    let _ = app_handle.emit("incoming-message", line);
                }
            });

            // Separate async task to send focus state every 30 seconds.
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                    let focused = APP_FOCUSED.load(Ordering::Relaxed);
                    let _ = commands::send_focus_state(focused).await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_contacts,
            commands::load_chat,
            commands::send_message,
            commands::add_contact,
            commands::update_contact,
            commands::load_user,
            commands::update_user,
            commands::delete_contact_messages,
            commands::delete_contact,
            commands::reset_tor_circuit,
            commands::delete_all_contacts,
            commands::get_config_value,
            commands::set_config_value,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

