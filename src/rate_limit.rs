//! Rate limiting for message spam prevention.

use crate::types::{Clients, Outgoing};
use std::time::{Duration, Instant};
use tracing::warn;

/// Check rate limit: max 5 messages in 10 seconds.
/// Returns true if message is allowed, false if rate limited.
pub async fn check_rate_limit(clients: &Clients, client_id: &str) -> bool {
    if let Some(mut r) = clients.get_mut(client_id) {
        let client = r.value_mut();
        let now = Instant::now();
        let window = Duration::from_secs(10);

        // Remove old timestamps outside 10-second window
        client
            .last_message_times
            .retain(|t| now.duration_since(*t) < window);

        if client.last_message_times.len() >= 5 {
            // Rate limited - send warning to client
            warn!("Client {} is rate limited", client.name);
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
