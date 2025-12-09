use crate::model;
use crate::rpc;
use crate::rpc::ReceiveRpcReply;
use crate::rpc::SendRpcCommand;

#[tauri::command]
pub async fn load_contacts() -> Result<Vec<model::Contact>, String> {
    let response = rpc::LoadContacts{}.receive().await.expect("Failed to load contacts.");
    Ok(response.contacts)
}

#[tauri::command]
pub async fn load_chat(onion_id: String) -> Result<Vec<model::Message>, String> {
    let response = rpc::LoadChat{ onion_id }.receive().await.expect("Failed to load chat.");
    Ok(response.messages)
}

#[tauri::command]
pub async fn send_message(to: String, text: String) -> Result<(), String> {
    let _ = rpc::SendMessage{ to, text }.send().await.expect("Failed to send message.");
    Ok(())
}

