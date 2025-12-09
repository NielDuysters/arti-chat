//! List of Remote Procedure Call commands.

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use crate::ipc;
use crate::model;

/// --- Load contacts ---
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoadContacts {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LoadContactsResponse {
    pub contacts: Vec<model::Contact>,
}

impl SendRpcCommand for LoadContacts {}
impl ReceiveRpcReply<LoadContactsResponse> for LoadContacts {} 

/// --- Load chat ---
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoadChat {
    pub onion_id: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LoadChatResponse {
    pub messages: Vec<model::Message>,
}

impl SendRpcCommand for LoadChat {}
impl ReceiveRpcReply<LoadChatResponse> for LoadChat {} 

/// Trait to send types as RPC command.
#[async_trait]
pub trait SendRpcCommand: Sized + serde::Serialize {
    async fn send(&self) -> anyhow::Result<tokio::net::UnixStream> {
        // Helper method to get command name from type.
        fn cmd_name<T>() -> &'static str {
            std::any::type_name::<T>()
                .rsplit("::")
                .next()
                .unwrap()
        }

        // Make connection to RPC socket.
        let mut stream = ipc::get_socket_stream(
            ipc::SocketPaths::RPC,
            2,
            tokio::time::Duration::from_millis(1000),
        ).await?;

        // Craft RPC command.
        let mut rpc_cmd = serde_json::to_value(self)?
            .as_object()
            .unwrap()
            .clone();
        rpc_cmd.insert("cmd".into(), cmd_name::<Self>().into());
        let rpc_json = serde_json::to_string(&rpc_cmd)? + "\n";

        // Write to RPC stream.
        stream
            .write_all(rpc_json.as_bytes())
            .await?;
        stream.flush().await.ok();

        Ok(stream)
    }
}

/// Trait to receive reply of RPC command.
#[async_trait]
pub trait ReceiveRpcReply<R>: SendRpcCommand
where R: serde::de::DeserializeOwned {
    async fn receive(&self) -> anyhow::Result<R> {
        // Send RPC call.
        let stream = self.send().await?;

        let mut reader = tokio::io::BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        if line.is_empty() {
            anyhow::bail!("Empty RPC response.");
        }

        tracing::debug!("Received RPC response: {}", line);

        let response = serde_json::from_str::<R>(&line)?;
        Ok(response)
    }
}
