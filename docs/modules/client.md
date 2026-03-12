# Module: client.rs

**Role:** WebSocket connection lifecycle — authentication phase, message dispatch, and cleanup on disconnect.

---

## Entry Point

### handle_ws_client

```rust
pub async fn handle_ws_client(ws: WebSocket, state: AppState)
```

Called by Warp for each WebSocket upgrade. Splits the socket into a sender and receiver. Spawns a task to forward outbound MPSC channel messages to the WebSocket sender. Runs the authentication phase followed by the main message loop.

---

## Authentication Phase

Before a client can send chat messages, it must set an identity. The server reads incoming messages and expects one of:

| Input | Action |
|-------|--------|
| `/name <username>` | Sets the display name; client enters as guest. |
| `/register <username> <password>` | Creates account and sets name. |
| `/login <username> <password>` | Authenticates and sets name. |

All other messages during this phase are rejected with a system message prompting the client to identify first.

Once a name is accepted:
- A `Client` entry is inserted into the `Clients` map.
- The client is placed in `lobby`.
- Room history is sent via `send_history_to_client_room`.
- A join announcement is broadcast to the room.
- Connection metrics are incremented.

---

## Main Message Loop

After authentication, the loop processes `IncomingMessage` values:

| Type | Handler |
|------|---------|
| `Cmd` | `commands::handle_command` |
| `Msg` | Rate-limit check, profanity censor, broadcast, optional link preview |
| `Typing` | `typing::set_typing_status` + `typing::broadcast_typing_status` |
| `React` | `room::add_reaction` |
| `Edit` | `room::edit_message` |
| `Delete` | `room::delete_message` |
| `MarkRead` | `room::broadcast_read_receipt` |

For `Msg`, the processing steps are:
1. `rate_limit::check_rate_limit` — drop message and warn client if over limit.
2. `helpers::censor_profanity` — replace banned words.
3. `room::broadcast_to_room_and_store` — broadcast to room and persist.
4. Extract any URL from the message text and, if found, call `helpers::fetch_preview` asynchronously; if a preview is returned, send a `LinkPreview` message to the room.

---

## Disconnect Cleanup

When the WebSocket stream ends (client closes tab, network drops, etc.):

1. The client's entry is removed from `Clients`.
2. `typing::broadcast_typing_status` is called to remove the client from any active typing indicator.
3. A leave announcement is sent to the room via `room::send_system_to_room`.

---

## Client Struct (defined in types.rs)

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Display name. |
| `tx` | `Tx` | MPSC sender for outbound messages. |
| `logged_in` | `bool` | Whether the client authenticated with a password. |
| `room` | `String` | Current room name. |
| `last_message_times` | `Vec<Instant>` | Timestamps used by the rate limiter. |
| `is_typing` | `bool` | Current typing state. |
| `last_read_msg_id` | `Option<String>` | Last acknowledged message ID. |
| `last_active` | `Instant` | Updated on every message; used for idle detection. |
