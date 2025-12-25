# Rust Chat — Real-Time WebSocket Chat Server

A fully local, offline-friendly, multi-user chat system built with **Rust**, **Warp**, and **WebSockets**.  
Featuring multiple themes and a classic/modern hybrid UI.

---

## Features

### Core
- Real-time WebSocket messaging
- Multiple rooms with member counts
- User accounts (register/login) + guest mode
- Private messaging
- File upload & sharing
- **Persistent message history (non-blocking async I/O)**

### Advanced
- **Link Previews** (Open Graph metadata extraction)
- **Emoji Picker** (categorized emoji selection)
- **Online Status & Idle Tracking** (Active/Idle indicators)
- **Admin Moderation** (`/kick` command)
- **Secure Private Messaging** (Stored separately from rooms)
- **Profanity Filter** (Auto-censors banned words)
- **Pinned Messages** with "click to jump" functionality
- **PWA Support** (Installable on mobile/desktop)
- Message reactions (👍 ❤️ 😂 😮 😢 🎉)
- Edit & delete messages
- @mentions with profile modals
- Typing indicators
- Sound notifications + unread count
- Rate limiting (5 msg / 10 sec)
- **Markdown Support** (with '?' Cheat Sheet)

### Themes
- **Classic Theme** — Newspaper/XP/BBS hybrid with serif fonts, blue gradients, alternating rows
- **Midnight Theme** — Modern dark mode with cyan/purple accents and glow effects
- Dark/Light mode toggle (persistent preference)

### UI/UX
- Messenger-style bubble layout OR classic forum-style (theme dependent)
- **Smooth animations** with polished hover effects
- **Interactive Profile Modals** in the user list
- **Empty state messaging** for new rooms
- Available rooms with auto-refresh
- Fully responsive (mobile + desktop)

---

## Themes

Switch themes by changing the CSS link in `static/index.html`:

```html
<!-- Classic Theme (Newspaper/XP/BBS) -->
<link rel="stylesheet" href="css/classic.css">

<!-- Midnight Theme (Modern Dark) -->
<link rel="stylesheet" href="css/midnight.css">
```

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
│   ├── css/
│   │   ├── base.css        # CSS variables & layout
│   │   ├── sidebar.css     # Sidebar styles
│   │   ├── chat.css        # Chat area styles
│   │   ├── messages.css    # Message styles
│   │   ├── polish.css      # UI polish & animations
│   │   ├── classic.css     # Classic theme
│   │   ├── midnight.css    # Dark theme
│   │   └── mobile-fix.css  # Mobile responsiveness
│   └── js/
│       ├── main.js         # Entry point
│       ├── websocket.js    # WebSocket handler
│       ├── messages.js     # Message rendering
│       ├── events.js       # Event listeners
│       ├── features.js     # Theme/sound/typing
│       ├── reactions.js    # Reaction system
│       ├── utils.js        # Utilities & emoji picker
│       ├── config.js       # Configuration
│       ├── state.js        # App state
│       └── dom.js          # DOM references
└── src/
    ├── main.rs             # Server entry
    ├── types.rs            # Type definitions
    ├── room.rs             # Room & history management
    ├── commands.rs         # Command handling
    ├── helpers.rs          # Utility functions
    ├── auth.rs             # Authentication
    ├── rate_limit.rs       # Rate limiting
    ├── typing.rs           # Typing indicators
    └── client.rs           # Client management
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
| `/stats` | Show server metrics (Mem, Clients) |
| `/rooms` | List rooms |
| `/leave` | Return to lobby |
| `/who` | List users with status |
| `/pin <msg_id>` | Pin a message |
| `/help` | Show commands |

---

## Tech Stack

- **Backend**: Rust (Warp, Tokio, DashMap)
- **Link Previews**: reqwest + scraper (Open Graph)
- **Frontend**: Vanilla JS, CSS (Modular structure)
- **Persistence**: JSON-based (Users, History) with async I/O
- **PWA**: Manifest, Service Worker
- **Performance**: OnceLock caching for Regex/Selectors

---

## Recent Updates

- ✅ Link preview generation (Open Graph metadata)
- ✅ Emoji picker with categorized emojis
- ✅ Classic theme (Newspaper/XP/BBS hybrid)
- ✅ Midnight theme (Modern dark mode)
- ✅ Non-blocking file I/O (tokio::fs)
- ✅ Regex/Selector caching (OnceLock)
- ✅ Mobile sidebar fixes
- ✅ UI polish pass (hover effects, animations)

---

## License

MIT
