//! Rust Chat Server — Real-Time WebSocket Chat
//!
//! Modular structure:
//! - types.rs: Core data structures
//! - auth.rs: User authentication
//! - room.rs: Room management
//! - commands.rs: Command handling
//! - client.rs: WebSocket client lifecycle
//! - helpers.rs: Client helper functions
//! - rate_limit.rs: Rate limiting
//! - typing.rs: Typing indicators
//! - upload.rs: File uploads

mod types;
mod auth;
mod room;
mod commands;
mod client;
mod helpers;
mod rate_limit;
mod typing;
mod upload;

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

use crate::types::{Clients, Histories, Users};
use crate::auth::load_users;
use crate::client::client_connected;
use crate::upload::handle_upload;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load users from disk
    let users_map = load_users().unwrap_or_default();
    let users: Users = Arc::new(Mutex::new(users_map));

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let histories: Histories = Arc::new(Mutex::new(HashMap::new()));

    // Ensure default "lobby" room exists
    {
        let mut h = histories.lock().await;
        h.entry("lobby".to_string())
            .or_insert_with(|| VecDeque::with_capacity(200));
    }

    // Warp filters for shared state
    let clients_filter = warp::any().map(move || clients.clone());
    let histories_filter = warp::any().map(move || histories.clone());
    let users_filter = warp::any().map(move || users.clone());

    // WebSocket route
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(clients_filter)
        .and(histories_filter)
        .and(users_filter)
        .map(|ws: warp::ws::Ws, remote, clients, histories, users| {
            ws.on_upgrade(move |socket| client_connected(socket, remote, clients, histories, users))
        });

    // Static file routes
    let index_route = warp::path::end().and(warp::fs::file("static/index.html"));
    let static_route = warp::fs::dir("static");
    let uploads_route = warp::path("uploads").and(warp::fs::dir("uploads"));

    // File upload route
    let upload_route = warp::path("upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(599_368_709))
        .and_then(handle_upload);

    // Combine routes
    let routes = ws_route
        .or(index_route)
        .or(static_route)
        .or(uploads_route)
        .or(upload_route)
        .with(warp::cors().allow_any_origin());

    // Ensure uploads directory exists
    if let Err(e) = std::fs::create_dir_all("uploads") {
        eprintln!("warning: failed to create uploads dir: {}", e);
    }

    // Get port from environment or default to 8080
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    println!("Server running at http://0.0.0.0:{}/", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}
