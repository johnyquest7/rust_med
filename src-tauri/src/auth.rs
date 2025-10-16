use aes_gcm::aead::{generic_array::GenericArray, Aead};
use aes_gcm::{Aes256Gcm, KeyInit};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use rand::Rng;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Authentication file structure matching the JSON schema
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthFile {
    pub version: u32,
    pub user_id: String,
    pub kdf: KdfParams,
    pub user: User,
    pub wrapped_dek: WrappedDek,
    pub created_at: String,
    pub last_password_change: String,
}

/// Key Derivation Function parameters for Argon2
#[derive(Debug, Serialize, Deserialize)]
pub struct KdfParams {
    pub algorithm: String,
    pub salt: String,
    pub params: KdfAlgorithmParams,
}

/// Argon2 algorithm parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct KdfAlgorithmParams {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

/// User information
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
}

/// Wrapped Data Encryption Key
#[derive(Debug, Serialize, Deserialize)]
pub struct WrappedDek {
    pub algorithm: String,
    pub nonce: String,
    pub ciphertext: String,
    pub tag: Option<String>,
}

/// Result types for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;

/// Authentication error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Authentication failed: {0}")]
    #[allow(dead_code)]
    Authentication(String),

    #[error("Cryptographic error: {0}")]
    Cryptographic(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Authentication state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthState {
    NotAuthenticated,
    Authenticated { user_id: String, username: String },
}

/// Authentication request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateRequest {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub username: String,
}

/// Default Argon2 parameters
impl Default for KdfAlgorithmParams {
    fn default() -> Self {
        Self {
            memory_kib: 65536,
            iterations: 3,
            parallelism: 2,
        }
    }
}

/// Generate a new user ID
pub fn generate_user_id() -> String {
    Uuid::new_v4().to_string()
}

/// Get the auth file path in the app data directory
#[allow(dead_code)]
pub fn get_auth_file_path() -> PathBuf {
    // This will be implemented to use Tauri's app data directory
    // For now, we'll use a placeholder that will be updated when we integrate with Tauri
    PathBuf::from("auth.json")
}

/// Save the auth file to disk (deprecated - use save_auth_to_db instead)
#[allow(dead_code)]
pub fn save_auth_file(auth_path: &PathBuf, auth_file: &AuthFile) -> AuthResult<()> {
    let content = serde_json::to_string_pretty(auth_file)
        .map_err(|e| AuthError::Serialization(format!("Failed to serialize auth file: {}", e)))?;

    fs::write(auth_path, content)
        .map_err(|e| AuthError::FileSystem(format!("Failed to write auth file: {}", e)))?;

    Ok(())
}

/// Generate a random salt for Argon2
pub fn generate_salt() -> AuthResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(salt.to_string())
}

/// Generate a random nonce for AES-GCM
pub fn generate_nonce() -> AuthResult<String> {
    let mut nonce_bytes = [0u8; 12]; // 96-bit nonce for AES-GCM
    rand::thread_rng().fill(&mut nonce_bytes);
    Ok(general_purpose::STANDARD.encode(nonce_bytes))
}

/// Derive a key from password using Argon2id
pub fn derive_key_from_password(password: &str, salt: &str) -> AuthResult<Vec<u8>> {
    let salt_string = SaltString::from_b64(salt)
        .map_err(|e| AuthError::Cryptographic(format!("Invalid salt: {}", e)))?;

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| AuthError::Cryptographic(format!("Failed to hash password: {}", e)))?;

    // Extract the hash bytes (first 32 bytes for AES-256)
    let hash = password_hash.hash.unwrap();
    let hash_bytes = hash.as_bytes();

    // Ensure we have at least 32 bytes, pad with zeros if necessary
    let mut key = vec![0u8; 32];
    let copy_len = std::cmp::min(32, hash_bytes.len());
    key[..copy_len].copy_from_slice(&hash_bytes[..copy_len]);

    Ok(key)
}

/// Encrypt data encryption key with derived key
pub fn encrypt_dek(dek: &[u8], key: &[u8], nonce: &str) -> AuthResult<(String, String)> {
    let key_array: GenericArray<u8, _> = GenericArray::from_slice(key).clone();
    let cipher = Aes256Gcm::new(&key_array);

    let nonce_bytes = general_purpose::STANDARD
        .decode(nonce)
        .map_err(|e| AuthError::Cryptographic(format!("Invalid nonce: {}", e)))?;
    let nonce_array: GenericArray<u8, _> = GenericArray::from_slice(&nonce_bytes).clone();

    let ciphertext = cipher
        .encrypt(&nonce_array, dek)
        .map_err(|e| AuthError::Cryptographic(format!("Failed to encrypt DEK: {}", e)))?;

    let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);
    Ok((ciphertext_b64, String::new())) // AES-GCM includes tag in ciphertext
}

/// Decrypt data encryption key with derived key
pub fn decrypt_dek(ciphertext: &str, key: &[u8], nonce: &str) -> AuthResult<Vec<u8>> {
    let key_array: GenericArray<u8, _> = GenericArray::from_slice(key).clone();
    let cipher = Aes256Gcm::new(&key_array);

    let nonce_bytes = general_purpose::STANDARD
        .decode(nonce)
        .map_err(|e| AuthError::Cryptographic(format!("Invalid nonce: {}", e)))?;
    let nonce_array: GenericArray<u8, _> = GenericArray::from_slice(&nonce_bytes).clone();

    let ciphertext_bytes = general_purpose::STANDARD
        .decode(ciphertext)
        .map_err(|e| AuthError::Cryptographic(format!("Invalid ciphertext: {}", e)))?;

    let dek = cipher
        .decrypt(&nonce_array, ciphertext_bytes.as_ref())
        .map_err(|e| AuthError::Cryptographic(format!("Failed to decrypt DEK: {}", e)))?;

    Ok(dek)
}

/// Verify password against stored hash
#[allow(dead_code)]
pub fn verify_password(password: &str, stored_hash: &str) -> AuthResult<bool> {
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| AuthError::Cryptographic(format!("Invalid stored hash: {}", e)))?;

    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Create a new user account
pub fn create_user_account(username: String, password: String) -> AuthResult<AuthFile> {
    // Validate input
    if username.trim().is_empty() {
        return Err(AuthError::InvalidInput(
            "Username cannot be empty".to_string(),
        ));
    }
    if password.len() < 8 {
        return Err(AuthError::InvalidInput(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    // Generate user ID
    let user_id = generate_user_id();

    // Generate salt and nonce
    let salt = generate_salt()?;
    let nonce = generate_nonce()?;

    // Derive key from password
    let derived_key = derive_key_from_password(&password, &salt)?;

    // Generate a random data encryption key (DEK)
    let mut dek = [0u8; 32];
    rand::thread_rng().fill(&mut dek);

    // Encrypt the DEK
    let (encrypted_dek, _) = encrypt_dek(&dek, &derived_key, &nonce)?;

    // Create auth file
    let now = Utc::now().to_rfc3339();
    let auth_file = AuthFile {
        version: 1,
        user_id,
        kdf: KdfParams {
            algorithm: "argon2id".to_string(),
            salt,
            params: KdfAlgorithmParams::default(),
        },
        user: User { username },
        wrapped_dek: WrappedDek {
            algorithm: "aes-256-gcm".to_string(),
            nonce,
            ciphertext: encrypted_dek,
            tag: None,
        },
        created_at: now.clone(),
        last_password_change: now,
    };

    Ok(auth_file)
}

/// Authenticate user with password
pub fn authenticate_user(auth_file: &AuthFile, password: &str) -> AuthResult<bool> {
    // Derive key from password using stored salt
    let derived_key = derive_key_from_password(password, &auth_file.kdf.salt)?;

    // Try to decrypt the DEK
    match decrypt_dek(
        &auth_file.wrapped_dek.ciphertext,
        &derived_key,
        &auth_file.wrapped_dek.nonce,
    ) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Get the decrypted DEK for authenticated user
pub fn get_dek(auth_file: &AuthFile, password: &str) -> AuthResult<Vec<u8>> {
    // Derive key from password using stored salt
    let derived_key = derive_key_from_password(password, &auth_file.kdf.salt)?;

    // Decrypt the DEK
    decrypt_dek(
        &auth_file.wrapped_dek.ciphertext,
        &derived_key,
        &auth_file.wrapped_dek.nonce,
    )
}

/// Encrypt data using the DEK
pub fn encrypt_data(data: &str, dek: &[u8]) -> AuthResult<(String, String)> {
    let key_array: GenericArray<u8, _> = GenericArray::from_slice(dek).clone();
    let cipher = Aes256Gcm::new(&key_array);

    // Generate a new nonce for this encryption
    let nonce = generate_nonce()?;
    let nonce_bytes = general_purpose::STANDARD
        .decode(&nonce)
        .map_err(|e| AuthError::Cryptographic(format!("Invalid nonce: {}", e)))?;
    let nonce_array: GenericArray<u8, _> = GenericArray::from_slice(&nonce_bytes).clone();

    let ciphertext = cipher
        .encrypt(&nonce_array, data.as_bytes())
        .map_err(|e| AuthError::Cryptographic(format!("Failed to encrypt data: {}", e)))?;

    let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);
    Ok((ciphertext_b64, nonce))
}

/// Decrypt data using the DEK
pub fn decrypt_data(ciphertext: &str, dek: &[u8], nonce: &str) -> AuthResult<String> {
    let key_array: GenericArray<u8, _> = GenericArray::from_slice(dek).clone();
    let cipher = Aes256Gcm::new(&key_array);

    let nonce_bytes = general_purpose::STANDARD
        .decode(nonce)
        .map_err(|e| AuthError::Cryptographic(format!("Invalid nonce: {}", e)))?;
    let nonce_array: GenericArray<u8, _> = GenericArray::from_slice(&nonce_bytes).clone();

    let ciphertext_bytes = general_purpose::STANDARD
        .decode(ciphertext)
        .map_err(|e| AuthError::Cryptographic(format!("Invalid ciphertext: {}", e)))?;

    let plaintext = cipher
        .decrypt(&nonce_array, ciphertext_bytes.as_ref())
        .map_err(|e| AuthError::Cryptographic(format!("Failed to decrypt data: {}", e)))?;

    String::from_utf8(plaintext)
        .map_err(|e| AuthError::Cryptographic(format!("Invalid UTF-8 in decrypted data: {}", e)))
}

// Database-compatible functions

/// Convert AuthFile to database-compatible AuthData
pub fn auth_file_to_db_data(auth_file: &AuthFile) -> crate::db::AuthData {
    crate::db::AuthData {
        version: auth_file.version,
        user_id: auth_file.user_id.clone(),
        username: auth_file.user.username.clone(),
        kdf_algorithm: auth_file.kdf.algorithm.clone(),
        kdf_salt: auth_file.kdf.salt.clone(),
        kdf_memory_kib: auth_file.kdf.params.memory_kib,
        kdf_iterations: auth_file.kdf.params.iterations,
        kdf_parallelism: auth_file.kdf.params.parallelism,
        wrapped_dek_algorithm: auth_file.wrapped_dek.algorithm.clone(),
        wrapped_dek_nonce: auth_file.wrapped_dek.nonce.clone(),
        wrapped_dek_ciphertext: auth_file.wrapped_dek.ciphertext.clone(),
        created_at: auth_file.created_at.clone(),
        last_password_change: auth_file.last_password_change.clone(),
    }
}

/// Convert database AuthData to AuthFile
pub fn db_data_to_auth_file(auth_data: &crate::db::AuthData) -> AuthFile {
    AuthFile {
        version: auth_data.version,
        user_id: auth_data.user_id.clone(),
        kdf: KdfParams {
            algorithm: auth_data.kdf_algorithm.clone(),
            salt: auth_data.kdf_salt.clone(),
            params: KdfAlgorithmParams {
                memory_kib: auth_data.kdf_memory_kib,
                iterations: auth_data.kdf_iterations,
                parallelism: auth_data.kdf_parallelism,
            },
        },
        user: User {
            username: auth_data.username.clone(),
        },
        wrapped_dek: WrappedDek {
            algorithm: auth_data.wrapped_dek_algorithm.clone(),
            nonce: auth_data.wrapped_dek_nonce.clone(),
            ciphertext: auth_data.wrapped_dek_ciphertext.clone(),
            tag: None,
        },
        created_at: auth_data.created_at.clone(),
        last_password_change: auth_data.last_password_change.clone(),
    }
}

/// Save auth file to database
pub fn save_auth_to_db(conn: &Connection, auth_file: &AuthFile) -> AuthResult<()> {
    let auth_data = auth_file_to_db_data(auth_file);
    crate::db::save_auth_data(conn, &auth_data)
        .map_err(|e| AuthError::FileSystem(format!("Failed to save auth data to database: {}", e)))
}

/// Load auth file from database
pub fn load_auth_from_db(conn: &Connection) -> AuthResult<AuthFile> {
    let auth_data = crate::db::load_auth_data(conn).map_err(|e| {
        AuthError::FileSystem(format!("Failed to load auth data from database: {}", e))
    })?;
    Ok(db_data_to_auth_file(&auth_data))
}

/// Check if auth exists in database
pub fn check_auth_exists_in_db(conn: &Connection) -> bool {
    crate::db::auth_data_exists(conn).unwrap_or(false)
}
