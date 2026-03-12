# Overview

ChatViaLan is a real-time, LAN-local chat server written in Rust. It requires no internet connection, no cloud account, and no external services. Any device on the same network can connect through a browser.

The server is built on [Warp](https://github.com/seanmonstar/warp) and [Tokio](https://tokio.rs/), using WebSockets for bidirectional communication. The frontend is plain HTML, CSS, and JavaScript with no frameworks.

---

## Building and Running

**Prerequisites:** Rust 1.75 or newer.

```bash
git clone <repo-url>
cd ChatViaLan
cargo run --release
```

The server starts on port `8080`. Open `http://localhost:8080` in a browser. On the same LAN, other devices can connect via the IP address printed in the terminal, or by scanning the QR code.

---

## First Use

When the server starts, five default rooms are created: `lobby`, `general`, `random`, `tech`, and `music`. Every new client is placed in `lobby` until they join another room.

Clients must set a name before they can send messages. They can do this with:

- `/name <username>` — use as a guest (no password)
- `/register <username> <password>` — create a permanent account
- `/login <username> <password>` — log in to an existing account

After setting a name, the client can chat and use any of the available commands. See [commands.md](commands.md) for the full list.

---

## Runtime Files

The following files are generated at runtime in the project root:

| File | Description |
|------|-------------|
| `users.json` | Registered user accounts (username to bcrypt hash). |
| `history.json` | Persisted room message history. |
| `private_history.json` | Persisted direct message history. |
| `uploads/` | Files uploaded by clients. |

These files are written periodically (every 5 minutes for history) and on graceful shutdown.

---

## Further Reading

- [Architecture](architecture.md) — Components, state management, and data flow.
- [API Reference](api.md) — HTTP endpoints and WebSocket message protocol.
- [Commands Reference](commands.md) — All chat commands and their behavior.
- [Backend Modules](modules/) — Per-module documentation for each Rust source file.
- [Frontend](frontend.md) — Frontend JavaScript and CSS structure.
