use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::Path,
    sync::Arc,
    time::SystemTime,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::{mpsc, Mutex};
use warp::Filter;
use uuid::Uuid;
use warp::multipart::{FormData, Part};
use warp::http::StatusCode;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use bytes::Buf;
use futures::TryStreamExt;

type Tx = mpsc::UnboundedSender<warp::ws::Message>;

#[derive(Clone)]
struct Client {
    name: String,
    tx: Tx,
    logged_in: bool,
    room: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Outgoing {
    System { text: String },
    Msg { from: String, text: String, ts: u64 },
    List { users: Vec<String> },
    History { items: Vec<HistoryItem> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HistoryItem {
    from: String,
    text: String,
    ts: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Incoming {
    Cmd { cmd: String },
    Msg { text: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load users from disk (username -> password_hash)
    let users_map = load_users().unwrap_or_default();
    let users = Arc::new(Mutex::new(users_map));

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    // histories per room
    let histories: Histories = Arc::new(Mutex::new(HashMap::new()));
    // ensure default "lobby" exists
    {
        let mut h = histories.lock().await;
        h.entry("lobby".to_string()).or_insert_with(|| VecDeque::with_capacity(200));
    }

    let clients_filter = warp::any().map(move || clients.clone());
    let histories_filter = warp::any().map(move || histories.clone());
    let users_filter = warp::any().map(move || users.clone());

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(clients_filter)
        .and(histories_filter)
        .and(users_filter)
        .map(
            |ws: warp::ws::Ws,
             remote: Option<std::net::SocketAddr>,
             clients,
             histories,
             users| {
                ws.on_upgrade(move |socket| {
                    client_connected(socket, remote, clients, histories, users)
                })
            },
        );

    // serve the web UI file at "/"
    let static_route = warp::path::end().and(warp::fs::file("static/index.html"));

    // serve uploaded files under /uploads
    let uploads_route = warp::path("uploads").and(warp::fs::dir("uploads"));

    // POST /upload => multipart handler
    let upload_route = warp::path("upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(50_000_000))
        .and_then(handle_upload);

    // combine all routes
    let routes = ws_route
        .or(static_route)
        .or(uploads_route)
        .or(upload_route)
        .with(warp::cors().allow_any_origin());

    // Ensure uploads directory exists at startup
    if let Err(e) = std::fs::create_dir_all("uploads") {
        eprintln!("warning: failed to create uploads dir at startup: {}", e);
    }

    println!("Server running at http://0.0.0.0:8080/");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;

    Ok(())
}

type Clients = Arc<Mutex<HashMap<String, Client>>>;
type Histories = Arc<Mutex<HashMap<String, VecDeque<HistoryItem>>>>;
type Users = Arc<Mutex<HashMap<String, String>>>; // username -> password_hash

async fn client_connected(
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

    // forward task: send messages from rx to websocket sink
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

    // helper to send system message to this connection only
    let send_system_to_this = |tx: &Tx, text: &str| {
        let msg = Outgoing::System {
            text: text.to_string(),
        };
        if let Ok(s) = serde_json::to_string(&msg) {
            let _ = tx.send(warp::ws::Message::text(s));
        }
    };

    // prompt
    send_system_to_this(
        &tx,
        "Welcome — choose a username, or /register or /login. Use /join <room> to switch rooms. Default room is 'lobby'.",
    );

    // auth / name first (same flow as before)
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
                                                send_system_to_this(
                                                    &tx,
                                                    &format!("Your name is '{}'. You are not authenticated.", chosen_name),
                                                );
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
                                        send_system_to_this(&tx, &format!("Please choose a name or login/register first. Unknown command: {}", other));
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

    // register client in default room
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

    // announce in lobby only
    send_system_to_room(&clients, &histories, &default_room, &format!("-- {} joined the room --", chosen_name)).await;
    send_history_to_client_room(&tx, &histories, &default_room).await;
    send_user_list_to_room(&clients, &default_room).await;

    // main loop: incoming messages/commands
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
                                // fallback: treat raw text as message
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

    // cleanup: remove client and announce in their room
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

/// Send a system message only to users in a room (also record optional history)
async fn send_system_to_room(clients: &Clients, histories: &Histories, room: &str, text: &str) {
    let msg = Outgoing::System { text: text.to_string() };
    // record to room history as system message
    let item = HistoryItem { from: "system".to_string(), text: text.to_string(), ts: now_ts() };
    {
        let mut locked_h = histories.lock().await;
        let q = locked_h.entry(room.to_string()).or_insert_with(|| VecDeque::with_capacity(200));
        q.push_back(item);
        while q.len() > 200 { q.pop_front(); }
    }
    let s = serde_json::to_string(&msg).unwrap_or_default();
    let locked = clients.lock().await;
    for c in locked.values() {
        if c.room == room {
            let _ = c.tx.send(warp::ws::Message::text(s.clone()));
        }
    }
}

/// Send history of a room to a single client tx
async fn send_history_to_client_room(tx: &Tx, histories: &Histories, room: &str) {
    let items: Vec<HistoryItem> = {
        let locked = histories.lock().await;
        locked.get(room).map(|q| q.iter().cloned().collect()).unwrap_or_default()
    };
    let msg = Outgoing::History { items };
    if let Ok(text) = serde_json::to_string(&msg) {
        let _ = tx.send(warp::ws::Message::text(text));
    }
}

/// Send user list only for a room (to those in that room)
async fn send_user_list_to_room(clients: &Clients, room: &str) {
    let names: Vec<String> = {
        let locked = clients.lock().await;
        locked.values().filter(|c| c.room == room).map(|c| c.name.clone()).collect()
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

/// Broadcast a message to all clients in a room and record to history
async fn broadcast_to_room_and_store(clients: &Clients, histories: &Histories, room: &str, item: HistoryItem) {
    // store
    {
        let mut locked_h = histories.lock().await;
        let q = locked_h.entry(room.to_string()).or_insert_with(|| VecDeque::with_capacity(200));
        q.push_back(item.clone());
        while q.len() > 200 { q.pop_front(); }
    }
    // outgoing
    let outgoing = Outgoing::Msg { from: item.from.clone(), text: item.text.clone(), ts: item.ts };
    if let Ok(s) = serde_json::to_string(&outgoing) {
        let locked = clients.lock().await;
        for c in locked.values() {
            if c.room == room {
                let _ = c.tx.send(warp::ws::Message::text(s.clone()));
            }
        }
    }
}

/// Move a client into a room (creates room history if needed), announces leave/join, sends history & user lists.
async fn join_room(client_id: &str, room: &str, clients: &Clients, histories: &Histories) {
    let target = room.trim();
    if target.is_empty() {
        return;
    }

    // move client to new room and capture old room
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

    // ensure room exists
    {
        let mut locked_h = histories.lock().await;
        locked_h.entry(target.to_string()).or_insert_with(|| VecDeque::with_capacity(200));
    }

    // announce leave in old room and update list
    send_system_to_room(clients, histories, &old_room, &format!("-- {} left the room --", client_name_by_id(clients, client_id).await)).await;
    send_user_list_to_room(clients, &old_room).await;

    // announce join in new room and update list + send history to mover
    send_system_to_room(clients, histories, target, &format!("-- {} joined the room --", client_name_by_id(clients, client_id).await)).await;
    if let Some(tx) = client_tx_by_id(clients, client_id).await {
        send_history_to_client_room(&tx, histories, target).await;
    }
    send_user_list_to_room(clients, target).await;

    // notify the mover
    if let Some(tx) = client_tx_by_id(clients, client_id).await {
        let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: format!("You joined room '{}'", target) }).unwrap()));
    }
}

async fn handle_cmd_with_rooms(client_id: &str, cmd_line: &str, clients: &Clients, histories: &Histories, users: &Users) {
    let mut parts = cmd_line.splitn(3, ' ');
    let cmd = parts.next().unwrap_or("");
    match cmd {
        "/join" => {
            if let Some(room) = parts.next() {
                join_room(client_id, room.trim(), clients, histories).await;
            } else {
                if let Some(tx) = client_tx_by_id(clients, client_id).await {
                    let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: "Usage: /join <room>".to_string() }).unwrap()));
                }
            }
        }
        "/rooms" => {
            // gather rooms
            let rooms: Vec<String> = {
                let locked_h = histories.lock().await;
                locked_h.keys().cloned().collect()
            };
            if let Some(tx) = client_tx_by_id(clients, client_id).await {
                let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: format!("Rooms: {}", rooms.join(", ")) }).unwrap()));
            }
        }
        "/leave" => {
            // go back to lobby
            join_room(client_id, "lobby", clients, histories).await;
        }
        "/room" => {
            let room = {
                let locked = clients.lock().await;
                locked.get(client_id).map(|c| c.room.clone()).unwrap_or_else(|| "lobby".to_string())
            };
            if let Some(tx) = client_tx_by_id(clients, client_id).await {
                let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: format!("Current room: {}", room) }).unwrap()));
            }
        }
        "/name" | "/register" | "/login" | "/list" | "/history" | "/msg" => {
            match cmd {
                "/name" => {
                    if let Some(newname) = parts.next() {
                        let newname = newname.trim();
                        if newname.is_empty() {
                            if let Some(tx) = client_tx_by_id(clients, client_id).await {
                                let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: "Usage: /name <new_name>".to_string() }).unwrap()));
                            }
                            return;
                        }
                        let unique_name = make_unique_name(clients, newname).await;

                        // set name under lock, capture room afterwards
                        {
                            let mut locked = clients.lock().await;
                            if let Some(c) = locked.get_mut(client_id) {
                                c.name = unique_name.clone();
                            }
                        }

                        let room = {
                            let locked = clients.lock().await;
                            locked.get(client_id).map(|c| c.room.clone()).unwrap_or_else(|| "lobby".to_string())
                        };

                        send_system_to_room(clients, histories, &room, &format!("-- {} is now known as {} --", client_name_by_id(clients, client_id).await, unique_name)).await;
                        send_user_list_to_room(clients, &room).await;
                        if let Some(tx) = client_tx_by_id(clients, client_id).await {
                            let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: format!("Your name is now '{}'", unique_name) }).unwrap()));
                        }
                    }
                }
                "/list" => {
                    // list users in current room
                    let room = {
                        let locked = clients.lock().await;
                        locked.get(client_id).map(|c| c.room.clone()).unwrap_or_else(|| "lobby".to_string())
                    };
                    send_user_list_to_room(clients, &room).await;
                }
                "/register" => {
                    if let Some(username) = parts.next() {
                        if let Some(password) = parts.next() {
                            match register_user(users, username.trim(), password.trim()).await {
                                Ok(_) => {
                                    if let Some(tx) = client_tx_by_id(clients, client_id).await {
                                        let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: format!("Registered '{}'. Use /login to authenticate or /name to set your display name.", username.trim()) }).unwrap()));
                                    }
                                }
                                Err(e) => {
                                    if let Some(tx) = client_tx_by_id(clients, client_id).await {
                                        let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: format!("Register failed: {}", e) }).unwrap()));
                                    }
                                }
                            }
                        }
                    }
                }
                "/login" => {
                    if let Some(username) = parts.next() {
                        if let Some(password) = parts.next() {
                            if verify_login(users, username.trim(), password.trim()).await {
                                // set client name to username
                                let unique_name = make_unique_name(clients, username.trim()).await;
                                {
                                    let mut locked = clients.lock().await;
                                    if let Some(c) = locked.get_mut(client_id) {
                                        c.name = unique_name.clone();
                                        c.logged_in = true;
                                    }
                                }

                                // announce in current room
                                let room = {
                                    let locked = clients.lock().await;
                                    locked.get(client_id).map(|c| c.room.clone()).unwrap_or_else(|| "lobby".to_string())
                                };
                                send_system_to_room(clients, histories, &room, &format!("-- {} logged in --", unique_name)).await;
                                send_user_list_to_room(clients, &room).await;
                                if let Some(tx) = client_tx_by_id(clients, client_id).await {
                                    let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: format!("Logged in as '{}'", unique_name) }).unwrap()));
                                }
                            } else {
                                if let Some(tx) = client_tx_by_id(clients, client_id).await {
                                    let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: "Login failed: invalid credentials".to_string() }).unwrap()));
                                }
                            }
                        }
                    }
                }
                "/history" => {
                    // return room history
                    if let Some(tx) = client_tx_by_id(clients, client_id).await {
                        let room = {
                            let locked = clients.lock().await;
                            locked.get(client_id).map(|c| c.room.clone()).unwrap_or_else(|| "lobby".to_string())
                        };
                        send_history_to_client_room(&tx, histories, &room).await;
                    }
                }
                "/msg" => {
                    if let Some(target) = parts.next() {
                        if let Some(text) = parts.next() {
                            let target_name = target.trim().to_lowercase();
                            // target should be in same room
                            let maybe_tx = {
                                let locked = clients.lock().await;
                                let current_room = locked.get(client_id).map(|x| x.room.clone()).unwrap_or_else(|| "lobby".to_string());
                                locked.values()
                                    .find(|c| c.name.to_lowercase() == target_name && c.room == current_room)
                                    .map(|c| c.tx.clone())
                            };
                            if let Some(tx) = maybe_tx {
                                let from = client_name_by_id(clients, client_id).await;
                                let item = Outgoing::Msg { from: from.clone(), text: text.to_string(), ts: now_ts() };
                                if let Ok(s) = serde_json::to_string(&item) {
                                    let _ = tx.send(warp::ws::Message::text(s));
                                }
                                // store in history for room
                                let room = {
                                    let locked = clients.lock().await;
                                    locked.get(client_id).map(|c| c.room.clone()).unwrap_or_else(|| "lobby".to_string())
                                };
                                let mut locked_h = histories.lock().await;
                                let q = locked_h.entry(room).or_insert_with(|| VecDeque::with_capacity(200));
                                q.push_back(HistoryItem { from: format!("(private) {} -> {}", from, target.trim()), text: text.to_string(), ts: now_ts() });
                                while q.len() > 200 { q.pop_front(); }
                            } else {
                                if let Some(tx) = client_tx_by_id(clients, client_id).await {
                                    let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: format!("User '{}' not found in your room", target.trim()) }).unwrap()));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {
            if let Some(tx) = client_tx_by_id(clients, client_id).await {
                let _ = tx.send(warp::ws::Message::text(serde_json::to_string(&Outgoing::System { text: "Unknown command".to_string() }).unwrap()));
            }
        }
    }
}

async fn handle_message_with_rooms(client_id: &str, text: &str, clients: &Clients, histories: &Histories) {
    let from = client_name_by_id(clients, client_id).await;
    let room = {
        let locked = clients.lock().await;
        locked.get(client_id).map(|c| c.room.clone()).unwrap_or_else(|| "lobby".to_string())
    };
    let item = HistoryItem { from: from.clone(), text: text.to_string(), ts: now_ts() };
    broadcast_to_room_and_store(clients, histories, &room, item).await;
}

async fn client_name_by_id(clients: &Clients, id: &str) -> String {
    let locked = clients.lock().await;
    locked.get(id).map(|c| c.name.clone()).unwrap_or_else(|| id.to_string())
}

async fn client_tx_by_id(clients: &Clients, id: &str) -> Option<Tx> {
    let locked = clients.lock().await;
    locked.get(id).map(|c| c.tx.clone())
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Load `users.json` (synchronous, done at startup)
fn load_users() -> anyhow::Result<HashMap<String, String>> {
    let path = "users.json";
    if Path::new(path).exists() {
        let s = fs::read_to_string(path)?;
        let m: HashMap<String, String> = serde_json::from_str(&s)?;
        Ok(m)
    } else {
        Ok(HashMap::new())
    }
}

/// Save users map to disk (async-friendly via spawn_blocking)
async fn save_users_async(map: &HashMap<String, String>) -> anyhow::Result<()> {
    let m = map.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let s = serde_json::to_string_pretty(&m)?;
        fs::write("users.json", s)?;
        Ok(())
    })
    .await??;
    Ok(())
}

/// Hash password with username salt (SHA256). Not for production.
fn hash_password(username: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(username.as_bytes());
    hasher.update(b":");
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Register a new user (async). Returns Err on duplicate or save error.
async fn register_user(users: &Users, username: &str, password: &str) -> Result<(), String> {
    let mut locked = users.lock().await;
    if locked.contains_key(username) {
        return Err("username already exists".into());
    }
    let hash = hash_password(username, password);
    locked.insert(username.to_string(), hash);
    // Save to disk without blocking
    if let Err(e) = save_users_async(&*locked).await {
        return Err(format!("failed to save users: {}", e));
    }
    Ok(())
}

/// Verify login (async)
async fn verify_login(users: &Users, username: &str, password: &str) -> bool {
    let locked = users.lock().await;
    if let Some(stored) = locked.get(username) {
        let h = hash_password(username, password);
        return &h == stored;
    }
    false
}

/// Make a username unique among currently connected clients (case-insensitive)
async fn make_unique_name(clients: &Clients, desired: &str) -> String {
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

async fn handle_upload(form: FormData) -> Result<impl warp::Reply, warp::Rejection> {
    // ensure uploads dir exists
    if let Err(e) = tokio::fs::create_dir_all("uploads").await {
        eprintln!("failed to create uploads dir: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"ok": false, "error": "internal"})),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    // iterate parts, save them; we return info for all files found
    let mut parts = form;
    let mut saved_urls: Vec<serde_json::Value> = Vec::new();

    // Note: Part::stream() yields Results of types implementing bytes::Buf
    while let Some(part_result) = parts.try_next().await.map_err(|e| {
        eprintln!("multipart error: {}", e);
        warp::reject::reject()
    })? {
        let part: Part = part_result;

        // grab filename into an owned String so we don't hold a borrow into `part`
        let filename_opt: Option<String> = part.filename().map(|s| s.to_string());

        // only handle file parts (ignore form fields without filename)
        if let Some(filename) = filename_opt {
            // sanitize filename (simple approach)
            let safe_name = filename.replace(|c: char| {
                !(c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
            }, "_");
            let id = Uuid::new_v4().to_string();
            let stored_name = format!("{}_{}", id, safe_name);
            let path = format!("uploads/{}", stored_name);

            // open file for writing
            let mut file = tokio::fs::File::create(&path).await.map_err(|e| {
                eprintln!("create file error: {}", e);
                warp::reject::reject()
            })?;

            // NOW we can move `part` (call stream) because we no longer borrow from it
            let mut stream = part.stream();

            // write body chunks; each chunk is a Buf; extract bytes via .chunk()
            while let Some(chunk_res) = stream.next().await {
                let mut buf = chunk_res.map_err(|e| {
                    eprintln!("chunk error: {}", e);
                    warp::reject::reject()
                })?;
                while buf.has_remaining() {
                    let bytes = buf.chunk();
                    if !bytes.is_empty() {
                        file.write_all(bytes).await.map_err(|e| {
                            eprintln!("write error: {}", e);
                            warp::reject::reject()
                        })?;
                        let n = bytes.len();
                        buf.advance(n);
                    } else {
                        break;
                    }
                }
            }

            // build public URL (served at /uploads/<stored_name>)
            let meta_len = file.metadata().await.map(|m| m.len()).unwrap_or(0);
            let url = format!("/uploads/{}", stored_name);
            saved_urls.push(json!({ "filename": filename, "url": url, "size": meta_len }));
        }
    }

    // return a JSON array of saved files (or error)
    if saved_urls.is_empty() {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({"ok": false, "files": []})),
            StatusCode::BAD_REQUEST,
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({"ok": true, "files": saved_urls})),
            StatusCode::OK,
        ))
    }
}
