//! Logic to connect with database.

use crate::error;
use async_trait::async_trait;
use rand::RngCore;
use rusqlite::{Connection, Row, ToSql, params, params_from_iter};
use tokio::sync::Mutex as TokioMutex;

/// Type for rusqlite database connection.
pub type DatabaseConnection = std::sync::Arc<TokioMutex<rusqlite::Connection>>;

/// Primary key (after insert) can be of type can be String (onion_id) or int (id).
#[non_exhaustive]
pub enum PrimaryKey<'a> {
    /// Caller knows PK at insert.
    Provided(&'a str),

    /// PK is autoincrement and not known at insert.
    AutoIncrement,
}

/// InsertID can be Integer or Text.
#[non_exhaustive]
pub enum InsertId {
    /// Text.
    Text(String),

    /// Integer.
    Integer(i64),
}

impl InsertId {
    /// Get insertId only if type  i64.
    pub fn expect_i64(&self) -> Result<i64, error::DatabaseError> {
        match self {
            InsertId::Integer(id) => Ok(*id),
            InsertId::Text(_) => Err(error::DatabaseError::InvalidPrimaryKeyType),
        }
    }
}

/// Create database tables + return connection.
pub async fn init_database(
    project_dir: std::path::PathBuf,
) -> Result<Connection, error::DatabaseError> {
    let conn = Connection::open(database_path(&project_dir))?;

    let db_key = retrieve_db_encryption_key()?;
    conn.pragma_update(None, "key", &db_key)?;
    conn.execute_batch(
        r#"
        PRAGMA cipher_memory_security = ON;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS user (
            onion_id TEXT PRIMARY KEY,
            nickname TEXT NOT NULL,
            public_key TEXT NOT NULL,
            private_key TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        INSERT INTO
            config (key, value)
        VALUES
            ('enable_notifications', 'true'),
            ('enable_attachments', 'true')
        ON CONFLICT(key) DO NOTHING;

        CREATE TABLE IF NOT EXISTS contact (
            onion_id TEXT PRIMARY KEY,
            nickname TEXT NOT NULL,
            public_key TEXT NOT NULL,
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
            FOREIGN KEY 
                (contact_onion_id)
            REFERENCES
                contact(onion_id)
            ON DELETE CASCADE
        );
        "#,
    )?;

    tracing::debug!("Database connection established");

    Ok(conn)
}

// --- User ---

/// Represents row in user table.
#[non_exhaustive]
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
#[non_exhaustive]
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
    fn table() -> &'static str {
        "user"
    }

    fn primary_key(&self) -> PrimaryKey {
        PrimaryKey::Provided(&self.onion_id)
    }

    fn delete_by() -> &'static str {
        "onion_id"
    }

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
    fn pk_column() -> &'static str {
        "onion_id"
    }

    fn pk_value(&self) -> &dyn ToSql {
        &self.onion_id
    }

    fn update_values(&self) -> Vec<(&'static str, Option<&dyn ToSql>)> {
        vec![
            (
                "public_key",
                self.public_key.as_ref().map(|v| v as &dyn ToSql),
            ),
            (
                "private_key",
                self.private_key.as_ref().map(|v| v as &dyn ToSql),
            ),
        ]
    }
}

// --- Contact ---

/// Represents row in contact table.
#[non_exhaustive]
#[derive(serde::Serialize, Debug)]
pub struct ContactDb {
    /// Column onion_id.
    pub onion_id: String,

    /// Column nickname.
    pub nickname: String,

    /// Column public_key.
    pub public_key: String,

    /// Computed field showing timestamp of last message with this contact..
    pub last_message_at: i32,

    /// Column last_viewed_at.
    pub last_viewed_at: i32,

    /// Computed field containing amount of unread messages from this contact.
    pub amount_unread_messages: i64,
}

/// Type allowing to update a contact.
#[non_exhaustive]
#[derive(serde::Serialize)]
pub struct UpdateContactDb {
    /// PK of contact to update.
    pub onion_id: String,

    /// Optional update for nickname column.
    pub nickname: Option<String>,

    /// Optional update for public_key column.
    pub public_key: Option<String>,
}

impl ContactDb {
    /// Retrieve all contacts with unread count in ONE query.
    pub async fn retrieve_all(
        order_column: Option<&str>,
        desc: Option<bool>,
        conn: DatabaseConnection,
    ) -> Result<Vec<Self>, error::DatabaseError> {
        let conn = conn.lock().await;
        let order = order_column.unwrap_or("last_message_at");
        let direction = if desc.unwrap_or(true) { "DESC" } else { "ASC" };
        let sql = format!(
            r#"
            SELECT
                contact.onion_id,
                contact.nickname,
                contact.public_key,
                COALESCE(MAX(message.timestamp), 0) AS last_message_at,
                contact.last_viewed_at,
                COUNT(message.id) AS amount_unread_messages
            FROM
                contact
            LEFT JOIN
                message
            ON
                message.contact_onion_id = contact.onion_id
                AND message.is_incoming = 1
                AND message.timestamp > contact.last_viewed_at
            GROUP BY
                contact.onion_id
            ORDER BY
                {} {}
            "#,
            order, direction
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], Self::from_row)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}

impl DbModel for ContactDb {
    fn table() -> &'static str {
        "contact"
    }

    fn primary_key(&self) -> PrimaryKey {
        PrimaryKey::Provided(&self.onion_id)
    }

    fn delete_by() -> &'static str {
        "onion_id"
    }

    fn insert_values(&self) -> Vec<(&'static str, &dyn ToSql)> {
        vec![
            ("onion_id", &self.onion_id),
            ("nickname", &self.nickname),
            ("public_key", &self.public_key),
            ("last_viewed_at", &self.last_viewed_at),
        ]
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            onion_id: row.get("onion_id")?,
            nickname: row.get("nickname")?,
            public_key: row.get("public_key")?,
            last_message_at: row.get("last_message_at").unwrap_or(0),
            last_viewed_at: row.get("last_viewed_at")?,
            amount_unread_messages: row.get("amount_unread_messages").unwrap_or(0),
        })
    }
}

impl DbUpdateModel<ContactDb> for UpdateContactDb {
    fn pk_column() -> &'static str {
        "onion_id"
    }

    fn pk_value(&self) -> &dyn ToSql {
        &self.onion_id
    }

    fn update_values(&self) -> Vec<(&'static str, Option<&dyn ToSql>)> {
        vec![
            ("nickname", self.nickname.as_ref().map(|v| v as &dyn ToSql)),
            (
                "public_key",
                self.public_key.as_ref().map(|v| v as &dyn ToSql),
            ),
        ]
    }
}

// --- Message ---

/// Represents row in message table.
#[non_exhaustive]
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
#[non_exhaustive]
#[derive(serde::Serialize)]
pub struct UpdateMessageDb {
    /// PK of message to update.
    pub id: i64,

    /// Optional update for sent_status column.
    pub sent_status: Option<bool>,
}

impl DbModel for MessageDb {
    fn table() -> &'static str {
        "message"
    }

    fn primary_key(&self) -> PrimaryKey {
        PrimaryKey::AutoIncrement
    }

    fn delete_by() -> &'static str {
        "contact_onion_id"
    }

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
    fn pk_column() -> &'static str {
        "id"
    }

    fn pk_value(&self) -> &dyn ToSql {
        &self.id
    }

    fn update_values(&self) -> Vec<(&'static str, Option<&dyn ToSql>)> {
        vec![(
            "sent_status",
            self.sent_status.as_ref().map(|v| v as &dyn ToSql),
        )]
    }
}

impl MessageDb {
    /// Retrieve messages for chat.
    pub async fn retrieve_messages(
        onion_id: &str,
        offset: &Option<usize>,
        limit: &Option<usize>,
        conn: DatabaseConnection,
    ) -> Result<Vec<Self>, error::DatabaseError> {
        let conn = conn.lock().await;

        let ts = chrono::Utc::now().timestamp();
        let mut stmt = conn.prepare("UPDATE contact SET last_viewed_at = ? WHERE onion_id = ?")?;
        stmt.execute(params![ts, onion_id])?;

        let mut sql = "SELECT * FROM MESSAGE
             WHERE
                contact_onion_id = ?
             ORDER BY
                timestamp DESC"
            .to_string();

        if limit.is_some() {
            sql.push_str(" LIMIT ?");
        }
        if offset.is_some() {
            sql.push_str(" OFFSET ?");
        }

        let mut stmt = conn.prepare(&sql)?;

        let limit_i64;
        let offset_i64;

        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
        params.push(&onion_id);
        if let Some(limit) = limit {
            limit_i64 = *limit as i64;
            params.push(&limit_i64);
        }
        if let Some(offset) = offset {
            offset_i64 = *offset as i64;
            params.push(&offset_i64);
        }

        let rows = stmt.query_map(params_from_iter(params), |row| Self::from_row(row))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Retrieve failed chat message.
    pub async fn failed_messages(
        conn: DatabaseConnection,
    ) -> Result<Vec<Self>, error::DatabaseError> {
        let conn = conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT * FROM MESSAGE
             WHERE
                sent_status = 0
              AND
                is_incoming = 0
              AND
                timestamp <= strftime('%s','now') - 30
              AND
                timestamp >= strftime('%s','now') - 300
             ORDER BY
                timestamp DESC",
        )?;

        let rows = stmt.query_map([], |row| Self::from_row(row))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}

/// Type to get and set configuration.
#[non_exhaustive]
pub struct ConfigDb;

impl ConfigDb {
    /// Insert or update config value.
    pub async fn set(
        key: &str,
        value: &str,
        conn: DatabaseConnection,
    ) -> Result<(), error::DatabaseError> {
        let conn = conn.lock().await;
        conn.execute(
            r#"
                INSERT INTO
                   config
                VALUES
                    (?, ?)
                ON CONFLICT(key) DO UPDATE
                    SET value=excluded.value
            "#,
            params![key, value],
        )?;

        Ok(())
    }

    /// Get config value by key.
    pub async fn get(
        key: &str,
        conn: DatabaseConnection,
    ) -> Result<Option<String>, error::DatabaseError> {
        let conn = conn.lock().await;
        let result = conn.query_row("SELECT value FROM config WHERE key = ?1", [key], |row| {
            row.get::<_, String>(0)
        });

        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // --- Type specific getters and setters ---

    /// Set config value as bool.
    pub async fn set_bool(
        key: &str,
        value: bool,
        conn: DatabaseConnection,
    ) -> Result<(), error::DatabaseError> {
        Self::set(key, &value.to_string(), conn).await
    }
    /// Get config value as bool.
    pub async fn get_bool(
        key: &str,
        conn: DatabaseConnection,
    ) -> Result<bool, error::DatabaseError> {
        Ok(Self::get(key, conn).await?.as_deref() == Some("true"))
    }
}

/// Public trait implementing default methods (insert, retrieve, update) for Db types.
#[async_trait]
pub trait DbModel: Sized {
    /// Return table name.
    fn table() -> &'static str;

    /// Primary key can be known or is autoincrement.
    fn primary_key(&self) -> PrimaryKey;

    /// Column used to delete rows.
    fn delete_by() -> &'static str;

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
    async fn retrieve(
        onion_id: &str,
        conn: DatabaseConnection,
    ) -> Result<Self, error::DatabaseError> {
        let conn = conn.lock().await;
        let sql = format!("SELECT * FROM {} WHERE onion_id = ?", Self::table());

        Ok(conn.query_row(&sql, [onion_id], Self::from_row)?)
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
            sql = format!(
                "{} ORDER BY {} {}",
                sql,
                oc,
                if desc.unwrap_or(true) { "DESC" } else { "ASC" }
            );
        }

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| Self::from_row(row))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Default delete behavior.
    async fn delete(onion_id: &str, conn: DatabaseConnection) -> Result<(), error::DatabaseError> {
        let conn = conn.lock().await;
        let sql = format!(
            "DELETE FROM {} WHERE {} = ?",
            Self::table(),
            Self::delete_by()
        );
        let mut stmt = conn.prepare(&sql)?;
        stmt.execute(params![onion_id])?;

        Ok(())
    }

    /// Default delete all behavior.
    async fn delete_all(conn: DatabaseConnection) -> Result<(), error::DatabaseError> {
        let conn = conn.lock().await;
        let sql = format!("DELETE FROM {}", Self::table());
        conn.execute(&sql, [])?;

        Ok(())
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

/// Helper method to get path do .db file in project_dir.
fn database_path(project_dir: &std::path::Path) -> std::path::PathBuf {
    project_dir.join("arti-chat.db")
}

/// Store/retrieve key for database encryption in OS keychain.
fn retrieve_db_encryption_key() -> Result<String, error::DatabaseError> {
    let entry = keyring::Entry::new("com.arti-chat.desktop", "db-key")?;
    match entry.get_password() {
        Ok(key) => Ok(key),
        Err(_) => {
            // Generate new key.
            let mut bytes = [0_u8; 32];
            rand::rng().fill_bytes(&mut bytes);
            let key = hex::encode(bytes);

            entry.set_password(&key)?;
            Ok(key)
        }
    }
}
