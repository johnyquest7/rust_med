# Authentication Implementation Plan

We're going to start adding authentication to the app. We'll store auth information in a file named auth.json in the app data directory. 

## Auth File Structure

The auth.json file will be structured like this:
```json
{
  "version": 1,
  "user_id": "alice",
  "kdf": {
    "algorithm": "argon2id",
    "salt": "base64-encoded-salt",
    "params": {
      "memory_kib": 65536,
      "iterations": 3,
      "parallelism": 2
    }
  },
  "user": {
    "username": "aled1027"
  },
  "wrapped_dek": {
    "algorithm": "aes-256-gcm",
    "nonce": "base64-encoded-nonce",
    "ciphertext": "base64-encoded-ciphertext",
    "tag": "optional-base64-tag-if-separated"
  },
  "created_at": "2025-10-08T14:21:00Z",
  "last_password_change": "2025-10-08T14:21:00Z"
}
```

## Authentication Workflow

1. When the app opens, check if the auth.json file exists.
2. If auth.json doesn't exist: show the user a form for their username and password. Have them create those things and store it in this file.
3. Then authenticate the user.
4. If the file auth.json does exist, ask the user for their password (show their username), and then authenticate the password against the file.
5. On success: authenticate the user.
6. On failure, tell the user the issue. Like their password was wrong and let them try again.

## Implementation Steps

### Phase 1: Backend Authentication (Rust/Tauri)

#### 1.1 Add Required Dependencies
- Add `argon2` crate for password hashing
- Add `aes-gcm` crate for encryption
- Add `base64` crate for encoding/decoding
- Add `rand` crate for generating salts and nonces
- Add `uuid` crate for generating user IDs

#### 1.2 Create Authentication Data Structures
- Define `AuthFile` struct matching the JSON schema
- Define `KdfParams` struct for Argon2 parameters
- Define `WrappedDek` struct for encrypted data encryption key
- Define `User` struct for user information
- Define result types for authentication operations

#### 1.3 Implement Core Authentication Functions
- `check_auth_file_exists()` - Check if auth.json exists
- `load_auth_file()` - Load and parse auth.json
- `save_auth_file()` - Save auth.json to disk
- `generate_user_id()` - Generate unique user ID
- `generate_salt()` - Generate random salt for Argon2
- `generate_nonce()` - Generate random nonce for AES-GCM
- `derive_key_from_password()` - Use Argon2 to derive key from password
- `encrypt_dek()` - Encrypt data encryption key with derived key
- `decrypt_dek()` - Decrypt data encryption key with derived key

#### 1.4 Implement Tauri Commands
- `check_auth_status()` - Check if user is authenticated
- `create_user_account()` - Create new user account with password
- `authenticate_user()` - Authenticate existing user with password
- `get_user_info()` - Get current user information
- `change_password()` - Change user password (future feature)

### Phase 2: Frontend Authentication (Svelte)

#### 2.1 Update Authentication Types
- Add new types for authentication responses
- Update existing types to match backend structures
- Add types for registration vs login flows

#### 2.2 Update Authentication Context
- Replace mock authentication with real Tauri command calls
- Implement proper error handling for authentication failures
- Add support for both registration and login flows
- Handle authentication state persistence

#### 2.3 Create Registration Component
- Create user registration form component
- Validate username and password requirements
- Handle registration errors and success states
- Integrate with authentication context

#### 2.4 Update Login Component
- Update existing login form to handle both flows
- Show username when auth.json exists
- Handle authentication errors properly
- Add loading states and error messages

#### 2.5 Update App Layout
- Add authentication guard to protect routes
- Show appropriate forms based on auth status
- Handle authentication state changes
- Add logout functionality

### Phase 3: Integration and Testing

#### 3.1 App Startup Flow
- Implement app initialization with auth check
- Show registration form if no auth.json exists
- Show login form if auth.json exists
- Redirect to main app after successful authentication

#### 3.2 Error Handling
- Handle file system errors gracefully
- Handle authentication failures with clear messages
- Handle network/IO errors during auth operations
- Add proper logging for debugging

#### 3.3 Security Considerations
- Ensure passwords are never stored in plain text
- Use secure random number generation
- Implement proper key derivation
- Add rate limiting for failed login attempts (future)

#### 3.4 Testing
- Test registration flow with new users
- Test login flow with existing users
- Test error cases (wrong password, corrupted files)
- Test app startup in various states

## File Locations

### Backend (Rust)
- `src-tauri/src/auth.rs` - Authentication logic and data structures
- `src-tauri/src/commands.rs` - Tauri command implementations
- `src-tauri/Cargo.toml` - Updated dependencies

### Frontend (Svelte)
- `frontend/src/lib/types.ts` - Updated type definitions
- `frontend/src/lib/hooks/auth-context.svelte.ts` - Updated auth context
- `frontend/src/lib/components/custom/register-form.svelte` - New registration component
- `frontend/src/lib/components/custom/login-form.svelte` - Updated login component
- `frontend/src/routes/+layout.svelte` - Updated app layout with auth guard

## Security Notes

- Passwords are hashed using Argon2id (memory-hard function)
- Data encryption keys are encrypted with AES-256-GCM
- All cryptographic operations use secure random number generation
- Auth file is stored in app data directory (OS-specific secure location)
- No sensitive data is stored in frontend localStorage
