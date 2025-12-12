//! Logic to connect with database.

use async_trait::async_trait;
use crate::error;
use rusqlite::{Connection, params, Row, ToSql};
use tokio::sync::Mutex as TokioMutex;

pub type DatabaseConnection = std::sync::Arc<TokioMutex<rusqlite::Connection>>; 

/// Primary key (after insert) can be of type can be String (onion_id) or int (id).
pub enum PrimaryKey<'a> {
    /// Caller knows PK at insert.
    Provided(&'a str),

    /// PK is autoincrement and not known at insert.
    AutoIncrement,
}

/// InsertID can be Integer or Text.
pub enum InsertId {
    /// Text.
    Text(String),
    
    /// Integer.
    Integer(i64),
}

impl InsertId {
     pub fn expect_i64(&self) -> Result<i64, error::DatabaseError> {
        match self {
            InsertId::Integer(id) => Ok(*id),
            InsertId::Text(_) => Err(error::DatabaseError::InvalidPrimaryKeyType),
        }
    }
}

/// Create database tables + return connection.
pub async fn init_database(project_dir: std::path::PathBuf) -> Result<Connection, error::DatabaseError> {
    let conn = Connection::open(database_path(project_dir)?)?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS user (
            onion_id TEXT PRIMARY KEY,
            nickname TEXT NOT NULL,
            public_key TEXT NOT NULL,
            private_key TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS contact (
            onion_id TEXT PRIMARY KEY,
            nickname TEXT NOT NULL,
            public_key TEXT NOT NULL,
            last_message_at INTEGER,
            last_viewed_at INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS message (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contact_onion_id TEXT NOT NULL,
            body TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            is_incoming INTEGER NOT NULL,
            sent_status INTEGER NOT NULL DEFAULT 0,
            verified_status INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (contact_onion_id) REFERENCES contact(onion_id)
        );
        "#,
    )?;

    tracing::debug!("Database connection established");

    Ok(conn)
}

// --- User ---

/// Represents row in user table.
#[derive(serde::Serialize)]
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

/// Type allowing to update user.
#[derive(serde::Serialize)]
pub struct UpdateUserDb {
    /// PK of user to update.
    pub onion_id: String,

    /// Optional update for public_key column.
    pub public_key: Option<String>,
    
    /// Optional update for private_key column.
    pub private_key: Option<String>,
}

impl DbModel for UserDb {
    fn table() -> &'static str { "user" }
    
    fn primary_key(&self) -> PrimaryKey { PrimaryKey::Provided(&self.onion_id) }

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

impl DbUpdateModel<UserDb> for UpdateUserDb {
    fn pk_column() ->  &'static str { "onion_id" }
    
    fn pk_value(&self) -> &dyn ToSql { &self.onion_id }
    
    fn update_values(&self) -> Vec<(&'static str, Option<&dyn ToSql>)> {
        vec![
            ("public_key", self.public_key.as_ref().map(|v| v as &dyn ToSql)),
            ("private_key", self.private_key.as_ref().map(|v| v as &dyn ToSql)),
        ]
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

/// Type allowing to update a contact.
#[derive(serde::Serialize)]
pub struct UpdateContactDb {
    /// PK of contact to update.
    pub onion_id: String,

    /// Optional update for nickname column.
    pub nickname: Option<String>,
    
    /// Optional update for public_key column.
    pub public_key: Option<String>,
}

impl DbModel for ContactDb {
    fn table() -> &'static str { "contact" }

    fn primary_key(&self) -> PrimaryKey { PrimaryKey::Provided(&self.onion_id) }

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

impl DbUpdateModel<ContactDb> for UpdateContactDb {
    fn pk_column() ->  &'static str { "onion_id" }
    
    fn pk_value(&self) -> &dyn ToSql { &self.onion_id }
    
    fn update_values(&self) -> Vec<(&'static str, Option<&dyn ToSql>)> {
        vec![
            ("nickname", self.nickname.as_ref().map(|v| v as &dyn ToSql)),
            ("public_key", self.public_key.as_ref().map(|v| v as &dyn ToSql)),
        ]
    }
}

// --- Message ---

/// Represents row in message table.
#[derive(serde::Serialize)]
pub struct MessageDb {
    /// PK Id of message.
    pub id: i64,

    /// Column contact_onion_id.
    pub contact_onion_id: String,

    /// Column body.
    pub body: String,

    /// Column timestamp.
    pub timestamp: i32,

    /// Column is_incoming.
    pub is_incoming: bool,
    
    /// Column sent_status.
    pub sent_status: bool,

    /// Column verified_status.
    pub verified_status: bool,
}

/// Type allowing to update a message.
#[derive(serde::Serialize)]
pub struct UpdateMessageDb {
    /// PK of message to update.
    pub id: i64,

    /// Optional update for sent_status column.
    pub sent_status: Option<bool>,
}

impl DbModel for MessageDb {
    fn table() -> &'static str { "message" }
    
    fn primary_key(&self) -> PrimaryKey { PrimaryKey::AutoIncrement }

    fn insert_values(&self) -> Vec<(&'static str, &dyn ToSql)> {
        vec![
            ("contact_onion_id", &self.contact_onion_id),
            ("body", &self.body),
            ("timestamp", &self.timestamp),
            ("is_incoming", &self.is_incoming),
            ("sent_status", &self.sent_status),
            ("verified_status", &self.verified_status),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            contact_onion_id: row.get("contact_onion_id")?,
            body: row.get("body")?,
            timestamp: row.get("timestamp")?,
            is_incoming: row.get("is_incoming")?,
            sent_status: row.get("sent_status")?,
            verified_status: row.get("verified_status")?,
        })
    }
}

impl DbUpdateModel<MessageDb> for UpdateMessageDb {
    fn pk_column() ->  &'static str { "id" }
    
    fn pk_value(&self) -> &dyn ToSql { &self.id }
    
    fn update_values(&self) -> Vec<(&'static str, Option<&dyn ToSql>)> {
        vec![
            ("sent_status", self.sent_status.as_ref().map(|v| v as &dyn ToSql)),
        ]
    }
}

impl MessageDb {
    /// Retrieve messages for chat.
    pub async fn retrieve_messages(
        onion_id: &str,
        conn: DatabaseConnection
    ) -> Result<Vec<Self>, error::DatabaseError>  {
        let conn = conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT * FROM MESSAGE
             WHERE
                contact_onion_id = ?
             ORDER BY
                timestamp ASC"
        )?;
        
        let rows = stmt.query_map(params![onion_id], |row| {
            Ok(Self::from_row(row)?)
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
    
    /// Retrieve failed chat message.
    pub async fn failed_messages(
        conn: DatabaseConnection
    ) -> Result<Vec<Self>, error::DatabaseError>  {
        let conn = conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT * FROM MESSAGE
             WHERE
                sent_status = 0
              AND
                is_incoming = 0
             ORDER BY
                timestamp DESC"
        )?;
        
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

/// Public trait implementing default methods (insert, retrieve, update) for Db types.
#[async_trait]
pub trait DbModel : Sized {
    /// Return table name.
    fn table() -> &'static str;

    /// Primary key can be known or is autoincrement.
    fn primary_key(&self) -> PrimaryKey;

    /// List of (column -> values) for INSERT (..column) VALUES (..values).
    fn insert_values(&self) -> Vec<(&'static str, &dyn ToSql)>;

    /// Row to self type.
    fn from_row(row: &Row) -> rusqlite::Result<Self>;

    /// Default insert behavior.
    async fn insert(&self, conn: DatabaseConnection) -> Result<InsertId, error::DatabaseError> {
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
        match self.primary_key() {
            PrimaryKey::Provided(id) => Ok(InsertId::Text(id.into())),
            PrimaryKey::AutoIncrement => Ok(InsertId::Integer(conn.last_insert_rowid())),
        }
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

/// Public trait with default behavior to update a model.
#[async_trait]
pub trait DbUpdateModel<R: DbModel> {
    /// PK column_name for update.
    fn pk_column() -> &'static str;

    /// PK value for WHERE clause.
    fn pk_value(&self) -> &dyn ToSql;
    
    /// List of (column -> values) for UPDATE column=value.
    fn update_values(&self) -> Vec<(&'static str, Option<&dyn ToSql>)>;

    /// Default update behavior.
    async fn update(&self, conn: DatabaseConnection) -> Result<(), error::DatabaseError> {
        let conn = conn.lock().await;
        let mut sets = Vec::new();
        let mut params: Vec<&dyn ToSql> = Vec::new();

        // Push SET and VALUE in vecs where property in Update struct is set.
        self.update_values()
            .into_iter()
            .filter_map(|(c, opt)| opt.map(|v| (c, v)))
            .for_each(|(c, v)| {
                sets.push(format!("{} = ?", c));
                params.push(v);
            });

        if sets.is_empty() {
            return Ok(());
        }

        // Push PK for WHERE.
        params.push(self.pk_value());

        let sql = format!(
            "UPDATE {} SET {} WHERE {} = ?",
            R::table(),
            sets.join(","),
            Self::pk_column(),
        );

        conn.execute(&sql, params.as_slice())?;
        Ok(())
    }
} 

// Helper method to get path do .db file in project_dir.
fn database_path(project_dir: std::path::PathBuf) -> Result<std::path::PathBuf, error::DatabaseError> {
    let path = project_dir.join("arti-chat.db");
    Ok(path)
}
