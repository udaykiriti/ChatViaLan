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
mod metrics;

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;
use warp::Filter;
use tracing::{info, warn};

use crate::types::{Clients, Histories, PrivateHistories, Users};
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
    let private_histories: PrivateHistories = Arc::new(RwLock::new(HashMap::new()));

    // Initialize server metrics
    let server_metrics = Arc::new(crate::metrics::ServerMetrics::new());

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
    let private_histories_c = private_histories.clone();
    let private_histories_filter = warp::any().map(move || private_histories_c.clone());
    let users_c = users.clone();
    let users_filter = warp::any().map(move || users_c.clone());
    
    // Metrics filter
    let metrics_c = server_metrics.clone();
    let metrics_filter = warp::any().map(move || metrics_c.clone());

    // Health endpoint
    let health_route = warp::path("health")
        .and(warp::get())
        .and(clients_filter.clone())
        .and(metrics_filter.clone())
        .map(|clients: crate::types::Clients, metrics: Arc<crate::metrics::ServerMetrics>| {
            let response = serde_json::json!({
                "status": "ok",
                "uptime": metrics.uptime_secs(),
                "clients": clients.len()
            });
            warp::reply::json(&response)
        });

    // Metrics endpoint
    let metrics_route = warp::path("metrics")
        .and(warp::get())
        .and(clients_filter.clone())
        .and(histories_filter.clone())
        .and(metrics_filter.clone())
        .and_then(|clients: crate::types::Clients, 
                   histories: crate::types::Histories, 
                   metrics: Arc<crate::metrics::ServerMetrics>| async move {
            let room_count = histories.read().await.len();
            let response = serde_json::json!({
                "uptime_seconds": metrics.uptime_secs(),
                "memory_mb": metrics.memory_usage_mb(),
                "total_messages": metrics.get_total_messages(),
                "active_clients": clients.len(),
                "active_rooms": room_count,
                "total_connections": metrics.get_total_connections()
            });
            Ok::<_, warp::Rejection>(warp::reply::json(&response))
        });

    // WebSocket route
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(clients_filter)
        .and(histories_filter)
        .and(private_histories_filter)
        .and(users_filter)
        .and(metrics_filter)
        .map(|ws: warp::ws::Ws, remote, clients, histories, private_histories, users, metrics| {
            ws.on_upgrade(move |socket| client_connected(socket, remote, clients, histories, private_histories, users, metrics))
        });

    // Static file routes
    let index_route = warp::path::end().and(warp::fs::file("static/index.html"));
    let static_route = warp::fs::dir("static");
    let uploads_route = warp::path("uploads").and(warp::fs::dir("uploads"));

    // File upload route
    let upload_route = warp::path("upload")
        .and(warp::post())
        // 5 GB limit
        .and(warp::multipart::form().max_length(5_368_709_120))
        .and_then(handle_upload);

    // Combine routes
    let routes = ws_route
        .or(health_route)
        .or(metrics_route)
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
    
    // Start mDNS Responder (for LAN discovery)
    // Start mDNS Responder (for LAN discovery)
    let mdns = mdns_sd::ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let service_type = "_http._tcp.local.";
    let instance_name = "RustChat";
    let hostname = "rustchat.local.";
    
    // Minimal properties for iOS
    let mut txt_props = HashMap::new();
    txt_props.insert("path".to_string(), "/".to_string());

    let service_info = mdns_sd::ServiceInfo::new(
        service_type,
        instance_name,
        hostname,
        "", 
        port,
        txt_props,
    ).expect("valid service info");
    
    // Register and keep alive
    mdns.register(service_info).expect("Failed to register mDNS service");
    info!("mDNS registered: {}.{}", instance_name, service_type);
    println!("Broadcast (mDNS): http://rustchat.local:{}", port);
    println!("(Note: If mDNS fails, check local firewall: UDP 5353 must be open)");

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

    // Print LAN Connection Info (QR Code)
    if let Ok(ip) = local_ip_address::local_ip() {
        let address = format!("http://{}:{}", ip, port);
        println!("\n\x1b[32m========================================\x1b[0m");
        println!("LAN CONNECT: Scan this to join!");
        println!("\x1b[32m========================================\x1b[0m");
        qr2term::print_qr(address.clone()).unwrap();
        println!("Server URL: {}\n", address);
    } else {
        println!("Could not detect local IP. Use http://localhost:{}", port);
    }

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
