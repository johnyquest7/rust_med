use rusqlite::{Connection, params};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};

/// Database error types
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    #[allow(dead_code)]
    Serialization(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

pub type DbResult<T> = Result<T, DbError>;

/// Initialize the database with schema
pub fn initialize_database(db_path: &PathBuf) -> DbResult<Connection> {
    let conn = Connection::open(db_path)?;

    // Create authentication table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS auth (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            version INTEGER NOT NULL,
            user_id TEXT NOT NULL,
            username TEXT NOT NULL,
            kdf_algorithm TEXT NOT NULL,
            kdf_salt TEXT NOT NULL,
            kdf_memory_kib INTEGER NOT NULL,
            kdf_iterations INTEGER NOT NULL,
            kdf_parallelism INTEGER NOT NULL,
            wrapped_dek_algorithm TEXT NOT NULL,
            wrapped_dek_nonce TEXT NOT NULL,
            wrapped_dek_ciphertext TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_password_change TEXT NOT NULL
        )",
        [],
    )?;

    // Create patient notes table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS patient_notes (
            id TEXT PRIMARY KEY,
            encrypted_data TEXT NOT NULL,
            nonce TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    // Create index on created_at for faster sorting
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notes_created_at ON patient_notes(created_at DESC)",
        [],
    )?;

    // Create setup status table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS setup_status (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            setup_completed INTEGER NOT NULL DEFAULT 0,
            completed_at TEXT
        )",
        [],
    )?;

    Ok(conn)
}

/// Authentication data structure for database
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub version: u32,
    pub user_id: String,
    pub username: String,
    pub kdf_algorithm: String,
    pub kdf_salt: String,
    pub kdf_memory_kib: u32,
    pub kdf_iterations: u32,
    pub kdf_parallelism: u32,
    pub wrapped_dek_algorithm: String,
    pub wrapped_dek_nonce: String,
    pub wrapped_dek_ciphertext: String,
    pub created_at: String,
    pub last_password_change: String,
}

/// Save authentication data to database
pub fn save_auth_data(conn: &Connection, auth_data: &AuthData) -> DbResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO auth (
            id, version, user_id, username,
            kdf_algorithm, kdf_salt, kdf_memory_kib, kdf_iterations, kdf_parallelism,
            wrapped_dek_algorithm, wrapped_dek_nonce, wrapped_dek_ciphertext,
            created_at, last_password_change
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            1, // id is always 1 (single user system)
            auth_data.version,
            auth_data.user_id,
            auth_data.username,
            auth_data.kdf_algorithm,
            auth_data.kdf_salt,
            auth_data.kdf_memory_kib,
            auth_data.kdf_iterations,
            auth_data.kdf_parallelism,
            auth_data.wrapped_dek_algorithm,
            auth_data.wrapped_dek_nonce,
            auth_data.wrapped_dek_ciphertext,
            auth_data.created_at,
            auth_data.last_password_change,
        ],
    )?;
    Ok(())
}

/// Load authentication data from database
pub fn load_auth_data(conn: &Connection) -> DbResult<AuthData> {
    let mut stmt = conn.prepare(
        "SELECT version, user_id, username,
                kdf_algorithm, kdf_salt, kdf_memory_kib, kdf_iterations, kdf_parallelism,
                wrapped_dek_algorithm, wrapped_dek_nonce, wrapped_dek_ciphertext,
                created_at, last_password_change
         FROM auth WHERE id = 1"
    )?;

    let auth_data = stmt.query_row([], |row| {
        Ok(AuthData {
            version: row.get(0)?,
            user_id: row.get(1)?,
            username: row.get(2)?,
            kdf_algorithm: row.get(3)?,
            kdf_salt: row.get(4)?,
            kdf_memory_kib: row.get(5)?,
            kdf_iterations: row.get(6)?,
            kdf_parallelism: row.get(7)?,
            wrapped_dek_algorithm: row.get(8)?,
            wrapped_dek_nonce: row.get(9)?,
            wrapped_dek_ciphertext: row.get(10)?,
            created_at: row.get(11)?,
            last_password_change: row.get(12)?,
        })
    }).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => DbError::NotFound("No auth data found".to_string()),
        _ => DbError::Sqlite(e),
    })?;

    Ok(auth_data)
}

/// Check if auth data exists
pub fn auth_data_exists(conn: &Connection) -> DbResult<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM auth WHERE id = 1")?;
    let count: i64 = stmt.query_row([], |row| row.get(0))?;
    Ok(count > 0)
}

/// Encrypted note structure for database
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedNoteData {
    pub id: String,
    pub encrypted_data: String,
    pub nonce: String,
    pub created_at: DateTime<Local>,
}

/// Save encrypted patient note to database
pub fn save_encrypted_note(conn: &Connection, note: &EncryptedNoteData) -> DbResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO patient_notes (id, encrypted_data, nonce, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            note.id,
            note.encrypted_data,
            note.nonce,
            note.created_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Load all encrypted patient notes from database
pub fn load_all_encrypted_notes(conn: &Connection) -> DbResult<Vec<EncryptedNoteData>> {
    let mut stmt = conn.prepare(
        "SELECT id, encrypted_data, nonce, created_at
         FROM patient_notes
         ORDER BY created_at DESC"
    )?;

    let notes = stmt.query_map([], |row| {
        let created_at_str: String = row.get(3)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
            .with_timezone(&Local);

        Ok(EncryptedNoteData {
            id: row.get(0)?,
            encrypted_data: row.get(1)?,
            nonce: row.get(2)?,
            created_at,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(notes)
}

/// Load a single encrypted patient note by ID
pub fn load_encrypted_note_by_id(conn: &Connection, note_id: &str) -> DbResult<EncryptedNoteData> {
    let mut stmt = conn.prepare(
        "SELECT id, encrypted_data, nonce, created_at
         FROM patient_notes
         WHERE id = ?1"
    )?;

    let note = stmt.query_row([note_id], |row| {
        let created_at_str: String = row.get(3)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
            .with_timezone(&Local);

        Ok(EncryptedNoteData {
            id: row.get(0)?,
            encrypted_data: row.get(1)?,
            nonce: row.get(2)?,
            created_at,
        })
    }).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Note not found: {}", note_id)),
        _ => DbError::Sqlite(e),
    })?;

    Ok(note)
}

/// Delete a patient note by ID
pub fn delete_note_by_id(conn: &Connection, note_id: &str) -> DbResult<bool> {
    let rows_affected = conn.execute(
        "DELETE FROM patient_notes WHERE id = ?1",
        [note_id],
    )?;
    Ok(rows_affected > 0)
}

/// Check if a note exists by ID
pub fn note_exists(conn: &Connection, note_id: &str) -> DbResult<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM patient_notes WHERE id = ?1")?;
    let count: i64 = stmt.query_row([note_id], |row| row.get(0))?;
    Ok(count > 0)
}

/// Check if initial setup is completed
pub fn is_setup_completed(conn: &Connection) -> DbResult<bool> {
    let mut stmt = conn.prepare("SELECT setup_completed FROM setup_status WHERE id = 1")?;
    let completed: i64 = stmt.query_row([], |row| row.get(0))
        .unwrap_or(0);
    Ok(completed == 1)
}

/// Mark setup as completed
pub fn mark_setup_completed(conn: &Connection) -> DbResult<()> {
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO setup_status (id, setup_completed, completed_at)
         VALUES (1, 1, ?1)",
        params![now],
    )?;
    Ok(())
}
