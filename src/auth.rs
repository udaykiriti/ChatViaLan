//! User authentication: registration, login, and password hashing.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use sha2::{Digest, Sha256};

use crate::types::Users;

/// Load users from `users.json` (synchronous, done at startup).
pub fn load_users() -> anyhow::Result<HashMap<String, String>> {
    let path = "users.json";
    if Path::new(path).exists() {
        let s = fs::read_to_string(path)?;
        let m: HashMap<String, String> = serde_json::from_str(&s)?;
        Ok(m)
    } else {
        Ok(HashMap::new())
    }
}

/// Save users map to disk (async-friendly via spawn_blocking).
pub async fn save_users_async(map: &HashMap<String, String>) -> anyhow::Result<()> {
    let m = map.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let s = serde_json::to_string_pretty(&m)?;
        fs::write("users.json", s)?;
        Ok(())
    })
    .await??;
    Ok(())
}

/// Hash password with username salt (SHA256). Not for production.
pub fn hash_password(username: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(username.as_bytes());
    hasher.update(b":");
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Register a new user. Returns Err on duplicate or save error.
pub async fn register_user(users: &Users, username: &str, password: &str) -> Result<(), String> {
    let mut locked = users.lock().await;
    if locked.contains_key(username) {
        return Err("username already exists".into());
    }
    let hash = hash_password(username, password);
    locked.insert(username.to_string(), hash);
    if let Err(e) = save_users_async(&*locked).await {
        return Err(format!("failed to save users: {}", e));
    }
    Ok(())
}

/// Verify login credentials.
pub async fn verify_login(users: &Users, username: &str, password: &str) -> bool {
    let locked = users.lock().await;
    if let Some(stored) = locked.get(username) {
        let h = hash_password(username, password);
        return &h == stored;
    }
    false
}
