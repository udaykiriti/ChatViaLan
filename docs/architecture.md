# Architecture

---

## Components

The application is composed of a Rust backend and a browser-based frontend that communicate exclusively over WebSockets.

```
Browser (HTML/CSS/JS)
        |
        | WebSocket (JSON messages)
        |
Warp HTTP server (main.rs)
        |
   +-----------+
   |           |
Client     Room
handler    manager
(client.rs) (room.rs)
   |
   +-- Auth (auth.rs)
   +-- Commands (commands.rs)
   +-- Rate Limit (rate_limit.rs)
   +-- Typing (typing.rs)
   +-- Upload (upload.rs)
   +-- Helpers (helpers.rs)
   +-- Metrics (metrics.rs)
```

---

## Shared State

All state is held in memory and shared across async tasks using thread-safe wrappers. The following types are defined in `types.rs` and passed through the application as `Arc`-wrapped values:

| Type | Backing Type | Description |
|------|-------------|-------------|
| `Clients` | `Arc<DashMap<String, Client>>` | Active WebSocket connections keyed by UUID. |
| `Histories` | `Arc<RwLock<HashMap<String, VecDeque<HistoryItem>>>>` | Room message history keyed by room name. |
| `PrivateHistories` | `Arc<RwLock<HashMap<String, VecDeque<HistoryItem>>>>` | DM history keyed by sorted username pair. |
| `Users` | `Arc<DashMap<String, String>>` | Registered users keyed by username (value is bcrypt hash). |

`DashMap` is used for the clients and users maps because they are written to frequently (on connect/disconnect and on registration). `RwLock<HashMap>` is used for history because writes are rare (batched every 5 minutes) while reads happen on every room join.

---

## Message Flow

### Inbound (client to server)

1. The browser sends a JSON-encoded `IncomingMessage` over the WebSocket.
2. The Warp filter deserializes it and routes it to the client's async task in `client.rs`.
3. The client task inspects the message type:
   - `Cmd` — forwarded to `commands.rs`.
   - `Msg` — rate-limited, censored, then broadcast via `room.rs`.
   - `Typing` — handled by `typing.rs`.
   - `React`, `Edit`, `Delete`, `MarkRead` — handled directly in `room.rs`.
4. Each operation sends `OutgoingMessage` values to the relevant clients via MPSC channels.

### Outbound (server to client)

Each `Client` holds an unbounded `tokio::sync::mpsc` sender (`Tx`). The Warp WebSocket task for that client reads from the corresponding receiver and forwards messages to the browser. This decouples message production from the WebSocket write path.

---

## Rooms

Rooms are not stored as explicit structs. Room membership is implicit: a client belongs to a room when `client.room == room_name`. Broadcasting to a room means iterating over all connected clients in `Clients` and filtering by room name.

Five rooms are pre-seeded at startup. New rooms can be created at runtime with `/join <new-room-name>`. Rooms have no lifecycle — they exist as long as at least one client is in them (or until history is cleared).

Room history is capped at 200 messages per room in `VecDeque`.

---

## Persistence

Persistence is handled entirely with JSON flat files via `tokio::fs`. There is no database.

- **User accounts** (`users.json`): Loaded synchronously at startup. Written asynchronously after every registration using `spawn_blocking` for the bcrypt computation.
- **Room history** (`history.json`) and **DM history** (`private_history.json`): Written every 5 minutes by background tasks, and again on graceful shutdown (Ctrl+C).

History is loaded at startup and merged into the in-memory maps before the server accepts connections.

---

## Background Tasks

Three background Tokio tasks run independently of the request loop:

| Task | Interval | Purpose |
|------|----------|---------|
| Idle detection | 30 seconds | Marks clients as `idle` after 5 minutes of inactivity; broadcasts status updates. |
| History save | 5 minutes | Serializes room history to `history.json`. |
| Private history save | 5 minutes | Serializes DM history to `private_history.json`. |

---

## LAN Discovery

At startup, the server:

1. Resolves the machine's local IP address using `local-ip-address`.
2. Registers an mDNS service record (`_chat._tcp.local.`) via `mdns-sd`, so clients can reach the server using a `.local` hostname on networks that support mDNS.
3. Prints the IP and port, and renders a QR code in the terminal for quick mobile access.

---

## Concurrency Model

The application uses Tokio's multi-thread runtime. Each WebSocket connection runs in its own async task. Shared state is never locked for long periods — `DashMap` provides sharded locking, and history writes are short-lived bulk operations. The rate limiter and typing state are per-client and require no synchronization.
