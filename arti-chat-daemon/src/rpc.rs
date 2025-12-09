//! Remote Procedure Call commands.

use async_trait::async_trait;
use crate::{client, db::{self, DbModel}, error::RpcError, ipc::MessageToUI};

/// List of RPC commands.
#[derive(serde::Deserialize)]
#[serde(tag = "cmd")]
pub enum RpcCommand {
    /// Load contacts.
    LoadContacts,

    /// Load chat.
    LoadChat { onion_id: String },

    /// Send message.
    SendMessage { to: String, text: String },
}

/// LoadContacts response.
#[derive(serde::Serialize)]
pub struct LoadContactsResponse {
    /// List of returned contacts.
    pub contacts: Vec<serde_json::Value>,
}
impl SendRpcReply for LoadContactsResponse {}

// LoadChat response.
#[derive(serde::Serialize)]
pub struct LoadChatResponse {
    /// List of messages in chat.
    pub messages: Vec<serde_json::Value>,
}
impl SendRpcReply for LoadChatResponse {}

/// Trait to define default behavior to send RPC reply.
#[async_trait]
pub trait SendRpcReply : serde::Serialize {
    /// Send RPC reply.
    fn send_rpc_reply(&self, tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>) -> Result<(), RpcError> {
        let json = serde_json::to_string(&self)? + "\n";
        tx.send(MessageToUI::Rpc(json))?;
        Ok(())
    }
}

// Implementation of RpcCommand containing routing to correct method.
impl RpcCommand {
    /// Route incoming RPC call to correct handler.
    pub async fn route(
        &self,
        tx_rpc: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        tx_broadcast: &Option<tokio::sync::mpsc::UnboundedSender<MessageToUI>>,
        client: &client::Client,
    ) -> Result<(), RpcError> {
        match self {
            RpcCommand::LoadContacts =>
                self.handle_load_contacts(&tx_rpc, client.db_conn.clone()).await,
            RpcCommand::LoadChat { onion_id } =>
                self.handle_load_chat(onion_id, &tx_rpc, client.db_conn.clone()).await,
            RpcCommand::SendMessage { to, text } => 
                self.handle_send_message(to, text, &tx_broadcast, client).await,
        }
    }

    // --- Local handlers ---

    async fn handle_load_contacts(
        &self,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        db_conn: db::DatabaseConnection,
    ) -> Result<(), RpcError> {
        let contacts = db::ContactDb::retrieve_all(None, None, db_conn.clone()).await?;

        LoadContactsResponse {
            contacts: contacts
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?

        }.send_rpc_reply(tx)
    }
    
    async fn handle_load_chat(
        &self,
        onion_id: &str,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        db_conn: db::DatabaseConnection,
    ) -> Result<(), RpcError> {
        let messages = db::MessageDb::retrieve_messages(onion_id, db_conn.clone()).await?;

        LoadChatResponse {
            messages: messages
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?

        }.send_rpc_reply(tx)
    }

    async fn handle_send_message(
        &self,
        to: &str,
        text: &str,
        tx: &Option<tokio::sync::mpsc::UnboundedSender<MessageToUI>>,
        client: &client::Client,
    ) -> Result<(), RpcError> {
        db::MessageDb {
            contact_onion_id: to.to_string(),
            body: text.to_string(),
            timestamp: chrono::Utc::now().timestamp() as i32,
            is_incoming: false,
            sent_status: false,
            verified_status: false,
        }.insert(client.db_conn.clone()).await?;

        Ok(())
    }
}

/// Send error as reply.
pub fn reply_rpc_error(
    tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
    err: &RpcError,
) {
    let _ = tx.send(MessageToUI::Rpc(format!(r#"{{"error":"{err}"}}\n"#)));
}
