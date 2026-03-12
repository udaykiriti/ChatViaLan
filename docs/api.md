# API Reference

---

## HTTP Endpoints

### GET /health

Returns basic server status.

**Response** (JSON):

```json
{
  "status": "ok",
  "uptime_secs": 3721,
  "clients": 4
}
```

---

### GET /metrics

Returns detailed server statistics.

**Response** (JSON):

```json
{
  "uptime_secs": 3721,
  "total_messages": 842,
  "total_connections": 17,
  "active_clients": 4,
  "rooms": 5,
  "memory_mb": 12
}
```

`memory_mb` is read from `/proc/self/statm` (Linux only). On other platforms it returns `0`.

---

### GET /ws

WebSocket upgrade endpoint. The client connects here and the connection is held open for the duration of the session. All chat communication happens over this connection using the JSON protocol described below.

---

### POST /upload

Accepts multipart form data containing one or more files.

**Constraints:** Maximum request size is 5 GB (enforced by Warp's body size limit).

**Response on success** (HTTP 201, JSON):

```json
{
  "filename": "a1b2c3d4_document.pdf",
  "url": "/uploads/a1b2c3d4_document.pdf",
  "size": 204800
}
```

Files are saved to the `uploads/` directory. The filename is prefixed with the first 8 characters of a UUID to avoid collisions. Characters outside `[A-Za-z0-9._-]` in the original filename are replaced with `_`.

**Response on failure:**
- `400 Bad Request` — No file parts found in the multipart body.
- `500 Internal Server Error` — File write failed.

---

### GET /uploads/:filename

Serves uploaded files statically. Resolved from the `uploads/` directory relative to the working directory.

---

### GET /static/:path

Serves the frontend assets (HTML, CSS, JavaScript, icons, manifest).

---

## WebSocket Protocol

All messages are JSON objects. Messages from the server have a `type` field (the enum tag). Messages from the client also have a `type` field.

### Client to Server (Incoming)

#### Cmd

Send a slash command.

```json
{ "type": "Cmd", "cmd": "/join tech" }
```

#### Msg

Send a chat message to the current room.

```json
{ "type": "Msg", "text": "Hello, room." }
```

Text is profanity-censored server-side before broadcast. If the text contains a URL, the server may follow it up with a `LinkPreview` message.

#### Typing

Notify the room that the client is or is not typing.

```json
{ "type": "Typing", "is_typing": true }
```

#### React

Add or toggle a reaction on a message. Sending the same emoji twice removes it.

```json
{ "type": "React", "msg_id": "a1b2c3d4", "emoji": "thumbsup" }
```

#### Edit

Edit one of the client's own messages.

```json
{ "type": "Edit", "msg_id": "a1b2c3d4", "new_text": "Corrected text." }
```

#### Delete

Delete one of the client's own messages. Deletion is soft — the message is kept in history with `deleted: true` and its text is replaced server-side.

```json
{ "type": "Delete", "msg_id": "a1b2c3d4" }
```

#### MarkRead

Inform the room which message was last read by this client.

```json
{ "type": "MarkRead", "last_msg_id": "a1b2c3d4" }
```

---

### Server to Client (Outgoing)

#### System

A server-generated informational message (join/leave notices, command responses, errors).

```json
{ "type": "System", "text": "alice has joined the room." }
```

#### Msg

A chat message from another client.

```json
{
  "type": "Msg",
  "id": "a1b2c3d4",
  "from": "alice",
  "text": "Hello.",
  "ts": 1710000000,
  "reactions": { "thumbsup": ["bob"] },
  "edited": false
}
```

#### History

Bulk delivery of existing room history on join or `/history` command.

```json
{
  "type": "History",
  "items": [
    {
      "id": "a1b2c3d4",
      "from": "alice",
      "text": "Hello.",
      "ts": 1710000000,
      "reactions": {},
      "edited": false,
      "deleted": false
    }
  ]
}
```

Deleted messages are included in history with `deleted: true`; the frontend should render them as removed.

#### List

User list for the current room.

```json
{ "type": "List", "users": ["alice", "bob"] }
```

#### RoomList

List of all rooms with member counts.

```json
{
  "type": "RoomList",
  "rooms": [
    { "name": "lobby", "members": 3 },
    { "name": "tech", "members": 1 }
  ]
}
```

#### Typing

Current set of users actively typing in the room.

```json
{ "type": "Typing", "users": ["alice"] }
```

#### Reaction

A reaction was added to or removed from a message.

```json
{
  "type": "Reaction",
  "msg_id": "a1b2c3d4",
  "emoji": "thumbsup",
  "user": "bob",
  "added": true
}
```

#### Edit

A message was edited.

```json
{ "type": "Edit", "msg_id": "a1b2c3d4", "new_text": "Corrected text." }
```

#### Delete

A message was deleted.

```json
{ "type": "Delete", "msg_id": "a1b2c3d4" }
```

#### ReadReceipt

A user has read up to a certain message.

```json
{ "type": "ReadReceipt", "user": "bob", "last_msg_id": "a1b2c3d4" }
```

#### Mention

The client was mentioned in a message.

```json
{ "type": "Mention", "from": "alice", "text": "Hey @bob!", "mentioned": "bob" }
```

#### Status

A user's presence status changed. Broadcast globally to all connected clients.

```json
{ "type": "Status", "user": "alice", "status": "idle" }
```

Known status values: `online`, `idle`.

#### LinkPreview

Metadata extracted from a URL in a message.

```json
{
  "type": "LinkPreview",
  "url": "https://example.com",
  "title": "Example Domain",
  "description": "An illustrative domain.",
  "image": "https://example.com/og.png"
}
```

`description` and `image` may be `null` if not found.

#### Nudge

A client used `/nudge`. The frontend should produce a visual or audio effect.

```json
{ "type": "Nudge", "from": "alice" }
```
