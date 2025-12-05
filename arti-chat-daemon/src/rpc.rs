//! Remote Procedure Call commands.

/// List of RPC commands.
#[derive(serde::Deserialize)]
#[serde(tag = "cmd")]
pub enum RpcCommand {
    /// Load contacts.
    LoadContacts,
}
