# Encryption Test Plan

## Overview
This document outlines how to test the note encryption functionality that has been implemented.

## What Was Implemented

### Backend Changes (Rust/Tauri)
1. **New Encryption Functions in `auth.rs`:**
   - `get_dek()` - Gets the decrypted DEK using password
   - `encrypt_data()` - Encrypts data using DEK
   - `decrypt_data()` - Decrypts data using DEK

2. **New EncryptedNote Structure:**
   - Stores encrypted transcript and medical note
   - Includes separate nonces for each encrypted field
   - Preserves metadata (id, names, dates, etc.)

3. **Updated Note Functions:**
   - `create_patient_note()` - Now encrypts notes before saving
   - `update_patient_note()` - Now encrypts notes before saving
   - `load_patient_notes()` - Now decrypts notes when loading
   - `update_patient_note()` - Now encrypts entire note JSON before saving
   - All functions now require password parameter

### Frontend Changes (Svelte/TypeScript)
1. **Updated AuthContext:**
   - Stores password temporarily in memory during session
   - Clears password on logout for security
   - Added `getPassword()` method

2. **Updated TauriService:**
   - All note operations now pass password from auth context
   - Returns appropriate errors if password not available

## Testing Steps

### 1. Create a New User Account
1. Launch the application
2. Register a new user account with username and password
3. Verify authentication succeeds

### 2. Create Encrypted Notes
1. After login, create a new medical note
2. Verify the note is saved successfully
3. Check the notes directory - files should contain encrypted data (not plain text)

### 3. Load and View Notes
1. Navigate to the notes page
2. Verify notes load and display correctly
3. Verify transcript and medical note content is readable (decrypted)

### 4. Update Notes
1. Edit an existing note
2. Save the changes
3. Verify the updated note displays correctly

### 5. Legacy Note Migration
1. If there are existing unencrypted notes, they should be automatically migrated to encrypted format when loaded
2. Check console logs for migration messages

### 6. Security Verification
1. Check that note files on disk contain encrypted data (not readable plain text)
2. Verify that logout clears the password from memory
3. Verify that notes cannot be loaded without proper authentication

## Expected Behavior

### File Structure
- Notes are stored as JSON files in the app data directory
- Each note file contains encrypted transcript and medical note fields
- Metadata (names, dates, etc.) remains unencrypted for indexing/searching

### Security
- Sensitive data (transcript, medical note) is encrypted using AES-256-GCM
- Each encryption uses a unique nonce
- DEK is derived from user password using Argon2id
- Password is only stored temporarily in memory during session

### Error Handling
- Appropriate error messages if password is not available
- Graceful handling of decryption failures
- Automatic migration of legacy unencrypted notes

## Verification Commands

To verify encryption is working:

1. **Check note files contain encrypted data:**
   ```bash
   # Navigate to app data directory
   cd ~/Library/Application\ Support/com.tauri.dev/OnDevice_Medical_Note/notes/
   
   # View a note file - should contain encrypted strings
   cat *.json
   ```

2. **Verify application functionality:**
   - Create, read, update notes through the UI
   - Verify all operations work correctly
   - Check that sensitive data is not visible in plain text on disk

## Success Criteria
- [ ] Notes are encrypted when saved to disk
- [ ] Notes are decrypted when loaded from disk
- [ ] All note operations work correctly through the UI
- [ ] Legacy notes are automatically migrated
- [ ] Password is cleared from memory on logout
- [ ] Appropriate error handling for missing password
