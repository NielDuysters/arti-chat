use crate::model;
use tokio::net::UnixStream;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};

#[tauri::command]
pub async fn load_contacts() -> Result<Vec<model::Contact>, String> {
    let socket_path = "/tmp/arti-chat.ipc.sock";

    // Connect to daemon
    let mut stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;

    // Build RPC request
    let req = serde_json::json!({
        "cmd": "LoadContacts",
    })
    .to_string()
        + "\n";

    // Send request
    stream
        .write_all(req.as_bytes())
        .await
        .map_err(|e| format!("Failed to write to daemon: {}", e))?;

    stream.flush().await.unwrap();

    // Prepare to read response
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    // Read one full JSON line
    reader
        .read_line(&mut line)
        .await
        .map_err(|e| format!("IPC read error: {}", e))?;

    if line.trim().is_empty() {
        return Err("Empty response from daemon".into());
    }

    // Parse JSON
    let parsed: model::ContactResponse =
        serde_json::from_str(&line).map_err(|e| format!("JSON parse error: {}", e))?;

    Ok(parsed.contacts)
}

