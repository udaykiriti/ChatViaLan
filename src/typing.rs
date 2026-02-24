//! Typing indicator functionality.

use crate::types::{Clients, Outgoing};

/// Set typing status for a client.
pub async fn set_typing_status(clients: &Clients, client_id: &str, is_typing: bool) {
    if let Some(mut r) = clients.get_mut(client_id) {
        r.value_mut().is_typing = is_typing;
    }
}

/// Broadcast who is typing in the room to all room members.
pub async fn broadcast_typing_status(clients: &Clients, client_id: &str) {
    let (room, typing_users) = {
        let room = clients
            .get(client_id)
            .map(|r| r.value().room.clone())
            .unwrap_or_default();
        let typing: Vec<String> = clients
            .iter()
            .filter(|r| r.value().room == room && r.value().is_typing)
            .map(|r| r.value().name.clone())
            .collect();
        (room, typing)
    };

    let msg = Outgoing::Typing {
        users: typing_users,
    };
    if let Ok(s) = serde_json::to_string(&msg) {
        for r in clients.iter() {
            let c = r.value();
            if c.room == room {
                let _ = c.tx.send(warp::ws::Message::text(s.clone()));
            }
        }
    }
}
