//! Remote Procedure Call commands.

use async_trait::async_trait;
use crate::{client, db::{self, DbModel}, error::RpcError, ipc::MessageToUI};

/// List of RPC commands.
#[derive(serde::Deserialize)]
#[serde(tag = "cmd")]
pub enum RpcCommand {
    /// Load contacts.
    LoadContacts,
}

/// LoadContacts response.
#[derive(serde::Serialize)]
pub struct LoadContactsResponse {
    /// List of returned contacts.
    pub contacts: Vec<serde_json::Value>,
}

impl SendRpcReply for LoadContactsResponse {}

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
        tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
        client: &client::Client,
    ) -> Result<(), RpcError> {
        match self {
            RpcCommand::LoadContacts =>
                self.handle_load_contacts(&tx, client.db_conn.clone()).await,
        }
    }

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
}

/// Send error as reply.
pub fn reply_rpc_error(
    tx: &tokio::sync::mpsc::UnboundedSender<MessageToUI>,
    err: &RpcError,
) {
    let _ = tx.send(MessageToUI::Rpc(format!(r#"{{"error":"{err}"}}\n"#)));
}
