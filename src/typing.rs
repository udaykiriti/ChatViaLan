//! Typing indicator functionality.

use crate::types::{Clients, Outgoing};

/// Set typing status for a client.
pub async fn set_typing_status(clients: &Clients, client_id: &str, is_typing: bool) {
    let mut locked = clients.write().await;
    if let Some(client) = locked.get_mut(client_id) {
        client.is_typing = is_typing;
    }
}

/// Broadcast who is typing in the room to all room members.
pub async fn broadcast_typing_status(clients: &Clients, client_id: &str) {
    let (room, typing_users) = {
        let locked = clients.read().await;
        let room = locked.get(client_id).map(|c| c.room.clone()).unwrap_or_default();
        let typing: Vec<String> = locked.values()
            .filter(|c| c.room == room && c.is_typing)
            .map(|c| c.name.clone())
            .collect();
        (room, typing)
    };
    
    let msg = Outgoing::Typing { users: typing_users };
    if let Ok(s) = serde_json::to_string(&msg) {
        let locked = clients.read().await;
        for c in locked.values() {
            if c.room == room {
                let _ = c.tx.send(warp::ws::Message::text(s.clone()));
            }
        }
    }
}
