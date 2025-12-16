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

#[tauri::command]
pub async fn add_contact(nickname: String, onion_id: String, public_key: String) -> Result<bool, String> {
    let response = rpc::AddContact{ nickname, onion_id, public_key }.receive().await.expect("Failed to add contact.");
    Ok(response.success)
}

#[tauri::command]
pub async fn update_contact(onion_id: String, nickname: Option<String>, public_key: Option<String>) -> Result<bool, String> {
    let response = rpc::UpdateContact{ nickname, onion_id, public_key }.receive().await.expect("Failed to update contact.");
    Ok(response.success)
}

#[tauri::command]
pub async fn load_user() -> Result<model::User, String> {
    let response = rpc::LoadUser{}.receive().await.expect("Failed to load user.");
    Ok(response.user)
}

#[tauri::command]
pub async fn update_user(public_key: Option<String>, private_key: Option<String>) -> Result<bool, String> {
    let response = rpc::UpdateUser{ public_key, private_key }.receive().await.expect("Failed to update user.");
    Ok(response.success)
}

#[tauri::command]
pub async fn delete_contact_messages(onion_id: String) -> Result<bool, String> {
    let response = rpc::DeleteContactMessages{ onion_id }.receive().await.expect("Failed to delete contact messages.");
    Ok(response.success)
}

#[tauri::command]
pub async fn delete_contact(onion_id: String) -> Result<bool, String> {
    let response = rpc::DeleteContact{ onion_id }.receive().await.expect("Failed to delete contact.");
    Ok(response.success)
}

#[tauri::command]
pub async fn reset_tor_circuit() -> Result<bool, String> {
    let response = rpc::ResetTorCircuit{}.receive().await.expect("Failed to reset Tor circuit.");
    Ok(response.success)
}

