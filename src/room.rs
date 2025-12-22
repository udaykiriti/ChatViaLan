//! Room management: broadcasting, history, and room switching.

use std::collections::VecDeque;
use crate::types::{Clients, Histories, HistoryItem, Outgoing, Tx};
use crate::client::{client_name_by_id, client_tx_by_id, now_ts};

/// Send a system message to all users in a room and record to history.
pub async fn send_system_to_room(clients: &Clients, histories: &Histories, room: &str, text: &str) {
    let msg = Outgoing::System { text: text.to_string() };
    let item = HistoryItem {
        from: "system".to_string(),
        text: text.to_string(),
        ts: now_ts(),
    };
    {
        let mut locked_h = histories.lock().await;
        let q = locked_h
            .entry(room.to_string())
            .or_insert_with(|| VecDeque::with_capacity(200));
        q.push_back(item);
        while q.len() > 200 {
            q.pop_front();
        }
    }
    let s = serde_json::to_string(&msg).unwrap_or_default();
    let locked = clients.lock().await;
    for c in locked.values() {
        if c.room == room {
            let _ = c.tx.send(warp::ws::Message::text(s.clone()));
        }
    }
}

/// Send room history to a single client.
pub async fn send_history_to_client_room(tx: &Tx, histories: &Histories, room: &str) {
    let items: Vec<HistoryItem> = {
        let locked = histories.lock().await;
        locked
            .get(room)
            .map(|q| q.iter().cloned().collect())
            .unwrap_or_default()
    };
    let msg = Outgoing::History { items };
    if let Ok(text) = serde_json::to_string(&msg) {
        let _ = tx.send(warp::ws::Message::text(text));
    }
}

/// Send user list to all users in a room.
pub async fn send_user_list_to_room(clients: &Clients, room: &str) {
    let names: Vec<String> = {
        let locked = clients.lock().await;
        locked
            .values()
            .filter(|c| c.room == room)
            .map(|c| c.name.clone())
            .collect()
    };
    let msg = Outgoing::List { users: names };
    let s = serde_json::to_string(&msg).unwrap_or_default();
    let locked = clients.lock().await;
    for c in locked.values() {
        if c.room == room {
            let _ = c.tx.send(warp::ws::Message::text(s.clone()));
        }
    }
}

/// Broadcast a message to all clients in a room and store in history.
pub async fn broadcast_to_room_and_store(
    clients: &Clients,
    histories: &Histories,
    room: &str,
    item: HistoryItem,
) {
    {
        let mut locked_h = histories.lock().await;
        let q = locked_h
            .entry(room.to_string())
            .or_insert_with(|| VecDeque::with_capacity(200));
        q.push_back(item.clone());
        while q.len() > 200 {
            q.pop_front();
        }
    }
    let outgoing = Outgoing::Msg {
        from: item.from.clone(),
        text: item.text.clone(),
        ts: item.ts,
    };
    if let Ok(s) = serde_json::to_string(&outgoing) {
        let locked = clients.lock().await;
        for c in locked.values() {
            if c.room == room {
                let _ = c.tx.send(warp::ws::Message::text(s.clone()));
            }
        }
    }
}

/// Move a client to a new room with announcements and history.
pub async fn join_room(client_id: &str, room: &str, clients: &Clients, histories: &Histories) {
    let target = room.trim();
    if target.is_empty() {
        return;
    }

    // Move client to new room and capture old room
    let old_room = {
        let mut locked = clients.lock().await;
        if let Some(c) = locked.get_mut(client_id) {
            let old = c.room.clone();
            c.room = target.to_string();
            old
        } else {
            return;
        }
    };

    // Ensure room exists in histories
    {
        let mut locked_h = histories.lock().await;
        locked_h
            .entry(target.to_string())
            .or_insert_with(|| VecDeque::with_capacity(200));
    }

    // Announce leave in old room
    let name = client_name_by_id(clients, client_id).await;
    send_system_to_room(clients, histories, &old_room, &format!("-- {} left the room --", name)).await;
    send_user_list_to_room(clients, &old_room).await;

    // Announce join in new room
    send_system_to_room(clients, histories, target, &format!("-- {} joined the room --", name)).await;
    if let Some(tx) = client_tx_by_id(clients, client_id).await {
        send_history_to_client_room(&tx, histories, target).await;
    }
    send_user_list_to_room(clients, target).await;

    // Notify the mover
    if let Some(tx) = client_tx_by_id(clients, client_id).await {
        let msg = Outgoing::System {
            text: format!("You joined room '{}'", target),
        };
        let _ = tx.send(warp::ws::Message::text(
            serde_json::to_string(&msg).unwrap(),
        ));
    }
}
