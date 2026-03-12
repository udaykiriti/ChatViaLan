# Module: metrics.rs

**Role:** Tracks server-wide performance counters and exposes them for the `/metrics` and `/stats` endpoints.

---

## Struct: Metrics

```rust
pub struct Metrics {
    start_time: Instant,
    total_messages: AtomicU64,
    total_connections: AtomicUsize,
}
```

All fields use lock-free atomic types or an immutable `Instant`, so the struct can be shared across threads with no synchronization overhead.

---

## Methods

### new

```rust
pub fn new() -> Self
```

Creates a new `Metrics` instance with counters at zero and `start_time` set to now.

---

### uptime_secs

```rust
pub fn uptime_secs(&self) -> u64
```

Returns the number of seconds elapsed since `start_time`.

---

### increment_messages

```rust
pub fn increment_messages(&self)
```

Atomically increments `total_messages` by 1. Called by `room::broadcast_to_room_and_store` for each message stored in history.

---

### increment_connections

```rust
pub fn increment_connections(&self)
```

Atomically increments `total_connections` by 1. Called by `client::handle_ws_client` when a new connection completes authentication.

---

### get_total_messages

```rust
pub fn get_total_messages(&self) -> u64
```

Returns the current value of `total_messages` with relaxed ordering.

---

### get_total_connections

```rust
pub fn get_total_connections(&self) -> usize
```

Returns the current value of `total_connections` with relaxed ordering.

---

### memory_usage_mb

```rust
pub fn memory_usage_mb(&self) -> u64
```

On Linux, reads `/proc/self/statm` and returns the resident set size (RSS) converted to megabytes. The conversion uses the system page size (typically 4 KB).

Returns `0` on non-Linux systems or if the file cannot be read.

---

## Usage

`Metrics` is constructed once in `main.rs` and shared as `Arc<Metrics>`. The counters are purely cumulative — they are never decremented. Active client count is read directly from the `Clients` map rather than being tracked here.
