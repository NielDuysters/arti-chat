//! Logic to connect with database.

use crate::error;
use rusqlite::{Connection, params};
use tokio::sync::Mutex as TokioMutex;

type DatabaseConnection = std::sync::Arc<TokioMutex<rusqlite::Connection>>; 

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

/// Represents row in user table.
pub struct UserDb {
    /// Column onion_id.
    pub onion_id: String,

    /// Column nickname.
    pub nickname: String,

    /// Column private_key.
    pub private_key: String,

    /// Column public_key.
    pub public_key: String,
}

impl UserDb {
    /// Insert new user.
    /// If an user with given onion_id already exists, the user is not inserted nor updated.
    pub async fn insert(&self, conn: DatabaseConnection) -> Result<(), error::DatabaseError> {
        let conn = conn.lock().await;
        conn.execute(
            "INSERT INTO user
                (onion_id, nickname, private_key, public_key)
            VALUES
                (?, ?, ?, ?)",
            params![
                self.onion_id,
                self.nickname,
                self.private_key,
                self.public_key,
            ]
        )?;

        Ok(())
    }

    /// Retrieve user by onion_id.
    pub async fn retrieve(conn: DatabaseConnection, onion_id: &str) -> Result<Self, error::DatabaseError> {
        let conn = conn.lock().await;
        conn.query_row(
            "SELECT
                onion_id, nickname, private_key, public_key
            FROM user
            WHERE onion_id = ?",
            params![onion_id],
            |row| {
                Ok(Self {
                    onion_id: row.get(0)?,
                    nickname: row.get(1)?,
                    private_key: row.get(2)?,
                    public_key: row.get(3)?,
                })
            }
        ).map_err(Into::into)
    }
}


fn database_path(project_dir: std::path::PathBuf) -> Result<std::path::PathBuf, error::DatabaseError> {
    let path = project_dir.join("arti-chat.db");
    Ok(path)
}
