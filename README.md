# Rust Chat — Real-Time WebSocket Chat Server

A fully local, offline-friendly, multi-user chat system built using **Rust**, **Warp**, and **WebSockets**.  
Works 100% offline on LAN/localhost with a modern, clean UI.

---

## Features

### Core
- Real-time WebSocket messaging
- Multiple rooms (`/join`, `/rooms`, `/leave`)
- User accounts (`/register`, `/login`) with guest mode
- Private messaging (`/msg user text`)
- File upload & sharing
- Persistent message history per room
- Works 100% offline (LAN/local)

### Advanced Features
- **Message Reactions** — React with emojis (thumbs up, heart, laugh, etc.)
- **Edit Messages** — Edit your own sent messages
- **Delete Messages** — Delete with confirmation
- **Mentions** — @username highlighting with notifications
- **Read Receipts** — Track who read messages
- **Sound Notifications** — Beep on new messages when tab unfocused
- **Unread Count** — Tab title shows unread count
- **Dark/Light Mode** — Toggle theme with preference saved
- **Typing Indicators** — Shows "X is typing..."
- **Rate Limiting** — Max 5 messages per 10 seconds
- **Relative Timestamps** — "2m ago" format

---

## Project Structure

```
rust-chat/
├── Cargo.toml
├── users.json            # Stored user accounts
├── uploads/              # Uploaded files
├── static/
│   ├── index.html        # Main HTML
│   ├── styles.css        # Clean modern UI
│   ├── app.js            # Frontend logic
│   └── favicon.svg       # App icon
└── src/
    ├── main.rs           # Server setup & routes
    ├── types.rs          # Core data structures
    ├── client.rs         # WebSocket client handling
    ├── commands.rs       # Command processing
    ├── room.rs           # Room management
    ├── auth.rs           # User authentication
    └── upload.rs         # File upload handling
```

---

## Quick Start

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Run the Server
```bash
cargo run --release
```

### 3. Open the Chat
- **Local**: http://localhost:8080
- **LAN**: `http://<your-ip>:8080`

Find your IP:
```bash
ip a          # Linux
ipconfig      # Windows
```

---

## Chat Commands

| Command | Description |
|---------|-------------|
| `/name <name>` | Change display name |
| `/register <user> <pass>` | Create account |
| `/login <user> <pass>` | Log in |
| `/msg <user> <text>` | Private message |
| `/join <room>` | Join/create a room |
| `/rooms` | List all rooms |
| `/leave` | Return to lobby |
| `/who` | List users in room |
| `/help` | Show all commands |

---

## UI Features

| Action | How |
|--------|-----|
| React | Click `+` on message, pick emoji |
| Edit | Hover your message, click edit icon |
| Delete | Hover your message, click delete icon |
| Mention | Type `@username` in message |
| Theme | Click theme toggle in header |
| Upload | Click Attach, then Upload |

---

## File Sharing

Upload files via the UI. Files stored in `uploads/`:
```
http://localhost:8080/uploads/<id>_filename.ext
```

---

## Security

- Passwords hashed with SHA-256 + salt
- Rate limiting prevents spam
- Safe for LAN/local use
- **Not production-ready** — use Argon2 for production

---

## LAN Setup

Allow firewall (Linux):
```bash
sudo firewall-cmd --add-port=8080/tcp --permanent
sudo firewall-cmd --reload
```

Custom port:
```bash
PORT=3000 cargo run --release
```

---

## Tech Stack

- **Backend**: Rust, Warp, Tokio
- **Frontend**: Vanilla JS, CSS (no frameworks)
- **Protocol**: WebSocket
- **Storage**: JSON files (users), in-memory (messages)

---

## License

MIT — Free to use, modify, and redistribute.
