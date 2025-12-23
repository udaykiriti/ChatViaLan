//! User authentication: registration, login, and secure bcrypt hashing.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use bcrypt::{hash, verify, DEFAULT_COST};
use tracing::{info, error};

use crate::types::Users;

/// Load users from `users.json` (synchronous, done at startup).
pub fn load_users() -> anyhow::Result<HashMap<String, String>> {
    let path = "users.json";
    if Path::new(path).exists() {
        let s = fs::read_to_string(path)?;
        let m: HashMap<String, String> = serde_json::from_str(&s)?;
        info!("Loaded {} users from disk", m.len());
        Ok(m)
    } else {
        Ok(HashMap::new())
    }
}

/// Save users map to disk (async-friendly via spawn_blocking).
pub async fn save_users_async(map: HashMap<String, String>) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let s = serde_json::to_string_pretty(&map)?;
        fs::write("users.json", s)?;
        Ok(())
    })
    .await??;
    Ok(())
}

/// Register a new user. Returns Err on duplicate or save error.
pub async fn register_user(users: &Users, username: &str, password: &str) -> Result<(), String> {
    if users.contains_key(username) {
        return Err("username already exists".into());
    }
    
    // Hash password with bcrypt
    let hashed = hash(password, DEFAULT_COST).map_err(|e| format!("hash error: {}", e))?;
    
    users.insert(username.to_string(), hashed);
    
    // Create a copy for saving (synchronous collection)
    let map_to_save: HashMap<String, String> = users.iter()
        .map(|r| (r.key().clone(), r.value().clone()))
        .collect();
    
    if let Err(e) = save_users_async(map_to_save).await {
        error!("failed to save users: {}", e);
        return Err(format!("failed to save users: {}", e));
    }
    
    info!("Registered new user: {}", username);
    Ok(())
}

/// Verify login credentials.
pub async fn verify_login(users: &Users, username: &str, password: &str) -> bool {
    if let Some(r) = users.get(username) {
        let stored_hash = r.value();
        match verify(password, stored_hash) {
            Ok(valid) => valid,
            Err(e) => {
                error!("bcrypt verify error for {}: {}", username, e);
                false
            }
        }
    } else {
        false
    }
}
