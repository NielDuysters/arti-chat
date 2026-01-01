//! Remote Procedure Call commands.

use async_trait::async_trait;
use crate::{client::{self, ClientConfigKey}, db::{self, DbModel, DbUpdateModel}, error::{self, RpcError}, ipc::MessageToUI, ui_focus};

/// List of RPC commands.
#[non_exhaustive]
#[derive(serde::Deserialize)]
#[serde(tag = "cmd")]
pub enum RpcCommand {
    /// Load contacts.
    LoadContacts,

    /// Load chat with a specific contact.
    LoadChat {
        /// Onion ID of the contact whose chat should be loaded.
        onion_id: String,
    },

    /// Send a message to a contact.
    SendMessage {
        /// Onion ID of the message recipient.
        to: String,
        /// Message text to send.
        text: String,
    },

    /// Add a new contact.
    AddContact {
        /// User-defined nickname for the contact.
        nickname: String,
        /// Onion ID of the contact.
        onion_id: String,
        /// Public key of the contact.
        public_key: String,
    },
    
    /// Update an existing contact.
    UpdateContact {
        /// Onion ID of the contact to update.
        onion_id: String,
        /// Optional new nickname for the contact.
        nickname: Option<String>,
        /// Optional new public key for the contact.
        public_key: Option<String>,
    },
    
    /// Load the local user profile.
    LoadUser,

    /// Update local user keys.
    UpdateUser {
        /// Optional new public key.
        public_key: Option<String>,
        /// Optional new private key.
        private_key: Option<String>,
    },

    /// Delete all messages associated with a contact.
    DeleteContactMessages {
        /// Onion ID of the contact whose messages should be deleted.
        onion_id: String,
    },

    /// Delete a contact.
    DeleteContact {
        /// Onion ID of the contact to delete.
        onion_id: String,
    },

    /// Reset the current Tor circuit.
    ResetTorCircuit,

    /// Delete all contacts.
    DeleteAllContacts,

    /// Notify the daemon of the application's focus state.
    SendAppFocusState {
        /// Whether the application is currently focused.
        focussed: bool,
    },

    /// Retrieve a configuration value.
    GetConfigValue {
        /// Configuration key to retrieve.
        key: String,
    },
    
    /// Set a configuration value.
    SetConfigValue {
        /// Configuration key to set.
        key: String,
        /// Value to associate with the configuration key.
        value: String,
    },

    /// Ping the local hidden service.
    PingHiddenService,

    /// Ping the daemon to check availability.
    PingDaemon,
}

/// LoadContacts response.
#[non_exhaustive]
#[derive(serde::Serialize)]
pub struct LoadContactsResponse {
    /// List of returned contacts.
    pub contacts: Vec<serde_json::Value>,
}
impl SendRpcReply for LoadContactsResponse {}

/// LoadChat response.
#[non_exhaustive]
#[derive(serde::Serialize)]
pub struct LoadChatResponse {
    /// List of messages in chat.
    pub messages: Vec<serde_json::Value>,
}
impl SendRpcReply for LoadChatResponse {}

/// General success response for calls only returning a success field.
#[non_exhaustive]
#[derive(serde::Serialize)]
pub struct SuccessResponse {
    /// Success status.
    pub success: bool,
}
impl SendRpcReply for SuccessResponse {}

/// LoadUser response.
#[non_exhaustive]
#[derive(serde::Serialize)]
pub struct LoadUserResponse {
    /// User.
    pub user: serde_json::Value,
}
impl SendRpcReply for LoadUserResponse {}

/// Get config value.
#[non_exhaustive]
#[derive(serde::Serialize)]
pub struct GetConfigValueResponse {
    /// Value of config.
    pub value: String,
}
impl SendRpcReply for GetConfigValueResponse {}

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
                self.handle_load_contacts(tx_rpc, client.db_conn.clone()).await,
            RpcCommand::LoadChat { onion_id } =>
                self.handle_load_chat(onion_id, tx_rpc, client.db_conn.clone()).await,
            RpcCommand::SendMessage { to, text } => 
                self.handle_send_message(to, text, tx_broadcast, client).await,
            RpcCommand::AddContact { nickname, onion_id, public_key } =>
                self.handle_add_contact(nickname, onion_id, public_key, tx_rpc, client.db_conn.clone()).await,
            RpcCommand::UpdateContact { onion_id, nickname, public_key } =>
                self.handle_update_contact(onion_id, nickname.as_deref(), public_key.as_deref(), tx_rpc, client.db_conn.clone()).await,
            RpcCommand::LoadUser => 
                self.handle_load_user(&client.get_identity_unredacted()?, tx_rpc, client.db_conn.clone()).await,
            RpcCommand::UpdateUser { public_key, private_key } =>
                self.handle_update_user(
                    &client.get_identity_unredacted()?,
                    public_key.as_deref(),
                    private_key.as_deref(),
                    tx_rpc,
                    client.db_conn.clone()
                ).await,
            RpcCommand::DeleteContactMessages { onion_id } =>
                self.handle_delete_contact_messages(onion_id, tx_rpc, client.db_conn.clone()).await,
            RpcCommand::DeleteContact { onion_id } =>
                self.handle_delete_contact(onion_id, tx_rpc, client.db_conn.clone()).await,
            RpcCommand::ResetTorCircuit => 
                self.handle_reset_tor_circuit(client, tx_rpc).await,
            RpcCommand::DeleteAllContacts =>
                self.handle_delete_all_contacts(tx_rpc, client.db_conn.clone()).await,
            RpcCommand::SendAppFocusState { focussed } =>
            {
                ui_focus::set_focussed(*focussed);
                Ok(())
            }
            RpcCommand::GetConfigValue { key } =>
                self.handle_get_config_value(key, client, tx_rpc).await,
            RpcCommand::SetConfigValue { key, value } =>
                self.handle_set_config_value(key, value, client).await,
            RpcCommand::PingHiddenService =>
                self.handle_ping_hidden_service(client, tx_rpc).await,
            RpcCommand::PingDaemon =>
                self.handle_ping_daemon(tx_rpc).await,

        }
    }

    // --- Local handlers ---

    /// Handler to load contacts.
    async fn handle_load_contacts(
        &self,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        db_conn: db::DatabaseConnection,
    ) -> Result<(), RpcError> {
        let contacts = db::ContactDb::retrieve_all(Some("last_message_at"), None, db_conn.clone()).await?;

        LoadContactsResponse {
            contacts: contacts
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?

        }.send_rpc_reply(tx)
    }
    
    /// Handler to load chat.
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

    /// Handler to send message.
    async fn handle_send_message(
        &self,
        to: &str,
        text: &str,
        tx: &Option<tokio::sync::mpsc::UnboundedSender<MessageToUI>>,
        client: &client::Client,
    ) -> Result<(), RpcError> {
        // Insert message into db.
        let message_id = db::MessageDb {
            id: 0,
            contact_onion_id: to.to_string(),
            body: text.to_string(),
            timestamp: chrono::Utc::now().timestamp() as i32,
            is_incoming: false,
            sent_status: false,
            verified_status: false,
        }.insert(client.db_conn.clone()).await?;

        // Send message to peer.
        if client.send_message_to_peer(to, text).await.is_ok() {
            // Update sent status.
            db::UpdateMessageDb {
                id: message_id.expect_i64()?,
                sent_status: Some(true),
            }.update(client.db_conn.clone()).await?;
        } 

        // By sending a incoming message to the UI over broadcast, the UI will reload the chat.
        #[derive(serde::Serialize)]
        struct SendIncomingMessage {
            /// HsId from peer we received this message from.
            pub onion_id: String,
        }
        let incoming_message = SendIncomingMessage { onion_id: to.to_string() };
        let incoming_message = serde_json::to_string(&incoming_message)? + "\n";
        if let Some(tx_broadcast) = tx {
            let _ = tx_broadcast.send(MessageToUI::Broadcast(incoming_message));
        }

        Ok(())
    }

    /// Handler to add new contact.
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
            last_viewed_at: chrono::Utc::now().timestamp() as i32,
            amount_unread_messages: 0,
        }.insert(db_conn.clone()).await.is_ok();

        SuccessResponse {
            success,
        }.send_rpc_reply(tx)
    }
    
    /// Handler to update existing contact.
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
    
    /// Handler to load user of app.
    async fn handle_load_user(
        &self,
        onion_id: &str,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        db_conn: db::DatabaseConnection,
    ) -> Result<(), RpcError> {
        let user = db::UserDb::retrieve(onion_id, db_conn.clone()).await?;

        LoadUserResponse {
            user: serde_json::to_value(user)?,
        }.send_rpc_reply(tx)
    }
    
    /// Handler to update user of app.
    async fn handle_update_user(
        &self,
        onion_id: &str,
        public_key: Option<&str>,
        private_key: Option<&str>,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        db_conn: db::DatabaseConnection,
    ) -> Result<(), RpcError> {
        let success = db::UpdateUserDb {
            onion_id: onion_id.into(),
            public_key: public_key.map(|pk| pk.to_string()),
            private_key: private_key.map(|n| n.to_string()),
        }.update(db_conn.clone()).await.is_ok();

        SuccessResponse {
            success,
        }.send_rpc_reply(tx)
    }
    
    /// Handler to delete all messages of a contact.
    async fn handle_delete_contact_messages(
        &self,
        onion_id: &str,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        db_conn: db::DatabaseConnection,
    ) -> Result<(), RpcError> {
        let success = db::MessageDb::delete(onion_id, db_conn.clone()).await.is_ok();
        SuccessResponse {
            success,
        }.send_rpc_reply(tx)
    }
    
    /// Handler to delete a contact and conversation.
    async fn handle_delete_contact(
        &self,
        onion_id: &str,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        db_conn: db::DatabaseConnection,
    ) -> Result<(), RpcError> {
        let success = db::ContactDb::delete(onion_id, db_conn.clone()).await.is_ok();
        SuccessResponse {
            success,
        }.send_rpc_reply(tx)
    }

    /// Handler to reset Tor circuit.
    async fn handle_reset_tor_circuit(
        &self,
        client: &client::Client,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
    ) -> Result<(), RpcError> {
        let success = client.reset_tor_circuit().await.is_ok();
        SuccessResponse {
            success,
        }.send_rpc_reply(tx)
    }

    /// Handler to delete all contacts and conversations.
    async fn handle_delete_all_contacts(
        &self,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        db_conn: db::DatabaseConnection,
    ) -> Result<(), RpcError> {
        let success = db::ContactDb::delete_all(db_conn.clone()).await.is_ok();
        SuccessResponse {
            success,
        }.send_rpc_reply(tx)
    }
    
    /// Handler to get configuration value of user.
    async fn handle_get_config_value(
        &self,
        key: &str,
        client: &client::Client,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
    ) -> Result<(), RpcError> {
        let key = key
            .parse::<ClientConfigKey>()
            .map_err(|_| error::ClientError::InvalidConfigKey)?;
        let cfg = client.config.lock().await;
        let value = cfg.get(&key);
        GetConfigValueResponse {
            value
        }.send_rpc_reply(tx)
    }
    
    /// Handler to update configuration value of user.
    async fn handle_set_config_value(
        &self,
        key: &str,
        value: &str,
        client: &client::Client,
    ) -> Result<(), RpcError> {
        let _ = db::ConfigDb::set(key, value, client.db_conn.clone()).await?;
        client.reload_config().await?;
        Ok(())
    }
    
    /// Handler to ping our own hidden service.
    async fn handle_ping_hidden_service(
        &self,
        client: &client::Client,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
    ) -> Result<(), RpcError> {
        let success = client.is_reachable().await.is_ok();
        SuccessResponse {
            success,
        }.send_rpc_reply(tx)
    }
    
    /// Handler to ping daemon.
    async fn handle_ping_daemon(
        &self,
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
    ) -> Result<(), RpcError> {
        SuccessResponse {
            success: true,
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
