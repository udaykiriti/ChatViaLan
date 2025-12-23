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
- **Persistent message history (disk-based)**

### Advanced
- **Online Status & Idle Tracking** (Active/Idle indicators)
- **Admin Moderation** (`/kick` command)
- **Pinned Messages** with "click to jump" functionality
- **PWA Support** (Installable on mobile/desktop)
- Message reactions (👍 ❤️ 😂 😮 😢 🎉)
- Edit & delete messages
- @mentions with profile modals
- Typing indicators
- Sound notifications + unread count
- Dark/light theme (persistent preference)
- Rate limiting (5 msg / 10 sec)

### UI/UX
- Messenger-style bubble layout
- **Smooth staggered animations** for new messages
- **Interactive Profile Modals** in the user list
- Available rooms with auto-refresh
- Fully responsive (mobile + desktop)

---

## Project Structure

```
rust-chat/
├── Cargo.toml
├── users.json              # User accounts
├── history.json            # Persistent chat history
├── uploads/                # Uploaded files
├── static/
│   ├── index.html
│   ├── manifest.json       # PWA Manifest
│   ├── sw.js               # Service Worker
│   ├── favicon.svg
│   ├── css/ ...            # Stylesheets
│   └── js/ ...             # Frontend logic
└── src/ ...                # Backend Rust source
```

---

## Quick Start

```bash
# Run server
cargo run --release

# Open browser
# Local: http://localhost:8080
```

---

## Commands

| Command | Description |
|---------|-------------|
| `/name <name>` | Set display name |
| `/register <u> <p>` | Create account |
| `/login <u> <p>` | Log in |
| `/msg <user> <text>` | Private message |
| `/join <room>` | Join room |
| `/kick <user>` | Kick a user (Admin only) |
| `/rooms` | List rooms |
| `/leave` | Return to lobby |
| `/who` | List users with status |
| `/help` | Show commands |

---

## Tech Stack

- **Backend**: Rust (Warp, Tokio, DashMap)
- **Frontend**: Vanilla JS, CSS (Modular structure)
- **Persistence**: JSON-based (Users, History)
- **PWA**: Manifest, Service Worker

---

## License

MIT
