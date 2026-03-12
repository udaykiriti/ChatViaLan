# Module: commands.rs

**Role:** Parses and executes all `/` commands sent by clients.

---

## Entry Point

### handle_command

```rust
pub async fn handle_command(cmd: &str, client_id: &str, state: &AppState) -> Result<()>
```

Receives the full command string (e.g., `"/join tech"`), splits it into tokens, and dispatches to the appropriate handler based on the first token.

Unknown commands result in a system message listing available commands.

---

## Command Implementations

### /name

```
/name <username>
```

Changes the client's display name. Calls `helpers::make_unique_name` to append a numeric suffix if the name is taken. Updates the entry in `Clients`. Does not require the client to be logged in.

---

### /register

```
/register <username> <password>
```

Calls `auth::register_user`. On success, updates the client's name and sets `logged_in = true`. On failure (e.g., duplicate username), sends an error system message.

---

### /login

```
/login <username> <password>
```

Calls `auth::verify_login`. On success, updates the client's name and sets `logged_in = true`. On failure, sends an error system message.

---

### /join

```
/join <room>
```

Calls `room::join_room`, which:
1. Sends a leave announcement to the old room.
2. Updates `client.room`.
3. Creates the room in `Histories` if it does not exist.
4. Sends a join announcement to the new room.
5. Delivers room history to the client.

---

### /leave

Calls `room::join_room` with `"lobby"` as the target.

---

### /rooms

Collects all room names from `Histories` and the current member count for each by scanning `Clients`. Sends a `RoomList` message to the requesting client.

---

### /room

Reads `client.room` and sends it as a system message.

---

### /msg

```
/msg <username> <text>
```

Sends a direct message to a named user (if connected). The DM key is computed by sorting both usernames case-insensitively and joining with a comma (e.g., `"alice,bob"`). The message is stored in `PrivateHistories` and sent to the recipient via their `Tx` channel.

---

### /who

Lists clients in the current room with an indicator for whether each is a registered account or a guest.

---

### /list

Calls `room::send_user_list_to_room`, broadcasting a `List` message to the entire room.

---

### /history

Calls `room::send_history_to_client_room` for the requesting client.

---

### /nudge

Broadcasts a `Nudge` message to all clients in the current room.

---

### /kick

```
/kick <username>
```

Admin-only (requires `logged_in == true`). Sends a `System` disconnect message to the target client, then closes their connection by dropping their `Tx` channel. The target client will be cleaned up on the next iteration of their receive loop.

---

### /stats

Reads from the `Metrics` struct and from `Clients`/`Histories` to produce a summary system message including: active connections, room count, total messages sent, and approximate memory usage.

---

### /help

Sends a formatted list of available commands as a system message.

---

## Mention Detection

After a `Msg` is broadcast (in `client.rs`), the message text is scanned for `@word` patterns using a compiled regex. For each match, if a connected client with that username exists, a `Mention` message is sent directly to that client.
