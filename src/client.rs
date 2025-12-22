//! WebSocket client handling and connection lifecycle.

use std::time::SystemTime;
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::types::{Client, Clients, Histories, Incoming, Outgoing, Tx, Users};
use crate::auth::{register_user, verify_login};
use crate::room::{send_system_to_room, send_history_to_client_room, send_user_list_to_room};
use crate::commands::{handle_cmd_with_rooms, handle_message_with_rooms};

/// Handle a new WebSocket connection.
pub async fn client_connected(
    ws: warp::ws::WebSocket,
    remote: Option<std::net::SocketAddr>,
    clients: Clients,
    histories: Histories,
    users: Users,
) {
    let addr = remote
        .map(|a| a.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Forward task: send messages from rx to WebSocket sink
    let forward_task = tokio::task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    let client_id = Uuid::new_v4().to_string();
    let default_room = "lobby".to_string();
    let mut chosen_name = format!("guest-{}", &client_id[..6]);
    let mut logged_in = false;

    // Helper to send system message to this connection only
    let send_system_to_this = |tx: &Tx, text: &str| {
        let msg = Outgoing::System { text: text.to_string() };
        if let Ok(s) = serde_json::to_string(&msg) {
            let _ = tx.send(warp::ws::Message::text(s));
        }
    };

    // Welcome prompt
    send_system_to_this(
        &tx,
        "Welcome — choose a username, or /register or /login. Use /join <room> to switch rooms.",
    );

    // Auth / name phase
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    if let Ok(text) = msg.to_str() {
                        match serde_json::from_str::<Incoming>(text) {
                            Ok(Incoming::Cmd { cmd }) => {
                                let mut parts = cmd.splitn(3, ' ');
                                match parts.next().unwrap_or("") {
                                    "/name" => {
                                        if let Some(name) = parts.next() {
                                            let name = name.trim();
                                            if !name.is_empty() {
                                                chosen_name = make_unique_name(&clients, name).await;
                                                logged_in = false;
                                                send_system_to_this(&tx, &format!("Your name is '{}'. You are not authenticated.", chosen_name));
                                                break;
                                            } else {
                                                send_system_to_this(&tx, "Usage: /name <username>");
                                            }
                                        } else {
                                            send_system_to_this(&tx, "Usage: /name <username>");
                                        }
                                    }
                                    "/register" => {
                                        let username_opt = parts.next();
                                        let password_opt = parts.next();
                                        if let (Some(username), Some(password)) = (username_opt, password_opt) {
                                            let username = username.trim().to_string();
                                            let password = password.trim().to_string();
                                            match register_user(&users, &username, &password).await {
                                                Ok(_) => {
                                                    chosen_name = make_unique_name(&clients, &username).await;
                                                    logged_in = true;
                                                    send_system_to_this(&tx, &format!("Registered and logged in as '{}'", chosen_name));
                                                    break;
                                                }
                                                Err(e) => {
                                                    send_system_to_this(&tx, &format!("Register failed: {}", e));
                                                }
                                            }
                                        } else {
                                            send_system_to_this(&tx, "Usage: /register <username> <password>");
                                        }
                                    }
                                    "/login" => {
                                        let username_opt = parts.next();
                                        let password_opt = parts.next();
                                        if let (Some(username), Some(password)) = (username_opt, password_opt) {
                                            let username = username.trim().to_string();
                                            let password = password.trim().to_string();
                                            if verify_login(&users, &username, &password).await {
                                                chosen_name = make_unique_name(&clients, &username).await;
                                                logged_in = true;
                                                send_system_to_this(&tx, &format!("Logged in as '{}'", chosen_name));
                                                break;
                                            } else {
                                                send_system_to_this(&tx, "Login failed: invalid username or password");
                                            }
                                        } else {
                                            send_system_to_this(&tx, "Usage: /login <username> <password>");
                                        }
                                    }
                                    other => {
                                        send_system_to_this(&tx, &format!("Please choose a name or login/register first. Unknown: {}", other));
                                    }
                                }
                            }
                            Ok(Incoming::Msg { .. }) => {
                                send_system_to_this(&tx, "Please choose a name or login/register before sending messages.");
                            }
                            Err(_) => {
                                send_system_to_this(&tx, "Please choose a name or login/register before sending messages.");
                            }
                        }
                    }
                } else if msg.is_close() {
                    drop(tx);
                    let _ = forward_task.await;
                    return;
                }
            }
            Err(e) => {
                eprintln!("websocket error during auth for {}: {}", addr, e);
                drop(tx);
                let _ = forward_task.await;
                return;
            }
        }
    }

    // Register client in default room
    let client = Client {
        name: chosen_name.clone(),
        tx: tx.clone(),
        logged_in,
        room: default_room.clone(),
    };

    {
        let mut locked = clients.lock().await;
        locked.insert(client_id.clone(), client);
    }

    println!("New connection: {} (id: {}, name: {}, logged_in={}, room={})", addr, client_id, chosen_name, logged_in, default_room);

    // Announce in lobby
    send_system_to_room(&clients, &histories, &default_room, &format!("-- {} joined the room --", chosen_name)).await;
    send_history_to_client_room(&tx, &histories, &default_room).await;
    send_user_list_to_room(&clients, &default_room).await;

    // Main message loop
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    if let Ok(text) = msg.to_str() {
                        match serde_json::from_str::<Incoming>(text) {
                            Ok(Incoming::Cmd { cmd }) => {
                                handle_cmd_with_rooms(&client_id, &cmd, &clients, &histories, &users).await;
                            }
                            Ok(Incoming::Msg { text }) => {
                                handle_message_with_rooms(&client_id, &text, &clients, &histories).await;
                            }
                            Err(_) => {
                                handle_message_with_rooms(&client_id, text, &clients, &histories).await;
                            }
                        }
                    }
                } else if msg.is_close() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("websocket error for {}: {}", addr, e);
                break;
            }
        }
    }

    // Cleanup
    let left_room = {
        let mut locked = clients.lock().await;
        locked.remove(&client_id).map(|c| c.room)
    };

    if let Some(room) = left_room {
        send_system_to_room(&clients, &histories, &room, &format!("-- {} left the room --", chosen_name)).await;
        send_user_list_to_room(&clients, &room).await;
    }

    drop(tx);
    let _ = forward_task.await;
}

/// Get client name by ID.
pub async fn client_name_by_id(clients: &Clients, id: &str) -> String {
    let locked = clients.lock().await;
    locked.get(id).map(|c| c.name.clone()).unwrap_or_else(|| id.to_string())
}

/// Get client tx channel by ID.
pub async fn client_tx_by_id(clients: &Clients, id: &str) -> Option<Tx> {
    let locked = clients.lock().await;
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
        let locked = clients.lock().await;
        let collision = locked.values().any(|c| c.name.eq_ignore_ascii_case(&candidate));
        drop(locked);
        if !collision {
            return candidate;
        }
        candidate = format!("{}-{}", desired, suffix);
        suffix += 1;
    }
}
