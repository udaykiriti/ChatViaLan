//! Command handling for chat commands.

use crate::auth::{register_user, verify_login};
use crate::helpers::{client_name_by_id, client_tx_by_id, make_unique_name, now_ts};
use crate::room::{
    broadcast_to_room_and_store, generate_msg_id, join_room, send_history_to_client_room,
    send_system_to_room, send_user_list_to_room,
};
use crate::types::{Clients, Histories, HistoryItem, Outgoing, RoomInfo, Users};
use std::collections::{HashMap, VecDeque};
use tracing::info;

/// Handle all `/` commands from a connected client.
pub async fn handle_cmd_with_rooms(
    client_id: &str,
    cmd_line: &str,
    clients: &Clients,
    histories: &Histories,
    private_histories: &crate::types::PrivateHistories,
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
            let room_list: Vec<RoomInfo> = {
                let locked_h = histories.read().await;
                locked_h
                    .keys()
                    .map(|room_name| {
                        let member_count = clients
                            .iter()
                            .filter(|r| &r.value().room == room_name)
                            .count();
                        RoomInfo {
                            name: room_name.clone(),
                            members: member_count,
                        }
                    })
                    .collect()
            };
            // Send structured room list
            let msg = Outgoing::RoomList { rooms: room_list };
            if let Some(tx) = client_tx_by_id(clients, client_id).await {
                if let Ok(s) = serde_json::to_string(&msg) {
                    let _ = tx.send(warp::ws::Message::text(s));
                }
            }
        }
        "/leave" => {
            join_room(client_id, "lobby", clients, histories).await;
        }
        "/room" => {
            let room = clients
                .get(client_id)
                .map(|r| r.value().room.clone())
                .unwrap_or_else(|| "lobby".to_string());
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
                let old_name = client_name_by_id(clients, client_id).await;
                if let Some(mut r) = clients.get_mut(client_id) {
                    r.value_mut().name = unique_name.clone();
                }
                let room = get_client_room(clients, client_id).await;
                send_system_to_room(
                    clients,
                    histories,
                    &room,
                    &format!("-- {} is now known as {} --", old_name, unique_name),
                )
                .await;
                send_user_list_to_room(clients, &room).await;
                send_to_client(
                    clients,
                    client_id,
                    &format!("Your name is now '{}'", unique_name),
                )
                .await;
                info!(
                    "Client {} (id: {}) changed name to {}",
                    old_name, client_id, unique_name
                );
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
                        send_to_client(
                            clients,
                            client_id,
                            &format!(
                                "Registered '{}'. Use /login to authenticate.",
                                username.trim()
                            ),
                        )
                        .await;
                    }
                    Err(e) => {
                        send_to_client(clients, client_id, &format!("Register failed: {}", e))
                            .await;
                    }
                }
            }
        }
        "/login" => {
            if let (Some(username), Some(password)) = (parts.next(), parts.next()) {
                if verify_login(users, username.trim(), password.trim()).await {
                    let unique_name = make_unique_name(clients, username.trim()).await;
                    if let Some(mut r) = clients.get_mut(client_id) {
                        let c = r.value_mut();
                        c.name = unique_name.clone();
                        c.logged_in = true;
                    }
                    let room = get_client_room(clients, client_id).await;
                    send_system_to_room(
                        clients,
                        histories,
                        &room,
                        &format!("-- {} logged in --", unique_name),
                    )
                    .await;
                    send_user_list_to_room(clients, &room).await;
                    send_to_client(
                        clients,
                        client_id,
                        &format!("Logged in as '{}'", unique_name),
                    )
                    .await;
                    info!("Client {} logged in as {}", client_id, unique_name);
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
                    let current_room = clients
                        .get(client_id)
                        .map(|r| r.value().room.clone())
                        .unwrap_or_else(|| "lobby".to_string());
                    clients
                        .iter()
                        .find(|r| {
                            let c = r.value();
                            c.name.to_lowercase() == target_name && c.room == current_room
                        })
                        .map(|r| r.value().tx.clone())
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

                    // Securely store in PrivateHistories
                    // Key: "alfred,batman" (sorted, case-insensitive)
                    let mut participants = [from.to_lowercase(), target_name.clone()];
                    participants.sort_unstable();
                    let key = participants.join(",");

                    let mut locked_ph = private_histories.write().await;
                    let q = locked_ph
                        .entry(key)
                        .or_insert_with(|| VecDeque::with_capacity(200));
                    q.push_back(HistoryItem {
                        id: msg_id,
                        from,
                        text: text.to_string(),
                        ts: now_ts(),
                        reactions: HashMap::new(),
                        edited: false,
                        deleted: false,
                    });
                    while q.len() > 200 {
                        q.pop_front();
                    }
                } else {
                    send_to_client(
                        clients,
                        client_id,
                        &format!("User '{}' not found in your room", target.trim()),
                    )
                    .await;
                }
            }
        }
        "/kick" => {
            if let Some(target) = parts.next() {
                let target_name = target.trim().to_lowercase();
                let is_logged_in = clients
                    .get(client_id)
                    .map(|r| r.value().logged_in)
                    .unwrap_or(false);

                if !is_logged_in {
                    send_to_client(clients, client_id, "You must be logged in to kick users.")
                        .await;
                    return;
                }

                let maybe_target_id = clients
                    .iter()
                    .find(|r| r.value().name.to_lowercase() == target_name)
                    .map(|r| r.key().clone());

                if let Some(tid) = maybe_target_id {
                    if tid == client_id {
                        send_to_client(clients, client_id, "You cannot kick yourself!").await;
                        return;
                    }
                    let target_disp_name = clients
                        .get(&tid)
                        .map(|r| r.value().name.clone())
                        .unwrap_or_default();
                    let room = get_client_room(clients, client_id).await;

                    // Notify room
                    send_system_to_room(
                        clients,
                        histories,
                        &room,
                        &format!("-- {} has been kicked by an admin --", target_disp_name),
                    )
                    .await;

                    // Close connection (by removing from clients)
                    clients.remove(&tid);
                    info!("Client {} was kicked by {}", target_disp_name, client_id);
                } else {
                    send_to_client(
                        clients,
                        client_id,
                        &format!("User '{}' not found", target.trim()),
                    )
                    .await;
                }
            } else {
                send_to_client(clients, client_id, "Usage: /kick <user>").await;
            }
        }
        "/stats" => {
            let total_clients = clients.len();
            let total_rooms = histories.read().await.len();
            let total_messages: usize = histories.read().await.values().map(|v| v.len()).sum();

            // Memory Usage (Approximate using /proc/self/statm on Linux)
            let mem_usage = if let Ok(content) = std::fs::read_to_string("/proc/self/statm") {
                let parts: Vec<&str> = content.split_whitespace().collect();
                if let Some(pages) = parts.get(1) {
                    if let Ok(pages_cnt) = pages.parse::<usize>() {
                        format!("{:.2} MB", (pages_cnt * 4) as f64 / 1024.0) // Assuming 4KB pages
                    } else {
                        "N/A".to_string()
                    }
                } else {
                    "N/A".to_string()
                }
            } else {
                "N/A".to_string()
            };

            let stats_msg = format!(
                "Server Stats:\nClients: {}\nRooms: {}\nMessages: {}\nMem: {}",
                total_clients, total_rooms, total_messages, mem_usage
            );
            send_to_client(clients, client_id, &stats_msg).await;
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
  /kick <user>     - Kick a user (logged-in only)
  /nudge           - Send a nudge (shake screen)
  /history         - Reload chat history
  /stats           - Show server metrics
  /help            - Show this help"#;
            send_to_client(clients, client_id, help_text).await;
        }
        "/who" => {
            let room = get_client_room(clients, client_id).await;
            let user_info: Vec<String> = clients
                .iter()
                .filter(|r| r.value().room == room)
                .map(|r| {
                    let c = r.value();
                    let status = if c.logged_in { "✓" } else { "guest" };
                    format!("{} ({})", c.name, status)
                })
                .collect();
            send_to_client(
                clients,
                client_id,
                &format!("Users in '{}': {}", room, user_info.join(", ")),
            )
            .await;
        }
        "/nudge" => {
            let from = client_name_by_id(clients, client_id).await;
            let room = get_client_room(clients, client_id).await;

            // Broadcast Nudge
            let msg = Outgoing::Nudge { from: from.clone() };
            if let Ok(json) = serde_json::to_string(&msg) {
                for r in clients.iter() {
                    if r.value().room == room {
                        let _ = r.value().tx.send(warp::ws::Message::text(json.clone()));
                    }
                }
            }

            // System message announcement
            send_system_to_room(
                clients,
                histories,
                &room,
                &format!("{} sent a nudge!", from),
            )
            .await;
        }
        _ => {
            send_to_client(
                clients,
                client_id,
                "Unknown command. Type /help for available commands.",
            )
            .await;
        }
    }
}

/// Handle regular chat messages.
pub async fn handle_message_with_rooms(
    client_id: &str,
    text: &str,
    clients: &Clients,
    histories: &Histories,
    metrics: &std::sync::Arc<crate::metrics::ServerMetrics>,
) {
    let from = client_name_by_id(clients, client_id).await;
    let room = get_client_room(clients, client_id).await;
    let filtered_text = crate::helpers::censor_profanity(text);
    let item = HistoryItem {
        id: generate_msg_id(),
        from,
        text: filtered_text,
        ts: now_ts(),
        reactions: HashMap::new(),
        edited: false,
        deleted: false,
    };
    broadcast_to_room_and_store(clients, histories, &room, item.clone()).await;

    // Increment message counter
    metrics.increment_messages();

    // Check for URLs and fetch previews
    static URL_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let url_regex = URL_RE.get_or_init(|| regex::Regex::new(r"(https?://\S+)").unwrap());
    if let Some(captures) = url_regex.captures(text) {
        if let Some(match_) = captures.get(0) {
            let url = match_.as_str().to_string();
            let clients_clone = clients.clone();
            let room_clone = room.clone();
            let msg_id = item.id.clone();

            // Spawn background task to fetch metadata
            tokio::spawn(async move {
                if let Some((title, desc, image)) = crate::helpers::fetch_preview(&url).await {
                    let preview_msg = Outgoing::LinkPreview {
                        msg_id,
                        title,
                        description: desc,
                        image,
                        url,
                    };
                    // Broadcast preview to proper room
                    if let Ok(json) = serde_json::to_string(&preview_msg) {
                        for r in clients_clone.iter() {
                            if r.value().room == room_clone {
                                let _ = r.value().tx.send(warp::ws::Message::text(json.clone()));
                            }
                        }
                    }
                }
            });
        }
    }
}

/// Helper: send system message to a single client.
async fn send_to_client(clients: &Clients, client_id: &str, text: &str) {
    if let Some(tx) = client_tx_by_id(clients, client_id).await {
        let msg = Outgoing::System {
            text: text.to_string(),
        };
        if let Ok(payload) = serde_json::to_string(&msg) {
            let _ = tx.send(warp::ws::Message::text(payload));
        }
    }
}

/// Helper: get client's current room.
async fn get_client_room(clients: &Clients, client_id: &str) -> String {
    clients
        .get(client_id)
        .map(|r| r.value().room.clone())
        .unwrap_or_else(|| "lobby".to_string())
}
