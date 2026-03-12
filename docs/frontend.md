# Frontend

The frontend is a single-page application written in plain HTML, CSS, and JavaScript. It uses no frameworks or build tools and is served statically from the `static/` directory.

---

## Entry Point

`static/index.html` — The only HTML file. Loads the CSS and JS modules and contains the application shell markup.

---

## JavaScript Modules

Located in `static/js/`. Loaded as ES modules.

| File | Responsibility |
|------|---------------|
| `main.js` | Application initialization. Sets up the WebSocket connection and wires all modules together on page load. |
| `websocket.js` | WebSocket connection management. Sends and receives JSON messages. Reconnects on unexpected close. Dispatches received messages to the appropriate handler based on the `type` field. |
| `state.js` | Client-side state: current room, username, login status, message list, typing users, reaction state, and unread counts. |
| `messages.js` | Renders `Msg`, `History`, `Edit`, `Delete`, and `System` messages into the chat DOM. Handles the message list and scroll behavior. |
| `reactions.js` | Renders reaction buttons on messages. Handles click events to send `React` messages. Updates reaction counts on receipt of `Reaction` messages. |
| `events.js` | Attaches event listeners to the input field, send button, room list, and other interactive elements. Delegates to the appropriate modules. |
| `dom.js` | Low-level DOM utilities: element creation, class toggling, scroll helpers, and modal open/close. |
| `features.js` | Higher-level feature logic: mention parsing in the input, typing indicator debounce, nudge animation, link preview rendering, and PWA install prompt. |
| `utils.js` | Pure utility functions: timestamp formatting, username extraction, text sanitization for display, and URL detection in message text. |
| `config.js` | Constants: WebSocket URL, default room name, rate-limit warning text, and feature flags. |

---

## CSS Modules

Located in `static/css/`. All loaded from `index.html`.

| File | Description |
|------|-------------|
| `base.css` | CSS reset, root variables (colors, fonts, spacing), and global element defaults. |
| `sidebar.css` | Room list panel, user list, and sidebar toggle behavior. |
| `chat.css` | Main chat layout, header bar, input area, and attachment button. |
| `messages.css` | Message bubbles, system message styling, timestamps, and edit/delete indicators. |
| `msn.css` | Retro theme overrides that give the app its classic MSN Messenger visual style. |
| `polish.css` | Fine-grained adjustments: hover effects, transitions, focus rings, and scrollbar styling. |
| `responsive.css` | Media queries for mobile viewports. Collapses the sidebar and adjusts layout for small screens. |

---

## Other Static Files

| File | Description |
|------|-------------|
| `manifest.json` | PWA manifest. Defines app name, icons, theme color, and display mode for installability. |
| `sw.js` | Service worker. Enables the PWA to be installed on mobile and desktop. Handles caching for offline-capable use. |
| `favicon.svg` | Application icon in SVG format. |
| `libs/` | Third-party libraries bundled locally (no CDN dependency). |

---

## WebSocket Protocol

The frontend communicates with the server exclusively through JSON messages over the WebSocket connection established at `GET /ws`. All message types and fields are documented in [api.md](api.md).

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `/` | Focus the text input. |
| `Esc` | Close open modals or the sidebar. |
| `Up` / `Down` | Navigate command autocomplete suggestions. |
| `Tab` | Accept the highlighted command suggestion. |
| `Enter` | Send the current message. |

---

## PWA

The application registers `sw.js` as a service worker on load. This enables:
- Installation as a standalone app on Android, iOS, and desktop (via browser prompt).
- Basic asset caching for faster subsequent loads.

The manifest and service worker are minimal and do not implement background sync or push notifications.
