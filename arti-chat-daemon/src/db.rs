//! Logic to connect with database.

use async_trait::async_trait;
use crate::error;
use rusqlite::{Connection, params, Row, ToSql};
use tokio::sync::Mutex as TokioMutex;

pub type DatabaseConnection = std::sync::Arc<TokioMutex<rusqlite::Connection>>; 

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

// --- User ---

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

impl DbModel for UserDb {
    fn table() -> &'static str { "user" }

    fn insert_values(&self) -> Vec<(&'static str, &dyn ToSql)> {
        vec![
            ("onion_id", &self.onion_id),
            ("nickname", &self.nickname),
            ("private_key", &self.private_key),
            ("public_key", &self.public_key),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            onion_id: row.get("onion_id")?,
            nickname: row.get("nickname")?,
            private_key: row.get("private_key")?,
            public_key: row.get("public_key")?,
        })
    }
}

// --- Contact ---

/// Represents row in contact table.
#[derive(serde::Serialize)]
pub struct ContactDb {
    /// Column onion_id.
    pub onion_id: String,

    /// Column nickname.
    pub nickname: String,

    /// Column public_key.
    pub public_key: String,

    /// Column last_message_at.
    pub last_message_at: i32,
    
    /// Column last_viewed_at.
    pub last_viewed_at: i32,
}

impl DbModel for ContactDb {
    fn table() -> &'static str { "contact" }

    fn insert_values(&self) -> Vec<(&'static str, &dyn ToSql)> {
        vec![
            ("onion_id", &self.onion_id),
            ("nickname", &self.nickname),
            ("public_key", &self.public_key),
            ("last_message_at", &self.last_message_at),
            ("last_viewed_at", &self.last_viewed_at),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            onion_id: row.get("onion_id")?,
            nickname: row.get("nickname")?,
            public_key: row.get("public_key")?,
            last_message_at: row.get("last_message_at")?,
            last_viewed_at: row.get("last_viewed_at")?,
        })
    }
}


/// Public trait implementing default methods (insert, retrieve, update) for Db types.
#[async_trait]
pub trait DbModel : Sized {
    /// Return table name.
    fn table() -> &'static str;

    /// List of (column -> values) for INSERT (..column) VALUES (..values).
    fn insert_values(&self) -> Vec<(&'static str, &dyn ToSql)>;

    /// Row to self type.
    fn from_row(row: &Row) -> rusqlite::Result<Self>;

    /// Default insert behavior.
    async fn insert(&self, conn: DatabaseConnection) -> Result<(), error::DatabaseError> {
        let conn = conn.lock().await;

        let columns: Vec<&str> = self.insert_values().iter().map(|(c, _)| *c).collect();
        let values: Vec<&dyn ToSql> = self.insert_values().iter().map(|(_, v)| *v).collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            Self::table(),
            columns.join(","),
            vec!["?"; values.len()].join(","),
        );

        conn.execute(&sql, values.as_slice())?;
        Ok(())
    }

    /// Default select behavior.
    async fn retrieve(onion_id: &str, conn: DatabaseConnection) -> Result<Self, error::DatabaseError> {
        let conn = conn.lock().await;
        let sql = format!("SELECT * FROM {} WHERE onion_id = ?", Self::table());

        Ok(conn.query_row(&sql, [onion_id], |row| Self::from_row(row))?)
    }

    /// Default select all behavior.
    async fn retrieve_all(
        order_column: Option<&str>,
        desc: Option<bool>,
        conn: DatabaseConnection,
    ) -> Result<Vec<Self>, error::DatabaseError> {
        let conn = conn.lock().await;
        let mut sql = format!("SELECT * FROM {}", Self::table());

        if let Some(oc) = order_column {
            sql = format!("{} ORDER BY {} {}", sql, oc, if desc.unwrap_or(true) { "DESC" } else { "ASC" });
        }

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(Self::from_row(row)?)
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}

// Helper method to get path do .db file in project_dir.
fn database_path(project_dir: std::path::PathBuf) -> Result<std::path::PathBuf, error::DatabaseError> {
    let path = project_dir.join("arti-chat.db");
    Ok(path)
}
