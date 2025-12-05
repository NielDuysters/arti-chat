//! Logic to connect with database.

use crate::error;
use rusqlite::{Connection, params};

fn database_path(project_dir: std::path::PathBuf) -> Result<std::path::PathBuf, error::DatabaseError> {
    let path = project_dir.join("arti-chat.db");
    Ok(path)
}

/// Create database tables + return connection.
pub async fn init_database(project_dir: std::path::PathBuf) -> Result<Connection, error::DatabaseError> {
    let conn = Connection::open(database_path(project_dir)?)?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS user (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            onion_id TEXT NOT NULL UNIQUE,
            nickname TEXT NOT NULL,
            public_key TEXT NOT NULL,
            private_key TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS contact (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            onion_id TEXT NOT NULL UNIQUE,
            nickname TEXT NOT NULL,
            public_key TEXT NOT NULL,
            last_message_at INTEGER,
            last_viewed_at INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS message (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contact_id INTEGER NOT NULL,
            sender_onion_id TEXT NOT NULL,
            body TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            is_incoming INTEGER NOT NULL,
            sent_status INTEGER NOT NULL DEFAULT 0,
            verified_status INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(contact_id) REFERENCES contact(id)
        );
        "#,
    )?;

    tracing::debug!("Database connection established");

    Ok(conn)
}
