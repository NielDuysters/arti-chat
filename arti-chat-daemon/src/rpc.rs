//! Remote Procedure Call commands.

use async_trait::async_trait;
use crate::{client, db::{self, DbModel, DbUpdateModel}, error::RpcError, ipc::MessageToUI};

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

    /// Add contact.
    AddContact { nickname: String, onion_id: String, public_key: String },
    
    /// Update contact.
    UpdateContact { onion_id: String, nickname: Option<String>, public_key: Option<String> },
}

/// LoadContacts response.
#[derive(serde::Serialize)]
pub struct LoadContactsResponse {
    /// List of returned contacts.
    pub contacts: Vec<serde_json::Value>,
}
impl SendRpcReply for LoadContactsResponse {}

/// LoadChat response.
#[derive(serde::Serialize)]
pub struct LoadChatResponse {
    /// List of messages in chat.
    pub messages: Vec<serde_json::Value>,
}
impl SendRpcReply for LoadChatResponse {}

/// General success response for calls only returning a success field.
#[derive(serde::Serialize)]
pub struct SuccessResponse {
    /// Success status.
    pub success: bool,
}
impl SendRpcReply for SuccessResponse {}

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
            RpcCommand::AddContact { nickname, onion_id, public_key } =>
                self.handle_add_contact(nickname, onion_id, public_key, &tx_rpc, client.db_conn.clone()).await,
            RpcCommand::UpdateContact { onion_id, nickname, public_key } =>
                self.handle_update_contact(onion_id, nickname.as_deref(), public_key.as_deref(), &tx_rpc, client.db_conn.clone()).await,
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
        // Insert message into db.
        let message_id = db::MessageDb {
            contact_onion_id: to.to_string(),
            body: text.to_string(),
            timestamp: chrono::Utc::now().timestamp() as i32,
            is_incoming: false,
            sent_status: false,
            verified_status: false,
        }.insert(client.db_conn.clone()).await?;

        // Send message to peer.
        if let Ok(_) = client.send_message_to_peer(to, text).await {
            db::UpdateMessageDb {
                id: message_id.expect_i64()?,
                sent_status: Some(true),
            }.update(client.db_conn.clone()).await?;
        } 

        // By sending a incoming message to the UI over broadcast, the UI will reload the chat.
        #[derive(serde::Serialize)]
        struct SendIncomingMessage {
            pub onion_id: String,
        }
        let incoming_message = SendIncomingMessage { onion_id: to.to_string() };
        let incoming_message = serde_json::to_string(&incoming_message)? + "\n";
        if let Some(tx_broadcast) = tx {
            let _ = tx_broadcast.send(MessageToUI::Broadcast(incoming_message));
        }

        Ok(())
    }

    async fn handle_add_contact(
        &self,
        nickname: &str,
        onion_id: &str,
        public_key: &str,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        db_conn: db::DatabaseConnection,
    ) -> Result<(), RpcError> {
        let success = db::ContactDb {
            onion_id: onion_id.into(),
            nickname: nickname.into(),
            public_key: public_key.into(),
            last_message_at: 0,
            last_viewed_at: 0,
        }.insert(db_conn.clone()).await.is_ok();

        SuccessResponse {
            success,
        }.send_rpc_reply(tx)
    }
    
    async fn handle_update_contact(
        &self,
        onion_id: &str,
        nickname: Option<&str>,
        public_key: Option<&str>,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        db_conn: db::DatabaseConnection,
    ) -> Result<(), RpcError> {
        let success = db::UpdateContactDb {
            onion_id: onion_id.into(),
            nickname: nickname.map(|n| n.to_string()),
            public_key: public_key.map(|pk| pk.to_string()),
        }.update(db_conn.clone()).await.is_ok();

        SuccessResponse {
            success,
        }.send_rpc_reply(tx)
    }
}

/// Send error as reply.
pub fn reply_rpc_error(
    tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
    err: &RpcError,
) {
    let _ = tx.send(MessageToUI::Rpc(format!(r#"{{"error":"{err}"}}\n"#)));
}
