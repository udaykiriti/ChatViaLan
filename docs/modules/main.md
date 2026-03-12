# Module: main.rs

**Role:** Application entry point. Initializes state, registers routes, starts background tasks, and handles shutdown.

---

## Startup Sequence

1. Initialize tracing/logging via `tracing_subscriber`.
2. Load user accounts from `users.json` into a `DashMap`.
3. Load room history from `history.json` and private history from `private_history.json`.
4. Construct the shared `AppState` bundle: `Clients`, `Histories`, `PrivateHistories`, `Users`, `Metrics`.
5. Seed five default rooms in the history map: `lobby`, `general`, `random`, `tech`, `music`.
6. Register Warp routes (see below).
7. Resolve the local IP address, register mDNS, and render the QR code.
8. Spawn three background tasks (idle detection, history save, private history save).
9. Start the Warp server on `0.0.0.0:8080` with a Ctrl+C signal handler that saves all data before exiting.

---

## Routes

| Method | Path | Handler |
|--------|------|---------|
| `GET` | `/health` | Inline closure — returns JSON with uptime and client count. |
| `GET` | `/metrics` | Inline closure — returns full metrics JSON. |
| `GET` | `/ws` | `client::handle_ws_client` — WebSocket upgrade. |
| `POST` | `/upload` | `upload::handle_upload` — Multipart file upload (5 GB limit). |
| `GET` | `/uploads/:file` | Static files from `uploads/` directory. |
| `GET` | `/*` | Static files from `static/` directory. |

---

## Background Tasks

### Idle Detection (every 30 seconds)

Iterates all connected clients. Any client whose `last_active` is more than 5 minutes ago has its status broadcast as `idle`. All others are broadcast as `online`. Uses `broadcast_status` from `room.rs`.

### History Save (every 5 minutes)

Acquires a read lock on `Histories` and calls `room::save_history`. Non-blocking from the perspective of message handling.

### Private History Save (every 5 minutes)

Acquires a read lock on `PrivateHistories` and calls `room::save_private_history`.

---

## Graceful Shutdown

A `tokio::signal::ctrl_c()` future is attached to the server via Warp's `with_graceful_shutdown`. On receipt:

1. Both history maps are saved to disk.
2. The process exits.

---

## Dependencies Referenced

`auth`, `client`, `room`, `upload`, `types`, `metrics`
