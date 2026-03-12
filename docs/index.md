# Documentation

---

## Contents

- [Overview](overview.md) — What the project is, how to build and run it, and runtime files.
- [Architecture](architecture.md) — Components, shared state, message flow, concurrency, and persistence model.
- [API Reference](api.md) — HTTP endpoints and the full WebSocket message protocol.
- [Commands Reference](commands.md) — All chat commands with usage and behavior.
- [Frontend](frontend.md) — JavaScript modules, CSS files, PWA, and service worker.

### Backend Modules

One file per Rust source module in `src/`:

- [main.rs](modules/main.md) — Entry point, routes, background tasks, and graceful shutdown.
- [types.rs](modules/types.md) — All shared types, type aliases, and message enums.
- [client.rs](modules/client.md) — WebSocket connection lifecycle and message dispatch.
- [commands.rs](modules/commands.md) — Slash command parser and handler implementations.
- [room.rs](modules/room.md) — Broadcasting, history management, reactions, edits, deletes.
- [auth.rs](modules/auth.md) — Registration, login, and bcrypt credential storage.
- [helpers.rs](modules/helpers.md) — Utility functions: name lookup, profanity filter, link preview fetch.
- [rate_limit.rs](modules/rate_limit.md) — Per-client message rate enforcement.
- [typing.rs](modules/typing.md) — Typing indicator state and broadcast.
- [metrics.rs](modules/metrics.md) — Server performance counters.
- [upload.rs](modules/upload.md) — Multipart file upload handler.
