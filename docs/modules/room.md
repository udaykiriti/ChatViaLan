# Module: room.rs

**Role:** Room membership, message broadcasting, history management, and message-level operations (reactions, edits, deletes).

---

## Broadcasting Functions

### send_system_to_room

```rust
pub async fn send_system_to_room(room: &str, text: &str, clients: &Clients, histories: &Histories)
```

Sends a `System` message to all clients in `room` and appends a `HistoryItem` to that room's history. Used for join/leave announcements and server-generated notices.

---

### send_user_list_to_room

```rust
pub async fn send_user_list_to_room(room: &str, clients: &Clients)
```

Collects the display names of all clients currently in `room` and sends a `List` message to each of them.

---

### send_history_to_client_room

```rust
pub async fn send_history_to_client_room(client_id: &str, clients: &Clients, histories: &Histories)
```

Sends a `History` message to one specific client containing all stored messages for that client's current room. Deleted messages are included with `deleted: true` so the frontend can render them as removed.

---

### broadcast_to_room_and_store

```rust
pub async fn broadcast_to_room_and_store(
    room: &str,
    from: &str,
    text: &str,
    clients: &Clients,
    histories: &Histories,
    metrics: &Metrics,
) -> String
```

The primary message broadcast function. Performs the following:
1. Generates a message ID (first 8 characters of a UUID v4).
2. Records the current Unix timestamp.
3. Constructs a `HistoryItem` and pushes it to the room's `VecDeque`. If the deque exceeds 200 entries, the oldest is removed.
4. Sends a `Msg` outgoing message to all clients in the room.
5. Calls `metrics.increment_messages()`.
6. Scans the text for `@word` patterns and sends `Mention` messages to matching connected clients.

Returns the generated message ID.

---

### broadcast_status

```rust
pub async fn broadcast_status(user: &str, status: &str, clients: &Clients)
```

Sends a `Status` message to **all** connected clients regardless of room. Used by the idle detection task in `main.rs`.

---

### broadcast_read_receipt

```rust
pub async fn broadcast_read_receipt(
    client_id: &str,
    last_msg_id: &str,
    clients: &Clients,
)
```

Sends a `ReadReceipt` message to all clients in the same room as `client_id`. Updates `client.last_read_msg_id`.

---

## Message Operations

### add_reaction

```rust
pub async fn add_reaction(
    client_id: &str,
    msg_id: &str,
    emoji: &str,
    clients: &Clients,
    histories: &Histories,
)
```

Locates the message by `msg_id` in the room's history. If the client's name is already in the emoji's reaction list, it is removed (toggle). Otherwise it is added. Broadcasts a `Reaction` message to the room.

---

### edit_message

```rust
pub async fn edit_message(
    client_id: &str,
    msg_id: &str,
    new_text: &str,
    clients: &Clients,
    histories: &Histories,
)
```

Finds the message in history. Only proceeds if `history_item.from == client.name` (owner-only). Sets `edited = true` and updates `text`. Broadcasts an `Edit` message to the room.

---

### delete_message

```rust
pub async fn delete_message(
    client_id: &str,
    msg_id: &str,
    clients: &Clients,
    histories: &Histories,
)
```

Finds the message in history. Only proceeds if `history_item.from == client.name`. Sets `deleted = true`. Broadcasts a `Delete` message to the room. The message entry remains in history with its content preserved internally (the frontend is responsible for hiding it).

---

## Room Switching

### join_room

```rust
pub async fn join_room(
    client_id: &str,
    new_room: &str,
    clients: &Clients,
    histories: &Histories,
)
```

1. Reads the client's current room and broadcasts a leave system message to it.
2. If `new_room` does not exist in `Histories`, inserts an empty `VecDeque` for it.
3. Updates `client.room` to `new_room`.
4. Broadcasts a join system message to `new_room`.
5. Calls `send_history_to_client_room` to deliver existing messages.

---

## Persistence

### save_history / load_history

```rust
pub async fn save_history(histories: &HashMap<String, VecDeque<HistoryItem>>)
pub async fn load_history() -> HashMap<String, VecDeque<HistoryItem>>
```

Serialize and deserialize `history.json` using `serde_json`. `load_history` returns an empty map on any error.

---

### save_private_history / load_private_history

```rust
pub async fn save_private_history(histories: &HashMap<String, VecDeque<HistoryItem>>)
pub async fn load_private_history() -> HashMap<String, VecDeque<HistoryItem>>
```

Same as above but for `private_history.json`. The key format is two lowercased usernames sorted alphabetically and joined by a comma (e.g., `"alice,bob"`).

---

## History Cap

Each room's history is stored in a `VecDeque<HistoryItem>`. When a new message is added via `broadcast_to_room_and_store`, if the length exceeds **200**, the front entry (oldest) is popped. This cap applies only to in-memory history; on-disk JSON reflects whatever is in memory at save time.
