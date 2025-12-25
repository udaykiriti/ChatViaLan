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
use tokio::sync::RwLock;
use dashmap::DashMap;
use warp::Filter;
use tracing::{info, warn};

use crate::types::{Clients, Histories, Users};
use crate::auth::load_users;
use crate::client::client_connected;
use crate::upload::handle_upload;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    info!("Starting Rust Chat Server...");

    // Load users from disk
    let users_map = load_users().unwrap_or_default();
    let users: Users = Arc::new(dashmap::DashMap::from_iter(users_map));

    let clients: Clients = Arc::new(DashMap::new());
    let histories: Histories = Arc::new(RwLock::new(HashMap::new()));

    // Load history from disk
    crate::room::load_history(&histories).await;

    // Ensure persistent rooms exist
    {
        let mut h = histories.write().await;
        for room in &["lobby", "general", "random", "tech", "music"] {
            h.entry(room.to_string())
                .or_insert_with(|| VecDeque::with_capacity(200));
        }
    }

    // Warp filters for shared state
    let clients_c = clients.clone();
    let clients_filter = warp::any().map(move || clients_c.clone());
    let histories_c = histories.clone();
    let histories_filter = warp::any().map(move || histories_c.clone());
    let users_c = users.clone();
    let users_filter = warp::any().map(move || users_c.clone());

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
        warn!("failed to create uploads dir: {}", e);
    }

    // Get port from environment or default to 8080
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // Background task for idle detection
    let clients_idle = clients.clone();
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        let mut statuses: HashMap<String, String> = HashMap::new(); // client_id -> status

        loop {
            interval.tick().await;
            let now = std::time::Instant::now();
            let mut updates = Vec::new();

            for r in clients_idle.iter() {
                let client_id = r.key().clone();
                let client = r.value();
                let last_active = client.last_active;
                let diff = now.duration_since(last_active);

                let current_status = if diff.as_secs() > 300 { // 5 minutes
                    "idle"
                } else {
                    "active"
                };

                let prev_status = statuses.get(&client_id).map(|s| s.as_str()).unwrap_or("active");
                if current_status != prev_status {
                    updates.push((client.name.clone(), current_status.to_string()));
                    statuses.insert(client_id, current_status.to_string());
                }
            }

            for (name, status) in updates {
                crate::room::broadcast_status(&clients_idle, &name, &status).await;
            }
            
            // Cleanup statuses for disconnected clients
            statuses.retain(|id, _| clients_idle.contains_key(id));
        }
    });

    // Background task for periodic history saving (every 5 minutes)
    let histories_saver = histories.clone();
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            crate::room::save_history(&histories_saver).await;
        }
    });

    info!("Server running at http://0.0.0.0:{}/", port);
    
    // Server task
    let server = warp::serve(routes).run(([0, 0, 0, 0], port));

    // Graceful shutdown with signal handling
    tokio::select! {
        _ = server => {
            info!("Server process finished");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received. Cleaning up...");
            
            // Save history
            crate::room::save_history(&histories).await;

            // Force save users before exit
            let users_map: HashMap<String, String> = users.iter()
                .map(|r| (r.key().clone(), r.value().clone()))
                .collect();
            if let Err(e) = crate::auth::save_users_async(users_map).await {
                warn!("failed to save users on shutdown: {}", e);
            }
            info!("Shutdown complete.");
        }
    }

    Ok(())
}
