use crate::model;
use crate::rpc;
use crate::rpc::ReceiveRpcReply;

#[tauri::command]
pub async fn load_contacts() -> Result<Vec<model::Contact>, String> {
    let response = rpc::LoadContacts{}.receive().await.expect("Failed to load contacts.");
    Ok(response.contacts)
}

