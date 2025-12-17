use crate::model;
use crate::rpc;
use crate::rpc::ReceiveRpcReply;
use crate::rpc::SendRpcCommand;

#[tauri::command]
pub async fn load_contacts() -> Result<Vec<model::Contact>, String> {
    let response = rpc::LoadContacts {}.receive().await.map_err(|e| format!("load_contacts failed: {e}"))?;
    Ok(response.contacts)
}

#[tauri::command]
pub async fn load_chat(onion_id: String) -> Result<Vec<model::Message>, String> {
    let response = rpc::LoadChat { onion_id }.receive().await.map_err(|e| format!("load_chat failed: {e}"))?;
    Ok(response.messages)
}

#[tauri::command]
pub async fn send_message(to: String, text: String) -> Result<(), String> {
    rpc::SendMessage { to, text }.send().await.map_err(|e| format!("send_message failed: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn add_contact(nickname: String, onion_id: String, public_key: String) -> Result<bool, String> {
    let response = rpc::AddContact { nickname, onion_id, public_key }.receive().await.map_err(|e| format!("add_contact failed: {e}"))?;
    Ok(response.success)
}

#[tauri::command]
pub async fn update_contact(onion_id: String, nickname: Option<String>, public_key: Option<String>) -> Result<bool, String> {
    let response = rpc::UpdateContact { nickname, onion_id, public_key }.receive().await.map_err(|e| format!("update_contact failed: {e}"))?;
    Ok(response.success)
}

#[tauri::command]
pub async fn load_user() -> Result<model::User, String> {
    let response = rpc::LoadUser {}.receive().await.map_err(|e| format!("load_user failed: {e}"))?;
    Ok(response.user)
}

#[tauri::command]
pub async fn update_user(public_key: Option<String>, private_key: Option<String>) -> Result<bool, String> {
    let response = rpc::UpdateUser { public_key, private_key }.receive().await.map_err(|e| format!("update_user failed: {e}"))?;
    Ok(response.success)
}

#[tauri::command]
pub async fn delete_contact_messages(onion_id: String) -> Result<bool, String> {
    let response = rpc::DeleteContactMessages { onion_id }.receive().await.map_err(|e| format!("delete_contact_messages failed: {e}"))?;
    Ok(response.success)
}

#[tauri::command]
pub async fn delete_contact(onion_id: String) -> Result<bool, String> {
    let response = rpc::DeleteContact { onion_id }.receive().await.map_err(|e| format!("delete_contact failed: {e}"))?;
    Ok(response.success)
}

#[tauri::command]
pub async fn reset_tor_circuit() -> Result<bool, String> {
    let response = rpc::ResetTorCircuit {}.receive().await.map_err(|e| format!("reset_tor_circuit failed: {e}"))?;
    Ok(response.success)
}

#[tauri::command]
pub async fn delete_all_contacts() -> Result<bool, String> {
    let response = rpc::DeleteAllContacts {}.receive().await.map_err(|e| format!("delete_all_contacts failed: {e}"))?;
    Ok(response.success)
}

pub async fn send_focus_state(focussed: bool) -> Result<(), String> {
    rpc::SendAppFocusState { focussed }.send().await.map_err(|e| format!("send_focus_state failed: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn get_config_value(key: String) -> Result<String, String> {
    let response = rpc::GetConfigValue { key }.receive().await.map_err(|e| format!("get_config_value failed: {e}"))?;
    Ok(response.value)
}

#[tauri::command]
pub async fn set_config_value(key: String, value: String) -> Result<(), String> {
    rpc::SetConfigValue { key, value }.send().await.map_err(|e| format!("set_config_value failed: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn ping_hidden_service() -> Result<bool, String> {
    match (rpc::PingHiddenService {}.receive().await) {
        Ok(r) => Ok(r.success),
        Err(e) => Err(format!("ping_hidden_service failed: {e}")),
    }
}

#[tauri::command]
pub async fn ping_daemon() -> Result<bool, String> {
    match (rpc::PingDaemon {}.receive().await) {
        Ok(r) => Ok(r.success),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn restart_daemon() -> Result<(), String> {
    let _ = std::process::Command::new("pkill")
        .arg("arti-chat-daemon-bin")
        .status();

    std::process::Command::new("arti-chat-daemon-bin")
        .spawn()
        .map_err(|e| format!("Failed to restart daemon: {e}"))?;

    Ok(())
}
