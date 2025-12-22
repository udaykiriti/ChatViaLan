//! Rate limiting for message spam prevention.

use std::time::{Duration, Instant};
use crate::types::{Clients, Outgoing};

/// Check rate limit: max 5 messages in 10 seconds.
/// Returns true if message is allowed, false if rate limited.
pub async fn check_rate_limit(clients: &Clients, client_id: &str) -> bool {
    let mut locked = clients.lock().await;
    if let Some(client) = locked.get_mut(client_id) {
        let now = Instant::now();
        let window = Duration::from_secs(10);
        
        // Remove old timestamps outside 10-second window
        client.last_message_times.retain(|t| now.duration_since(*t) < window);
        
        if client.last_message_times.len() >= 5 {
            // Rate limited - send warning to client
            let msg = Outgoing::System {
                text: "Rate limited: slow down! Max 5 messages per 10 seconds.".to_string(),
            };
            if let Ok(s) = serde_json::to_string(&msg) {
                let _ = client.tx.send(warp::ws::Message::text(s));
            }
            return false;
        }
        
        client.last_message_times.push(now);
    }
    true
}
