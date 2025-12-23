//! Room management: broadcasting, history, and room switching.

use std::collections::{HashMap, VecDeque};
use std::fs;
use uuid::Uuid;
use tracing::{info, error};
use crate::types::{Clients, Histories, HistoryItem, Outgoing, Tx};
use crate::helpers::{client_name_by_id, client_tx_by_id, now_ts};

/// Generate unique message ID.
pub fn generate_msg_id() -> String {
    Uuid::new_v4().to_string()[..8].to_string()
}

/// Send a system message to all users in a room and record to history.
pub async fn send_system_to_room(clients: &Clients, histories: &Histories, room: &str, text: &str) {
    let msg = Outgoing::System { text: text.to_string() };
    let item = HistoryItem {
        id: generate_msg_id(),
        from: "system".to_string(),
        text: text.to_string(),
        ts: now_ts(),
        reactions: HashMap::new(),
        edited: false,
        deleted: false,
    };
    {
        let mut locked_h = histories.write().await;
        let q = locked_h
            .entry(room.to_string())
            .or_insert_with(|| VecDeque::with_capacity(200));
        q.push_back(item);
        while q.len() > 200 {
            q.pop_front();
        }
    }
    let s = serde_json::to_string(&msg).unwrap_or_default();
    for r in clients.iter() {
        let c = r.value();
        if c.room == room {
            let _ = c.tx.send(warp::ws::Message::text(s.clone()));
        }
    }
}

/// Send room history to a single client (filter deleted messages).
pub async fn send_history_to_client_room(tx: &Tx, histories: &Histories, room: &str) {
    let items: Vec<HistoryItem> = {
        let locked = histories.read().await;
        locked
            .get(room)
            .map(|q| q.iter().filter(|i| !i.deleted).cloned().collect())
            .unwrap_or_default()
    };
    let msg = Outgoing::History { items };
    if let Ok(text) = serde_json::to_string(&msg) {
        let _ = tx.send(warp::ws::Message::text(text));
    }
}

/// Send user list to all users in a room.
pub async fn send_user_list_to_room(clients: &Clients, room: &str) {
    let names: Vec<String> = clients
        .iter()
        .filter(|r| r.value().room == room)
        .map(|r| r.value().name.clone())
        .collect();
    
    let msg = Outgoing::List { users: names.clone() };
    let s = serde_json::to_string(&msg).unwrap_or_default();

    for r in clients.iter() {
        let c = r.value();
        if c.room == room {
            let _ = c.tx.send(warp::ws::Message::text(s.clone()));
        }
    }
    info!("Broadcast user list for room '{}': {} users", room, names.len());
}

/// Broadcast a message to all clients in a room and store in history.
/// Returns the message ID.
pub async fn broadcast_to_room_and_store(
    clients: &Clients,
    histories: &Histories,
    room: &str,
    item: HistoryItem,
) -> String {
    let msg_id = item.id.clone();
    
    // Check for @mentions
    let mentions = extract_mentions(&item.text);
    
    {
        let mut locked_h = histories.write().await;
        let q = locked_h
            .entry(room.to_string())
            .or_insert_with(|| VecDeque::with_capacity(200));
        q.push_back(item.clone());
        while q.len() > 200 {
            q.pop_front();
        }
    }
    let outgoing = Outgoing::Msg {
        id: item.id.clone(),
        from: item.from.clone(),
        text: item.text.clone(),
        ts: item.ts,
        reactions: item.reactions.clone(),
        edited: item.edited,
    };
    if let Ok(s) = serde_json::to_string(&outgoing) {
        for r in clients.iter() {
            let c = r.value();
            if c.room == room {
                let _ = c.tx.send(warp::ws::Message::text(s.clone()));
            }
        }
        
        // Send mention notifications to mentioned users
        for mentioned in &mentions {
            for r in clients.iter() {
                let c = r.value();
                if c.name.to_lowercase() == mentioned.to_lowercase() && c.room == room {
                    let mention_msg = Outgoing::Mention {
                        from: item.from.clone(),
                        text: item.text.clone(),
                        mentioned: mentioned.clone(),
                    };
                    if let Ok(m) = serde_json::to_string(&mention_msg) {
                        let _ = c.tx.send(warp::ws::Message::text(m));
                    }
                }
            }
        }
    }
    
    msg_id
}

/// Extract @mentions from text.
fn extract_mentions(text: &str) -> Vec<String> {
    let re = regex::Regex::new(r"@(\w+)").unwrap();
    re.captures_iter(text)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

/// Add or toggle a reaction on a message.
pub async fn add_reaction(
    clients: &Clients,
    histories: &Histories,
    room: &str,
    msg_id: &str,
    emoji: &str,
    user: &str,
) {
    let added = {
        let mut locked_h = histories.write().await;
        if let Some(q) = locked_h.get_mut(room) {
            if let Some(item) = q.iter_mut().find(|i| i.id == msg_id) {
                let users = item.reactions.entry(emoji.to_string()).or_insert_with(Vec::new);
                if users.contains(&user.to_string()) {
                    users.retain(|u| u != user);
                    false
                } else {
                    users.push(user.to_string());
                    true
                }
            } else {
                return;
            }
        } else {
            return;
        }
    };
    
    // Broadcast reaction update
    let msg = Outgoing::Reaction {
        msg_id: msg_id.to_string(),
        emoji: emoji.to_string(),
        user: user.to_string(),
        added,
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

/// Edit a message.
pub async fn edit_message(
    clients: &Clients,
    histories: &Histories,
    room: &str,
    msg_id: &str,
    new_text: &str,
    requester: &str,
) -> bool {
    let edited = {
        let mut locked_h = histories.write().await;
        if let Some(q) = locked_h.get_mut(room) {
            if let Some(item) = q.iter_mut().find(|i| i.id == msg_id && i.from == requester) {
                item.text = new_text.to_string();
                item.edited = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    };
    
    if edited {
        let msg = Outgoing::Edit {
            msg_id: msg_id.to_string(),
            new_text: new_text.to_string(),
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
    
    edited
}

/// Delete a message.
pub async fn delete_message(
    clients: &Clients,
    histories: &Histories,
    room: &str,
    msg_id: &str,
    requester: &str,
) -> bool {
    let deleted = {
        let mut locked_h = histories.write().await;
        if let Some(q) = locked_h.get_mut(room) {
            if let Some(item) = q.iter_mut().find(|i| i.id == msg_id && i.from == requester) {
                item.deleted = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    };
    
    if deleted {
        let msg = Outgoing::Delete {
            msg_id: msg_id.to_string(),
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
    
    deleted
}

/// Broadcast read receipt.
pub async fn broadcast_read_receipt(clients: &Clients, room: &str, user: &str, last_msg_id: &str) {
    let msg = Outgoing::ReadReceipt {
        user: user.to_string(),
        last_msg_id: last_msg_id.to_string(),
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

/// Move a client to a new room with announcements and history.
pub async fn join_room(client_id: &str, room: &str, clients: &Clients, histories: &Histories) {
    let target = room.trim();
    if target.is_empty() {
        return;
    }

    // Move client to new room and capture old room
    let old_room = {
        if let Some(mut c) = clients.get_mut(client_id) {
            let old = c.room.clone();
            c.room = target.to_string();
            old
        } else {
            return;
        }
    };

    // Ensure room exists in histories
    {
        let mut locked_h = histories.write().await;
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
    info!("Client {} joined room '{}'", name, target);
}

/// Broadcast a status update (active/idle) to ALL connected clients.
pub async fn broadcast_status(clients: &Clients, user: &str, status: &str) {
    let msg = Outgoing::Status {
        user: user.to_string(),
        status: status.to_string(),
    };
    if let Ok(s) = serde_json::to_string(&msg) {
        for r in clients.iter() {
            let _ = r.value().tx.send(warp::ws::Message::text(s.clone()));
        }
    }
}

pub async fn save_history(histories: &Histories) {
    let h = histories.read().await;
    match serde_json::to_string(&*h) {
        Ok(json) => {
            if let Err(e) = fs::write("history.json", json) {
                error!("Failed to save history.json: {}", e);
            } else {
                info!("History saved to history.json");
            }
        }
        Err(e) => error!("Failed to serialize history: {}", e),
    }
}

pub async fn load_history(histories: &Histories) {
    if let Ok(json) = fs::read_to_string("history.json") {
        match serde_json::from_str::<HashMap<String, VecDeque<HistoryItem>>>(&json) {
            Ok(loaded) => {
                let mut h = histories.write().await;
                *h = loaded;
                info!("Loaded chat history from history.json");
            }
            Err(e) => error!("Failed to parse history.json: {}", e),
        }
    }
}
