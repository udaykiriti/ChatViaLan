# Rust Chat — Real-Time WebSocket Chat Server

A fully local, offline-friendly, multi-user chat system built with **Rust**, **Warp**, and **WebSockets**.  
Messenger-style UI with real-time features.

---

## Features

### Core
- Real-time WebSocket messaging
- Multiple rooms with member counts
- User accounts (register/login) + guest mode
- Private messaging
- File upload & sharing
- Persistent message history

### Advanced
- Message reactions (👍 ❤️ 😂 😮 😢 🎉)
- Edit & delete messages
- @mentions with notifications
- Typing indicators
- Sound notifications + unread count
- Dark/light theme
- Rate limiting (5 msg / 10 sec)

### UI
- Messenger-style layout
- Message bubbles (sent right, received left)
- Available rooms with member counts
- Auto-refresh rooms every 10 seconds
- Fully responsive (mobile + desktop)

---

## Project Structure

```
rust-chat/
├── Cargo.toml
├── users.json              # User accounts
├── uploads/                # Uploaded files
├── static/
│   ├── index.html
│   ├── favicon.svg
│   ├── css/
│   │   ├── base.css        # Variables, layout
│   │   ├── sidebar.css     # Sidebar, rooms
│   │   ├── chat.css        # Header, input
│   │   └── messages.css    # Message bubbles
│   └── js/
│       ├── config.js       # Config constants
│       ├── state.js        # App state
│       ├── dom.js          # DOM references
│       ├── utils.js        # Utilities
│       ├── features.js     # Theme, sound, typing
│       ├── reactions.js    # Reactions, edit, delete
│       ├── messages.js     # Message rendering
│       ├── websocket.js    # WebSocket connection
│       ├── events.js       # Event listeners
│       └── main.js         # Entry point
└── src/
    ├── main.rs             # Server setup
    ├── types.rs            # Data structures
    ├── client.rs           # WebSocket handling
    ├── commands.rs         # Command processing
    ├── room.rs             # Room management
    ├── helpers.rs          # Helper functions
    ├── rate_limit.rs       # Rate limiting
    ├── typing.rs           # Typing indicators
    ├── auth.rs             # Authentication
    └── upload.rs           # File uploads
```

---

## Quick Start

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Run server
cargo run --release

# Open browser
# Local: http://localhost:8080
# LAN:   http://<your-ip>:8080
```

---

## Commands

| Command | Description |
|---------|-------------|
| `/name <name>` | Set display name |
| `/register <user> <pass>` | Create account |
| `/login <user> <pass>` | Log in |
| `/msg <user> <text>` | Private message |
| `/join <room>` | Join room |
| `/rooms` | List rooms |
| `/leave` | Return to lobby |
| `/who` | List users |
| `/help` | Show commands |

---

## Tech Stack

- **Backend**: Rust, Warp, Tokio
- **Frontend**: Vanilla JS, CSS
- **Protocol**: WebSocket
- **Storage**: JSON (users), in-memory (messages)

---

## License

MIT
