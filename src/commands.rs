//! Command handling for chat commands.

use std::collections::{HashMap, VecDeque};
use crate::types::{Clients, Histories, HistoryItem, Outgoing, Users};
use crate::auth::{register_user, verify_login};
use crate::room::{send_system_to_room, send_user_list_to_room, send_history_to_client_room, broadcast_to_room_and_store, join_room, generate_msg_id};
use crate::helpers::{client_name_by_id, client_tx_by_id, make_unique_name, now_ts};

/// Handle all `/` commands from a connected client.
pub async fn handle_cmd_with_rooms(
    client_id: &str,
    cmd_line: &str,
    clients: &Clients,
    histories: &Histories,
    users: &Users,
) {
    let mut parts = cmd_line.splitn(3, ' ');
    let cmd = parts.next().unwrap_or("");
    
    match cmd {
        "/join" => {
            if let Some(room) = parts.next() {
                join_room(client_id, room.trim(), clients, histories).await;
            } else {
                send_to_client(clients, client_id, "Usage: /join <room>").await;
            }
        }
        "/rooms" => {
            let rooms: Vec<String> = {
                let locked_h = histories.lock().await;
                locked_h.keys().cloned().collect()
            };
            send_to_client(clients, client_id, &format!("Rooms: {}", rooms.join(", "))).await;
        }
        "/leave" => {
            join_room(client_id, "lobby", clients, histories).await;
        }
        "/room" => {
            let room = {
                let locked = clients.lock().await;
                locked.get(client_id).map(|c| c.room.clone()).unwrap_or_else(|| "lobby".to_string())
            };
            send_to_client(clients, client_id, &format!("Current room: {}", room)).await;
        }
        "/name" => {
            if let Some(newname) = parts.next() {
                let newname = newname.trim();
                if newname.is_empty() {
                    send_to_client(clients, client_id, "Usage: /name <new_name>").await;
                    return;
                }
                let unique_name = make_unique_name(clients, newname).await;
                {
                    let mut locked = clients.lock().await;
                    if let Some(c) = locked.get_mut(client_id) {
                        c.name = unique_name.clone();
                    }
                }
                let room = get_client_room(clients, client_id).await;
                send_system_to_room(clients, histories, &room, &format!("-- {} is now known as {} --", client_name_by_id(clients, client_id).await, unique_name)).await;
                send_user_list_to_room(clients, &room).await;
                send_to_client(clients, client_id, &format!("Your name is now '{}'", unique_name)).await;
            }
        }
        "/list" => {
            let room = get_client_room(clients, client_id).await;
            send_user_list_to_room(clients, &room).await;
        }
        "/register" => {
            if let (Some(username), Some(password)) = (parts.next(), parts.next()) {
                match register_user(users, username.trim(), password.trim()).await {
                    Ok(_) => {
                        send_to_client(clients, client_id, &format!("Registered '{}'. Use /login to authenticate.", username.trim())).await;
                    }
                    Err(e) => {
                        send_to_client(clients, client_id, &format!("Register failed: {}", e)).await;
                    }
                }
            }
        }
        "/login" => {
            if let (Some(username), Some(password)) = (parts.next(), parts.next()) {
                if verify_login(users, username.trim(), password.trim()).await {
                    let unique_name = make_unique_name(clients, username.trim()).await;
                    {
                        let mut locked = clients.lock().await;
                        if let Some(c) = locked.get_mut(client_id) {
                            c.name = unique_name.clone();
                            c.logged_in = true;
                        }
                    }
                    let room = get_client_room(clients, client_id).await;
                    send_system_to_room(clients, histories, &room, &format!("-- {} logged in --", unique_name)).await;
                    send_user_list_to_room(clients, &room).await;
                    send_to_client(clients, client_id, &format!("Logged in as '{}'", unique_name)).await;
                } else {
                    send_to_client(clients, client_id, "Login failed: invalid credentials").await;
                }
            }
        }
        "/history" => {
            if let Some(tx) = client_tx_by_id(clients, client_id).await {
                let room = get_client_room(clients, client_id).await;
                send_history_to_client_room(&tx, histories, &room).await;
            }
        }
        "/msg" => {
            if let (Some(target), Some(text)) = (parts.next(), parts.next()) {
                let target_name = target.trim().to_lowercase();
                let maybe_tx = {
                    let locked = clients.lock().await;
                    let current_room = locked.get(client_id).map(|x| x.room.clone()).unwrap_or_else(|| "lobby".to_string());
                    locked.values()
                        .find(|c| c.name.to_lowercase() == target_name && c.room == current_room)
                        .map(|c| c.tx.clone())
                };
                if let Some(tx) = maybe_tx {
                    let from = client_name_by_id(clients, client_id).await;
                    let msg_id = generate_msg_id();
                    let item = Outgoing::Msg { 
                        id: msg_id.clone(), 
                        from: from.clone(), 
                        text: text.to_string(), 
                        ts: now_ts(),
                        reactions: HashMap::new(),
                        edited: false,
                    };
                    if let Ok(s) = serde_json::to_string(&item) {
                        let _ = tx.send(warp::ws::Message::text(s));
                    }
                    // Store in history
                    let room = get_client_room(clients, client_id).await;
                    let mut locked_h = histories.lock().await;
                    let q = locked_h.entry(room).or_insert_with(|| VecDeque::with_capacity(200));
                    q.push_back(HistoryItem { 
                        id: msg_id,
                        from: format!("(private) {} -> {}", from, target.trim()), 
                        text: text.to_string(), 
                        ts: now_ts(),
                        reactions: HashMap::new(),
                        edited: false,
                        deleted: false,
                    });
                    while q.len() > 200 { q.pop_front(); }
                } else {
                    send_to_client(clients, client_id, &format!("User '{}' not found in your room", target.trim())).await;
                }
            }
        }
        "/help" => {
            let help_text = r#"Available commands:
  /name <name>     - Set your display name
  /register <u> <p> - Create an account
  /login <u> <p>    - Log in to your account
  /msg <user> <text> - Private message a user
  /join <room>     - Join or create a room
  /leave           - Return to lobby
  /rooms           - List all rooms
  /room            - Show current room
  /list            - List users in room
  /who             - Show users with status
  /history         - Reload chat history
  /help            - Show this help"#;
            send_to_client(clients, client_id, help_text).await;
        }
        "/who" => {
            let room = get_client_room(clients, client_id).await;
            let user_info: Vec<String> = {
                let locked = clients.lock().await;
                locked.values()
                    .filter(|c| c.room == room)
                    .map(|c| {
                        let status = if c.logged_in { "✓" } else { "guest" };
                        format!("{} ({})", c.name, status)
                    })
                    .collect()
            };
            send_to_client(clients, client_id, &format!("Users in '{}': {}", room, user_info.join(", "))).await;
        }
        _ => {
            send_to_client(clients, client_id, "Unknown command. Type /help for available commands.").await;
        }
    }
}

/// Handle regular chat messages.
pub async fn handle_message_with_rooms(
    client_id: &str,
    text: &str,
    clients: &Clients,
    histories: &Histories,
) {
    let from = client_name_by_id(clients, client_id).await;
    let room = get_client_room(clients, client_id).await;
    let item = HistoryItem {
        id: generate_msg_id(),
        from,
        text: text.to_string(),
        ts: now_ts(),
        reactions: HashMap::new(),
        edited: false,
        deleted: false,
    };
    broadcast_to_room_and_store(clients, histories, &room, item).await;
}

/// Helper: send system message to a single client.
async fn send_to_client(clients: &Clients, client_id: &str, text: &str) {
    if let Some(tx) = client_tx_by_id(clients, client_id).await {
        let msg = Outgoing::System { text: text.to_string() };
        let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&msg).unwrap()));
    }
}

/// Helper: get client's current room.
async fn get_client_room(clients: &Clients, client_id: &str) -> String {
    let locked = clients.lock().await;
    locked.get(client_id).map(|c| c.room.clone()).unwrap_or_else(|| "lobby".to_string())
}
