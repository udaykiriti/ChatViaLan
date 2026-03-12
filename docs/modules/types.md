# Module: types.rs

**Role:** Defines all shared data structures, type aliases, and message formats used throughout the application.

---

## Type Aliases

| Alias | Underlying Type | Description |
|-------|----------------|-------------|
| `Tx` | `UnboundedSender<Message>` | Outbound channel for sending WebSocket messages to one client. |
| `Clients` | `Arc<DashMap<String, Client>>` | Registry of all active connections, keyed by UUID. |
| `Histories` | `Arc<RwLock<HashMap<String, VecDeque<HistoryItem>>>>` | Room message history, keyed by room name. |
| `PrivateHistories` | `Arc<RwLock<HashMap<String, VecDeque<HistoryItem>>>>` | Direct message history, keyed by sorted username pair. |
| `Users` | `Arc<DashMap<String, String>>` | Registered accounts, keyed by username (value is bcrypt hash). |

---

## Client

```rust
pub struct Client {
    pub name: String,
    pub tx: Tx,
    pub logged_in: bool,
    pub room: String,
    pub last_message_times: Vec<Instant>,
    pub is_typing: bool,
    pub last_read_msg_id: Option<String>,
    pub last_active: Instant,
}
```

Holds all per-connection state. Stored in `Clients` and accessed exclusively by the owning connection task (except for reads from other tasks broadcasting to the room).

---

## HistoryItem

```rust
pub struct HistoryItem {
    pub id: String,
    pub from: String,
    pub text: String,
    pub ts: u64,
    pub reactions: HashMap<String, Vec<String>>,
    pub edited: bool,
    pub deleted: bool,
}
```

One persisted message entry. `reactions` maps emoji names to lists of usernames who reacted. `deleted` is a soft-delete flag — the item is kept but the frontend should not display its content.

---

## RoomInfo

```rust
pub struct RoomInfo {
    pub name: String,
    pub members: usize,
}
```

Used in `RoomList` outgoing messages to describe a room and its current member count.

---

## AppState

A bundle of all shared state handles passed to route handlers and background tasks. Contains:

- `clients: Clients`
- `histories: Histories`
- `private_histories: PrivateHistories`
- `users: Users`
- `metrics: Arc<Metrics>`

Cloned cheaply (all fields are `Arc`-wrapped).

---

## OutgoingMessage

Tagged enum serialized with `serde`. The `type` field is the variant name. Sent from server to client.

| Variant | Additional Fields | Description |
|---------|------------------|-------------|
| `System` | `text` | Server notice or error. |
| `Msg` | `id`, `from`, `text`, `ts`, `reactions`, `edited` | Chat message. |
| `History` | `items` | Bulk history on room join. |
| `List` | `users` | User list for the current room. |
| `RoomList` | `rooms` | All rooms with member counts. |
| `Typing` | `users` | Users currently typing. |
| `Reaction` | `msg_id`, `emoji`, `user`, `added` | Reaction added or removed. |
| `Edit` | `msg_id`, `new_text` | Message was edited. |
| `Delete` | `msg_id` | Message was deleted. |
| `ReadReceipt` | `user`, `last_msg_id` | Read acknowledgment. |
| `Mention` | `from`, `text`, `mentioned` | Direct mention notification. |
| `Status` | `user`, `status` | Presence status changed. |
| `LinkPreview` | `url`, `title`, `description`, `image` | Open Graph preview for a URL. |
| `Nudge` | `from` | Screen-shake/sound effect trigger. |

---

## IncomingMessage

Tagged enum deserialized from JSON. The `type` field is the variant name. Sent from client to server.

| Variant | Additional Fields | Description |
|---------|------------------|-------------|
| `Cmd` | `cmd` | Slash command string. |
| `Msg` | `text` | Chat message text. |
| `Typing` | `is_typing` | Typing state update. |
| `React` | `msg_id`, `emoji` | Add or toggle a reaction. |
| `Edit` | `msg_id`, `new_text` | Edit a message. |
| `Delete` | `msg_id` | Delete a message. |
| `MarkRead` | `last_msg_id` | Mark a message as read. |
