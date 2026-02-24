//! WebSocket client handling and connection lifecycle.

use futures::{SinkExt, StreamExt};
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::auth::{register_user, verify_login};
use crate::commands::{handle_cmd_with_rooms, handle_message_with_rooms};
use crate::helpers::{client_tx_by_id, make_unique_name};
use crate::rate_limit::check_rate_limit;
use crate::room::{send_history_to_client_room, send_system_to_room, send_user_list_to_room};
use crate::types::{Client, Clients, Histories, Incoming, Outgoing, PrivateHistories, Tx, Users};
use crate::typing::{broadcast_typing_status, set_typing_status};

/// Handle a new WebSocket connection.
pub async fn client_connected(
    ws: warp::ws::WebSocket,
    remote: Option<std::net::SocketAddr>,
    clients: Clients,
    histories: Histories,
    private_histories: PrivateHistories,
    users: Users,
    metrics: std::sync::Arc<crate::metrics::ServerMetrics>,
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
        let msg = Outgoing::System {
            text: text.to_string(),
        };
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
    let mut auth_completed = false;
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
                                                chosen_name =
                                                    make_unique_name(&clients, name).await;
                                                logged_in = false;
                                                send_system_to_this(&tx, &format!("Your name is '{}'. You are not authenticated.", chosen_name));
                                                auth_completed = true;
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
                                        if let (Some(username), Some(password)) =
                                            (username_opt, password_opt)
                                        {
                                            let username = username.trim().to_string();
                                            let password = password.trim().to_string();
                                            match register_user(&users, &username, &password).await
                                            {
                                                Ok(_) => {
                                                    chosen_name =
                                                        make_unique_name(&clients, &username).await;
                                                    logged_in = true;
                                                    send_system_to_this(
                                                        &tx,
                                                        &format!(
                                                            "Registered and logged in as '{}'",
                                                            chosen_name
                                                        ),
                                                    );
                                                    auth_completed = true;
                                                    break;
                                                }
                                                Err(e) => {
                                                    send_system_to_this(
                                                        &tx,
                                                        &format!("Register failed: {}", e),
                                                    );
                                                }
                                            }
                                        } else {
                                            send_system_to_this(
                                                &tx,
                                                "Usage: /register <username> <password>",
                                            );
                                        }
                                    }
                                    "/login" => {
                                        let username_opt = parts.next();
                                        let password_opt = parts.next();
                                        if let (Some(username), Some(password)) =
                                            (username_opt, password_opt)
                                        {
                                            let username = username.trim().to_string();
                                            let password = password.trim().to_string();
                                            if verify_login(&users, &username, &password).await {
                                                chosen_name =
                                                    make_unique_name(&clients, &username).await;
                                                logged_in = true;
                                                send_system_to_this(
                                                    &tx,
                                                    &format!("Logged in as '{}'", chosen_name),
                                                );
                                                auth_completed = true;
                                                break;
                                            } else {
                                                send_system_to_this(
                                                    &tx,
                                                    "Login failed: invalid username or password",
                                                );
                                            }
                                        } else {
                                            send_system_to_this(
                                                &tx,
                                                "Usage: /login <username> <password>",
                                            );
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
                            Ok(Incoming::Typing { .. }) => {
                                // Ignore typing during auth phase
                            }
                            Ok(Incoming::React { .. })
                            | Ok(Incoming::Edit { .. })
                            | Ok(Incoming::Delete { .. })
                            | Ok(Incoming::MarkRead { .. }) => {
                                // Ignore these during auth phase
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
                warn!("websocket error during auth for {}: {}", addr, e);
                drop(tx);
                let _ = forward_task.await;
                return;
            }
        }
    }

    // Connection ended before auth/name selection completed.
    if !auth_completed {
        drop(tx);
        let _ = forward_task.await;
        return;
    }

    // Register the client
    let client = Client {
        name: chosen_name.clone(),
        tx: tx.clone(),
        room: default_room.clone(),
        last_message_times: Vec::new(),
        is_typing: false,
        last_read_msg_id: None,
        last_active: Instant::now(),
        logged_in,
    };
    clients.insert(client_id.clone(), client);

    // Increment connection counter
    metrics.increment_connections();

    info!(
        "New connection: {} (id: {}, name: {}, logged_in={}, room={})",
        addr, client_id, chosen_name, logged_in, default_room
    );

    // Announce in lobby
    send_system_to_room(
        &clients,
        &histories,
        &default_room,
        &format!("-- {} joined the room --", chosen_name),
    )
    .await;
    send_history_to_client_room(&tx, &histories, &default_room).await;
    send_user_list_to_room(&clients, &default_room).await;

    // Main message loop
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    // Update activity
                    if let Some(mut r) = clients.get_mut(&client_id) {
                        r.value_mut().last_active = std::time::Instant::now();
                    }

                    if let Ok(text) = msg.to_str() {
                        match serde_json::from_str::<Incoming>(text) {
                            Ok(Incoming::Cmd { cmd }) => {
                                handle_cmd_with_rooms(
                                    &client_id,
                                    &cmd,
                                    &clients,
                                    &histories,
                                    &private_histories,
                                    &users,
                                )
                                .await;
                            }
                            Ok(Incoming::Msg { text }) => {
                                if check_rate_limit(&clients, &client_id).await {
                                    handle_message_with_rooms(
                                        &client_id, &text, &clients, &histories, &metrics,
                                    )
                                    .await;
                                    set_typing_status(&clients, &client_id, false).await;
                                }
                            }
                            Ok(Incoming::Typing { is_typing }) => {
                                set_typing_status(&clients, &client_id, is_typing).await;
                                broadcast_typing_status(&clients, &client_id).await;
                            }
                            Ok(Incoming::React { msg_id, emoji }) => {
                                let (room, name) = {
                                    clients
                                        .get(&client_id)
                                        .map(|r| {
                                            let c = r.value();
                                            (c.room.clone(), c.name.clone())
                                        })
                                        .unwrap_or_default()
                                };
                                crate::room::add_reaction(
                                    &clients, &histories, &room, &msg_id, &emoji, &name,
                                )
                                .await;
                            }
                            Ok(Incoming::Edit { msg_id, new_text }) => {
                                let (room, name) = {
                                    clients
                                        .get(&client_id)
                                        .map(|r| {
                                            let c = r.value();
                                            (c.room.clone(), c.name.clone())
                                        })
                                        .unwrap_or_default()
                                };
                                let edited = crate::room::edit_message(
                                    &clients, &histories, &room, &msg_id, &new_text, &name,
                                )
                                .await;
                                if !edited {
                                    if let Some(tx) = client_tx_by_id(&clients, &client_id).await {
                                        let msg = Outgoing::System {
                                            text: "Cannot edit this message".to_string(),
                                        };
                                        if let Ok(payload) = serde_json::to_string(&msg) {
                                            let _ = tx.send(warp::ws::Message::text(payload));
                                        }
                                    }
                                }
                            }
                            Ok(Incoming::Delete { msg_id }) => {
                                let (room, name) = {
                                    clients
                                        .get(&client_id)
                                        .map(|r| {
                                            let c = r.value();
                                            (c.room.clone(), c.name.clone())
                                        })
                                        .unwrap_or_default()
                                };
                                let deleted = crate::room::delete_message(
                                    &clients, &histories, &room, &msg_id, &name,
                                )
                                .await;
                                if !deleted {
                                    if let Some(tx) = client_tx_by_id(&clients, &client_id).await {
                                        let msg = Outgoing::System {
                                            text: "Cannot delete this message".to_string(),
                                        };
                                        if let Ok(payload) = serde_json::to_string(&msg) {
                                            let _ = tx.send(warp::ws::Message::text(payload));
                                        }
                                    }
                                }
                            }
                            Ok(Incoming::MarkRead { last_msg_id }) => {
                                let (room, name) = {
                                    if let Some(mut r) = clients.get_mut(&client_id) {
                                        let c = r.value_mut();
                                        c.last_read_msg_id = Some(last_msg_id.clone());
                                        (c.room.clone(), c.name.clone())
                                    } else {
                                        (String::new(), String::new())
                                    }
                                };
                                if !room.is_empty() {
                                    crate::room::broadcast_read_receipt(
                                        &clients,
                                        &room,
                                        &name,
                                        &last_msg_id,
                                    )
                                    .await;
                                }
                            }
                            Err(_) => {
                                if check_rate_limit(&clients, &client_id).await {
                                    handle_message_with_rooms(
                                        &client_id, text, &clients, &histories, &metrics,
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                } else if msg.is_close() {
                    break;
                }
            }
            Err(e) => {
                warn!("websocket error for {}: {}", addr, e);
                break;
            }
        }
    }

    // Cleanup
    let left_room = clients.remove(&client_id).map(|(_, c)| c.room);

    if let Some(room) = left_room {
        send_system_to_room(
            &clients,
            &histories,
            &room,
            &format!("-- {} left the room --", chosen_name),
        )
        .await;
        send_user_list_to_room(&clients, &room).await;
        info!(
            "Client disconnected: {} (name: {}, room: {})",
            client_id, chosen_name, room
        );
    }

    drop(tx);
    let _ = forward_task.await;
}
