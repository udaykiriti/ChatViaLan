# Module: typing.rs

**Role:** Manages per-client typing state and broadcasts typing indicators to the room.

---

## Functions

### set_typing_status

```rust
pub fn set_typing_status(client_id: &str, is_typing: bool, clients: &Clients)
```

Updates the `is_typing` field on the `Client` identified by `client_id`. If the client is not found the call is a no-op.

---

### broadcast_typing_status

```rust
pub async fn broadcast_typing_status(client_id: &str, clients: &Clients)
```

Determines the room of the client identified by `client_id`, then collects the names of all clients in that room whose `is_typing` is `true`. Sends a `Typing { users }` message to every client in the room.

This function is called:
- When a `Typing` message arrives from a client (in `client.rs`).
- After a client disconnects, to clear them from any active typing display.

---

## Behavior Notes

- Typing state is stored per connection, not per username. If a user connects from two tabs, each tab tracks its own state independently.
- There is no server-side timeout that clears the typing flag. The client is responsible for sending `is_typing: false` when the user stops typing (e.g., after sending a message or after an inactivity timer on the frontend).
