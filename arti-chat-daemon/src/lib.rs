#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![allow(renamed_and_removed_lints)]
#![allow(unknown_lints)]
#![warn(missing_docs)]
#![warn(noop_method_call)]
#![warn(unreachable_pub)]
#![warn(clippy::all)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::cargo_common_metadata)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::checked_conversions)]
#![warn(clippy::cognitive_complexity)]
#![deny(clippy::debug_assert_with_mut_call)]
#![deny(clippy::exhaustive_enums)]
#![deny(clippy::exhaustive_structs)]
#![deny(clippy::expl_impl_clone_on_copy)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::implicit_clone)]
#![deny(clippy::large_stack_arrays)]
#![warn(clippy::manual_ok_or)]
#![deny(clippy::missing_docs_in_private_items)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::option_option)]
#![deny(clippy::print_stderr)]
#![deny(clippy::print_stdout)]
#![warn(clippy::rc_buffer)]
#![deny(clippy::ref_option_ref)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unchecked_time_subtraction)]
#![deny(clippy::unnecessary_wraps)]
#![warn(clippy::unseparated_literal_suffix)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::mod_module_files)]
#![allow(clippy::let_unit_value)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::significant_drop_in_scrutinee)] 
#![allow(clippy::result_large_err)]
#![allow(clippy::needless_raw_string_hashes)] 
#![allow(clippy::needless_lifetimes)] 
#![allow(mismatched_lifetime_syntaxes)]

use tokio::sync::Mutex as TokioMutex;

pub mod client;
pub mod db;
pub mod error;
pub mod ipc;
pub mod message;
pub mod rpc;
pub mod ui_focus;

/// Project directory storing sqlite db + config.
pub static PROJECT_DIR: once_cell::sync::Lazy<directories::ProjectDirs> = once_cell::sync::Lazy::new(|| {
    directories::ProjectDirs::from("com", "arti-chat", "desktop")
        .expect("Failed to determine project directories")
});

/// Run daemon and start all required services.
pub async fn run() -> Result<(), error::DaemonError> {
    tracing::info!("Daemon entrypoint reached.");

    let project_dir = create_project_dir()?;

    // Create database connection.
    let db_conn = db::init_database(project_dir).await?;
    let db_conn = std::sync::Arc::new(TokioMutex::new(db_conn));

    // Create Tor client + launch hidden service.
    let client = client::Client::launch(db_conn).await?;
    let client = std::sync::Arc::new(client);
    let onion_address = client.get_identity_unredacted()?;
    tracing::info!("Onion address: {}", onion_address);

    // Channel to receive incoming messages in the client, and sent them to our
    // IPC server to broadcast them to the UI.
    let (message_tx, message_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Start IPC server.
    tokio::spawn(ipc::run_ipc_server(message_rx, client.clone()));

    // Service hidden service.
    client.serve(message_tx).await?;


    Ok(())
}

fn create_project_dir() -> Result<std::path::PathBuf, error::DaemonError> {
    let path = PROJECT_DIR.data_local_dir();
    std::fs::create_dir_all(path)?;

    tracing::info!("Created project directory: {:?}", path);

    Ok(path.to_path_buf())
}
