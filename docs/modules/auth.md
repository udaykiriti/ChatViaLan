# Module: auth.rs

**Role:** User registration, login, and credential persistence.

---

## Functions

### load_users

```rust
pub fn load_users() -> HashMap<String, String>
```

Synchronously reads `users.json` from disk and deserializes it into a `HashMap<username, bcrypt_hash>`. Called once at startup before the async runtime is fully engaged. Returns an empty map if the file is missing or malformed.

---

### save_users_async

```rust
pub async fn save_users_async(users: &Users)
```

Serializes the current users map to JSON and writes it to `users.json`. The bcrypt computation and file write are done inside `spawn_blocking` to avoid blocking the async executor.

---

### register_user

```rust
pub async fn register_user(username: &str, password: &str, users: &Users) -> Result<()>
```

Registers a new user.

1. Checks whether `username` already exists in `Users`. Returns an error if it does.
2. Hashes the password with `bcrypt::hash(password, DEFAULT_COST)`.
3. Inserts the username and hash into `Users`.
4. Calls `save_users_async` to persist the change.

Returns `Ok(())` on success, or an `anyhow::Error` describing the failure.

---

### verify_login

```rust
pub fn verify_login(username: &str, password: &str, users: &Users) -> bool
```

Looks up `username` in `Users`. If found, verifies `password` against the stored bcrypt hash using `bcrypt::verify`. Returns `false` if the username does not exist or the password does not match.

---

## Security Notes

- Passwords are never stored or logged in plaintext.
- bcrypt `DEFAULT_COST` (12) is used. This is intentionally slow to resist brute-force attacks.
- Usernames are treated as case-sensitive by this module; normalization is the caller's responsibility.
