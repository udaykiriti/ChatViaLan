# Module: helpers.rs

**Role:** Shared utility functions used across the backend.

---

## Functions

### client_name_by_id

```rust
pub fn client_name_by_id(client_id: &str, clients: &Clients) -> Option<String>
```

Looks up a client in the `Clients` map by ID and returns its display name, or `None` if not found.

---

### client_tx_by_id

```rust
pub fn client_tx_by_id(client_id: &str, clients: &Clients) -> Option<Tx>
```

Returns a clone of the client's outbound MPSC sender channel, or `None` if not found. Used whenever a message needs to be sent to a specific client by ID.

---

### now_ts

```rust
pub fn now_ts() -> u64
```

Returns the current Unix timestamp in seconds as a `u64`. Used to populate `ts` on `HistoryItem` entries.

---

### censor_profanity

```rust
pub fn censor_profanity(text: &str) -> String
```

Scans `text` for a predefined list of banned words using a case-insensitive regex and replaces any matches with `"****"`. Returns the modified string.

The word list is compiled into a `Regex` on first call and cached via `once_cell`. To add or remove banned words, edit the word list in this function.

---

### make_unique_name

```rust
pub fn make_unique_name(desired: &str, clients: &Clients, exclude_id: &str) -> String
```

Returns `desired` if no other connected client is using that name. Otherwise appends an incrementing suffix until a free name is found (e.g., `alice`, `alice-1`, `alice-2`). The `exclude_id` parameter skips the calling client's own entry in the map, so a client can re-set its current name without triggering a conflict.

---

### fetch_preview

```rust
pub async fn fetch_preview(url: &str) -> Option<(String, Option<String>, Option<String>)>
```

Fetches the URL using a `reqwest` client with:
- 5-second timeout
- 256 KB response body limit
- Rejects non-HTML content types

Parses the HTML with `scraper` to extract Open Graph metadata:

| Tag | Used for |
|-----|---------|
| `og:title` | Title (falls back to `<title>` element) |
| `og:description` | Description |
| `og:image` | Preview image URL |

Returns `Some((title, description, image))` or `None` if the request fails, times out, or the response is not HTML. `description` and `image` may be `None` independently of each other.

The returned tuple is used by `client.rs` to construct a `LinkPreview` outgoing message.
