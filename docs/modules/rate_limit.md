# Module: rate_limit.rs

**Role:** Prevents message flooding by enforcing a per-client send rate.

---

## Policy

Each client is limited to **5 messages per 10-second sliding window**. Attempts beyond this threshold are dropped and the client receives a warning system message.

---

## Function

### check_rate_limit

```rust
pub fn check_rate_limit(client: &mut Client) -> bool
```

Operates on the `last_message_times: Vec<Instant>` field of the `Client` struct.

Steps:
1. Remove all entries older than 10 seconds from `last_message_times`.
2. If the length is 5 or greater, the client is over the limit. Send a system warning via the client's `Tx` and return `false`.
3. Otherwise, push `Instant::now()` and return `true`.

The caller (`client.rs`) must call this before broadcasting any message. If it returns `false`, the message is discarded entirely and not stored in history.

---

## Notes

- The limit is enforced in memory only; it resets if the client disconnects and reconnects.
- The check is synchronous and requires mutable access to the `Client`. This is safe because `client.rs` holds exclusive ownership of the `Client` reference during message processing.
- There is no cooldown message beyond the initial warning. If the client continues to exceed the limit, subsequent attempts are silently dropped until the window slides.
