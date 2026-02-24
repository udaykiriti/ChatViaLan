# Rust Chat — Real-Time WebSocket Chat Server

![Rust](https://img.shields.io/badge/rust-v1.75%2B-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Maintained](https://img.shields.io/badge/maintained-yes-green.svg)

A fully local, offline-friendly, multi-user chat system built with **Rust**, **Warp**, and **WebSockets**.  
Featuring multiple themes, a classic/modern hybrid UI, and robust admin tools.


## Features

### Core
- **Real-time Messaging**: Instant delivery via WebSockets.
- **Rooms**: Multiple channels with active member counts.
- **Auth**: User accounts (Register/Login) and Guest mode.
- **Privacy**: Secure private messaging (stored separately) and file sharing.
- **Persistence**: Chat history saved asynchronously (JSON).

### Advanced
- **Link Previews**: Automatic Open Graph metadata extraction (Title, Description, Image).
- **Interactive UI**:
    - **Emoji Picker**: Categorized emojis with skin tone support.
    - **Reactions**: React to messages (👍 ❤️ 😂 😮 😢 🎉).
    - **Mentions**: `@user` support with profile modals.
    - **Typing Indicators**: Real-time "User is typing..." status.
- **Engagement**:
    - **Nudge**: `/nudge` command to shake the room and play a sound.
    - **Pinned Messages**: Pin important messages for easy access.
    - **Online Status**: Auto-away/idle detection.
- **Ease of Use**:
    - **QR Code LAN Connect**: Scan terminal QR code to join from mobile.
    - **PWA**: Installable on mobile and desktop devices.
    - **Shortcuts**: extensive keyboard control.
- **Safety**:
    - **Profanity Filter**: Auto-censoring of banned words.
    - **Moderation**: `/kick` command for admins.

---

## UI

The app uses modular CSS (`base`, `sidebar`, `chat`, `messages`, `msn`, `responsive`) with a classic/retro visual style out of the box.

---

## Quick Start

### 1. Run the Server
```bash
cargo run --release
```

### 2. Connect
- **Localhost**: Visit `http://localhost:8080`
- **LAN (Mobile)**: Scan the QR code printed in your terminal.

---

## Commands

| Command | Usage | Description |
|---------|-------|-------------|
| **Identity** | `/name <name>` | Set your display name. |
| **Auth** | `/register <u> <p>` | Create a new account. |
| | `/login <u> <p>` | Log in to your account. |
| **Chat** | `/msg <user> <text>` | Send a private message. |
| | `/join <room>` | Switch rooms (e.g., `/join tech`). |
| | `/leave` | Return to `lobby`. |
| | `/rooms` | List all available rooms and member counts. |
| | `/room` | Show your current room. |
| | `/list` | Broadcast user list for current room. |
| | `/nudge` | **Shake screen** & play sound. |
| | `/history` | Reload current room history. |
| **Info** | `/who` | List users in current room. |
| | `/stats` | Show server memory & connections. |
| | `/help` | Show available commands. |
| **Admin** | `/kick <user>` | Kick a user (Admin only). |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `/` | Focus text input. |
| `Esc` | Close modals or sidebar. |
| `↑` / `↓` | Navigate command suggestions. |
| `Tab` | Autocomplete command from suggestion. |
| `Enter` | Send message. |

---

## API Endpoints

The server exposes endpoints for monitoring and health checks:

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Basic status (uptime, client count). |
| `GET` | `/metrics` | Detailed stats (memory usage, total messages, rooms). |

---

## Technical Details

- **Backend**: Rust (Warp, Tokio)
    - *State Management*: `DashMap` for high-concurrency capability.
    - *Discovery*: `mdns-sd` for zero-config LAN discovery.
- **Frontend**: Vanilla JavaScript (ES6+), Modular CSS.
    - *No Frameworks*: Lightweight and fast.
- **Storage**: JSON-based flat files with non-blocking async I/O (`tokio::fs`).

## Notes

- Runtime files (`history.json`, `users.json`, `uploads/`) are generated locally while the server runs.
- PWA support is enabled via `static/sw.js` registration from the frontend.
- `reqwest` is configured with Rustls TLS, so OpenSSL system development packages are not required for normal builds.

## Recent Updates

- **Stability**: Improved startup resilience for mDNS/QR paths.
- **WebSocket Lifecycle**: Fixed early-auth disconnect handling.
- **PM History**: Normalized private-history keys case-insensitively.
- **Build Portability**: Switched HTTP TLS backend to Rustls.

---

## License

MIT
[Read the License](LICENSE)
