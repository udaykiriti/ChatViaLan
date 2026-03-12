# Commands Reference

All commands are sent as `Cmd` WebSocket messages with the `cmd` field containing the full command string, including the leading `/`.

---

## Identity

### /name \<username\>

Set or change the display name. If the name is already taken by another connected client, a numeric suffix is appended automatically (e.g., `alice-1`).

This command is also used for guest access — the client can chat without registering.

```
/name alice
```

### /register \<username\> \<password\>

Create a new permanent account. The username must not already exist in `users.json`. The password is hashed with bcrypt before storage.

```
/register alice hunter2
```

### /login \<username\> \<password\>

Authenticate with an existing account. On success, the client's `logged_in` flag is set to `true`, which is required for admin actions like `/kick`.

```
/login alice hunter2
```

---

## Room Navigation

### /join \<room\>

Move to a different room. If the room does not exist it is created dynamically. The client's current room history is cleared and the new room's history is delivered.

```
/join tech
```

### /leave

Return to `lobby` from any other room.

### /rooms

List all rooms and their current member counts. The server responds with a `RoomList` message.

### /room

Show the name of the current room as a system message.

---

## Messaging

### /msg \<username\> \<text\>

Send a direct message to another connected user. The message is stored in `PrivateHistories` under a key formed by sorting and lowercasing both usernames (e.g., `alice,bob`). DMs are not visible to other clients.

```
/msg bob Hey, are you free?
```

### /history

Re-deliver the current room's message history to the requesting client. Useful after a reconnect or UI refresh.

### /nudge

Broadcast a `Nudge` message to everyone in the current room. The frontend is expected to respond with a shake animation or sound.

---

## Information

### /who

List the users currently in the same room, along with their login status (registered or guest).

### /list

Broadcast a `List` message containing the usernames of everyone in the current room.

### /stats

Display server-level metrics: active connections, room count, total messages, and approximate memory usage. The information comes from the `Metrics` struct.

### /help

Print the list of available commands as a system message.

---

## Admin

### /kick \<username\>

Disconnect a user from the server. Only available to clients who are logged in (`logged_in == true`). The kicked client receives a system message before being disconnected.

```
/kick spammer
```

---

## Mentions

Typing `@username` anywhere in a message body automatically triggers a `Mention` message sent directly to that user, in addition to the normal room broadcast. No explicit command is needed.

```
Hey @bob, check this out.
```
