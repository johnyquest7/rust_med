We're going to start adding authentication to the app. We'll store auth information in a file named auth.json in the app data directory. 

The file will be structured like this:
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
  }
  "wrapped_dek": {
    "algorithm": "aes-256-gcm",
    "nonce": "base64-encoded-nonce",
    "ciphertext": "base64-encoded-ciphertext",
    "tag": "optional-base64-tag-if-separated"
  },
  "created_at": "2025-10-08T14:21:00Z",
  "last_password_change": "2025-10-08T14:21:00Z"
}


Workflow:

1. When the app opens, check if the auth.json file exists.
2. If auth.json doesn't exist: show the user a form for their username and password. Have them create those things and store it in this file.
3. Then authenticate the user.
4. If the file auth.json does exist, ask the user for their password (show their username), and then authenticate the password against the file.
5. On success: authenticate the user.
6. On failure, tell the user the issue. Like their password was wrong and let them try again.  

The plan for this is in plan.md. Update the plan in plan.md with additional that needs to happen, and then we'll work on it in pieces.
