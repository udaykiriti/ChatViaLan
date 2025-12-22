//! Helper functions for client operations.

use std::time::SystemTime;
use crate::types::{Clients, Tx};

/// Get client name by ID.
pub async fn client_name_by_id(clients: &Clients, id: &str) -> String {
    let locked = clients.read().await;
    locked.get(id).map(|c| c.name.clone()).unwrap_or_else(|| id.to_string())
}

/// Get client tx channel by ID.
pub async fn client_tx_by_id(clients: &Clients, id: &str) -> Option<Tx> {
    let locked = clients.read().await;
    locked.get(id).map(|c| c.tx.clone())
}

/// Get current Unix timestamp.
pub fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Make a username unique among currently connected clients.
pub async fn make_unique_name(clients: &Clients, desired: &str) -> String {
    let mut candidate = desired.to_string();
    let mut suffix = 1usize;
    loop {
        let collision = {
            let locked = clients.read().await;
            locked.values().any(|c| c.name.eq_ignore_ascii_case(&candidate))
        };
        if !collision {
            return candidate;
        }
        candidate = format!("{}-{}", desired, suffix);
        suffix += 1;
    }
}
