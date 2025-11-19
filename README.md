# ⚡ Rust Chat — Real-Time WebSocket Chat Server

A fully local, offline-friendly, multi-user chat system built using **Rust**, **Warp**, and **WebSockets**.  
Designed for LAN/localhost use, works completely without internet, and includes file sharing, rooms, login system, and a clean responsive UI.

---

##  Features

-  Real-time WebSocket chat  
-  Multiple rooms (`/join`, `/rooms`, `/leave`)  
-  User accounts (`/register`, `/login`) stored in `users.json`  
-  Guest mode support  
-  Private messaging (`/msg username text`)  
-  File upload + sharing (stores files in `uploads/`)  
-  Persistent message history per room  
-  Responsive UI (single-page HTML/JS)  
-  Works 100% offline (LAN or local)  
-  Simple project structure, easy to modify  

---

##  Project Flow

rust-chat/
│
├── Cargo.toml
├── users.json # usernames + salted SHA256 hashed passwords
├── uploads/ # stored uploaded files
├── static/
│ └── index.html # frontend UI
└── src/
└── main.rs # Rust server code


Skip to content
Chat history

You said:
⚡ Rust Chat — Real-Time WebSocket Chat Server

A fully local, offline-capable, multi-user chat system written in Rust + Warp with the following features:

✅ Features

Real-time chat using WebSockets

Rooms support (/join <room>, /rooms, /leave)

User accounts with register/login (stored in users.json)

Guest mode with display names

Persistent chat history per room

Private messaging (/msg username text)

File upload + sharing (uploads stored in uploads/ folder)

Works 100% offline (LAN or localhost)

Clean responsive UI (static/index.html)

📁 Project Structure
rust-chat/
│
├── src/
│   └── main.rs        # WebSocket server + HTTP routes
│
├── static/
│   └── index.html     # Frontend UI (HTML/CSS/JS)
│
├── uploads/           # Uploaded files stored here (auto-created)
├── users.json         # Registered username/password hashes
└── Cargo.toml

🚀 Run the Project
1. Install Rust

If you don't already have it:

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

2. Run the server
cargo run


You should see:

Server running at http://0.0.0.0:8080/

3. Open the chat UI

On same machine:
👉 http://localhost:8080

On another device in same WiFi/LAN:
👉 http://YOUR_LOCAL_IP:8080
(Example: http://172.20.10.2:8080)

No internet required — works completely offline.

💬 Chat Commands
Command	Description
/name <newname>	Change your display name
/register <user> <pass>	Create a new account
/login <user> <pass>	Authenticate as a registered user
/msg <user> <text>	Private message
/history	Show message history for current room
/join <roomname>	Create or join a room
/rooms	List all active rooms
/leave	Return to the lobby room
🗂 File Upload & Sharing

From the UI:

Click Upload & Share

File is sent to POST /upload

Server stores it in uploads/<unique-id>_filename.ext

Chat automatically posts the file link

Everyone in the room can click and download it

File links look like:

http://localhost:8080/uploads/12345_file.png

🌐 Offline / LAN Usage

This project does not require internet.

Works offline because:

UI is served locally (static/index.html)

WebSocket connection is local (ws://localhost/ws)

All files stored locally

No CDN or cloud calls

To use in LAN:

Run:

ip a


Find your IPv4 address (example: 192.168.1.15)

Connect from phone or another laptop:

http://192.168.1.15:8080


Make sure firewall allows port 8080:

sudo firewall-cmd --add-port=8080/tcp --permanent
sudo firewall-cmd --reload

🔧 Server-Side File Storage

Uploaded files go here:

uploads/
   ├── 8f23a2b4_picture.png
   ├── 9ac12abc_notes.pdf
   └── ...


Users’ credentials are stored securely hashed:

users.json

📌 Notes & Tips

Passwords are hashed using SHA-256 + username salt
(Good for local use, not production-secure)

History capped to 200 messages per room

Rooms auto-create on /join roomname

Server auto-reconnects clients when offline temporarily

📄 License

You may use, modify, or redistribute this project freely.  
give some good code for readme.d
ChatGPT said:
README.md — Rust Chat (Real-Time WebSocket Chat Server)
# ⚡ Rust Chat — Real-Time WebSocket Chat Server

A fully local, offline-capable, multi-user chat system written in **Rust** + **warp** that works over LAN or localhost.  
Designed to be simple, secure-enough for local use, and easy to run — perfect for demos, offline group chats, or learning WebSockets in Rust.

---

## ✅ Features

- Real-time chat using WebSockets
- Multiple rooms support (`/join <room>`, `/rooms`, `/leave`)
- User accounts with `register` / `login` (stored in `users.json`)
- Guest mode with display names
- Persistent chat history per room (capped)
- Private messaging: `/msg <username> <text>`
- File upload & sharing (files stored in `uploads/`)
- Works completely offline (LAN or localhost)
- Responsive single-file UI at `static/index.html`
- Minimal dependencies; easy to read and modify

---

## Project Structure

rust-chat/
│
├── Cargo.toml
├── users.json # persisted user db (username -> passwd hash + salt)
├── uploads/ # uploaded files (created automatically)
├── static/
│ └── index.html # frontend UI (HTML/CSS/JS)
└── src/
└── main.rs # WebSocket server + HTTP routes


---

## Quickstart (Linux / macOS / Windows WSL)

1. Install Rust (if not already):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
Clone & run:

git clone <this-repo-url>
cd rust-chat
cargo run --release
You should see output like:

Server running at http://0.0.0.0:8080/
Open the UI:

On the same machine: http://localhost:8080

On another device in the same LAN: http://<YOUR_LOCAL_IP>:8080 (e.g. http://192.168.1.15:8080)

Chat Commands
Use the chat input to type commands (commands start with /):

/name <newname> — change your display name

/register <username> <password> — create an account

/login <username> <password> — authenticate as a registered user

/join <roomname> — create or join a room

/rooms — list active rooms

/leave — return to the lobby

/msg <username> <text> — send a private message

/history — show message history for current room

File Upload & Sharing
Click Upload & Share (UI) to POST a file to /upload.

Server stores files as uploads/<unique-id>_<original-filename>.

Chat auto-publishes a clickable file link, e.g.:

http://localhost:8080/uploads/8f23a2b4_picture.png
Example: CURL file upload
curl -F "file=@./example.png" http://localhost:8080/upload
The server returns a JSON response with url that you can paste in chat.

Storage & Security Notes
users.json stores users with SHA-256(password + username-salt).

Good for local demos but not production-grade. Replace with a proper password hashing algorithm (e.g., Argon2) for real deployments.

Uploaded files are saved as-is in uploads/. Sanitize/validate uploads before exposing to public networks if you intend to use this outside a trusted LAN.

Message history is capped (default: 200 messages per room) to avoid unbounded memory use.

Configuration
By default the server listens on port 8080 and binds to 0.0.0.0:

This allows LAN access. To restrict to localhost, change bind address in src/main.rs.

You can change history caps, upload limits, or password hashing strategy in main.rs.

Common Troubleshooting
Cannot access from other devices: ensure the server machine firewall allows port 8080 (example for firewall-cmd):

sudo firewall-cmd --add-port=8080/tcp --permanent
sudo firewall-cmd --reload
Uploads folder missing: server should auto-create uploads/ on start. If not, create it manually:

mkdir -p uploads
chmod 755 uploads
Port already in use: change the port in src/main.rs and rebuild.

Extending the Project
Ideas for improvements:

Replace SHA-256 with Argon2 or bcrypt for secure password storage.

Add TLS (HTTPS/WSS) to support encrypted LAN communication.

Add message persistence to a lightweight DB (e.g., SQLite) for larger history and queries.

Add typing indicators, read receipts, or message edit/delete.

Add admin/moderation commands and user reporting.

Development Tips
The UI is a single self-contained file static/index.html — quick to edit and test.

The Rust server uses warp for HTTP + WebSocket routes — familiar patterns if you’ve used warp before.

Keep changes small and test with two clients (open two browser tabs or two devices on LAN).

License
MIT — feel free to use, modify, and redistribute.

Example users.json (auto-managed)
{
  "alice": {
    "salt": "alice",
    "hash": "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3..."
  }
}
Contact / Help
If you want, I can:

generate a polished README.md file for you (ready to paste into the repo),

produce index.html and src/main.rs example code (single-file, minimal),

or walk through adding TLS or Argon2 password hashing.

Just tell me which of the above you'd like next! 🚀

You said:
give readme.md file format so i can paste there
ChatGPT said:
Here is a fully ready-to-paste README.md file — clean formatting, perfect for GitHub.
Just copy-paste it directly into your repo.

README.md
# ⚡ Rust Chat — Real-Time WebSocket Chat Server

A fully local, offline-friendly, multi-user chat system built using **Rust**, **Warp**, and **WebSockets**.  
Designed for LAN/localhost use, works completely without internet, and includes file sharing, rooms, login system, and a clean responsive UI.

---

## 🚀 Features

- 🔌 Real-time WebSocket chat  
- 🏠 Multiple rooms (`/join`, `/rooms`, `/leave`)  
- 👤 User accounts (`/register`, `/login`) stored in `users.json`  
- 👥 Guest mode support  
- 💬 Private messaging (`/msg username text`)  
- 🗂 File upload + sharing (stores files in `uploads/`)  
- 🕒 Persistent message history per room  
- 📱 Responsive UI (single-page HTML/JS)  
- 🌐 Works 100% offline (LAN or local)  
- 🧰 Simple project structure, easy to modify  

---

## 📁 Project Structure

rust-chat/
│
├── Cargo.toml
├── users.json # usernames + salted SHA256 hashed passwords
├── uploads/ # stored uploaded files
├── static/
│ └── index.html # frontend UI
└── src/
└── main.rs # Rust server code


---

## 🛠 Installation & Run

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
2. Run the server
cargo run --release
You should see:

Server running at http://0.0.0.0:8080/
3. Open the Chat UI
On same machine → http://localhost:8080

On another device in same WiFi/LAN → http://YOUR_LOCAL_IP:8080

To find your IP:

ip a
Example: 192.168.1.15

💬 Chat Commands
Command	Description
/name <newname>	Change your display name
/register <user> <pass>	Create an account
/login <user> <pass>	Log in as a user
/msg <user> <text>	Private message
/join <room>	Join or create a room
/rooms	List rooms
/leave	Back to lobby
/history	Show previous messages
🗂 File Upload & Sharing
Click Upload & Share in the UI

File is sent to POST /upload

Server saves file as:

uploads/<unique_id>_filename.ext
Chat posts a clickable link:

http://localhost:8080/uploads/1234_file.png
Works for images, PDFs, videos, zips, anything.

🌐 Offline / LAN Usage
This app does not require the internet.

Works offline because:

UI is hosted locally (static/index.html)

WebSocket runs at ws://localhost/ws

No external fonts or CDNs

All passwords/files stored locally

Allow port 8080 through firewall (Linux)
sudo firewall-cmd --add-port=8080/tcp --permanent
sudo firewall-cmd --reload
🔐 Security Notes
Passwords use SHA-256 with username salt

Safe for local/offline use

Not production secure

If you want, you can upgrade to Argon2 easily.

🧩 Configuration
You can modify:

max message history

upload size limit

room behavior

password hashing

server port

All available in main.rs.

🧱 Example users.json
{
  "alice": {
    "salt": "alice",
    "hash": "c6f5b1633e93c2a4a093e..."
  }
}
🚀 Ideas for Improvement
Add Argon2 hashing

Add message editing/deleting

Add typing indicators

Add push notifications

Add admin/ban system

Add SQLite for long-term message storage

📄 License
This project is free to use, modify, and redistribute.

❤️ Need Anything Else?
I can also generate:

index.html

main.rs

Modern UI with Tailwind

Dockerfile

Systemd service file

LAN-only optimized version

Authentication upgrade (Argon2)

Just tell me! 🚀


---

If you want, I can also **generate a smaller README**, a more professional one, or one with badges.
You said:
generate .md file
ChatGPT said:
Here is the ready-to-download, ready-to-paste README.md content in pure Markdown format — no explanations, no extra text.

Just copy → create a file named README.md → paste.

# ⚡ Rust Chat — Real-Time WebSocket Chat Server

A fully local, offline-friendly, multi-user chat system built using **Rust**, **Warp**, and **WebSockets**.  
Designed for LAN or localhost use with rooms, login, private messages, uploads, and a clean responsive UI.

---

## 🚀 Features

- 🔌 Real-time WebSocket chat  
- 🏠 Multiple rooms (`/join`, `/rooms`, `/leave`)  
- 👤 User accounts (`/register`, `/login`) with salted SHA-256  
- 👥 Guest mode  
- 💬 Private messaging (`/msg username text`)  
- 🗂 File upload + sharing  
- 🕒 Persistent chat history  
- 📱 Responsive UI  
- 🌐 Works 100% offline  
- 🧰 Simple, clean codebase  

---

## 📁 Project Structure

rust-chat/
│
├── Cargo.toml
├── users.json # usernames + salted SHA256 hashes
├── uploads/ # stored uploaded files
├── static/
│ └── index.html # frontend UI
└── src/
└── main.rs # Rust server code


---

## 🛠 Installation & Run

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
Run the Project
cargo run --release
You should see:

Server running at http://0.0.0.0:8080/
Open the Chat UI
Local machine: http://localhost:8080

Same WiFi/LAN: http://YOUR_LOCAL_IP:8080

Find IP:

ip a
💬 Chat Commands
Command	Description
/name <newname>	Change display name
/register <user> <pass>	Create account
/login <user> <pass>	Log in
/msg <user> <text>	Private message
/join <room>	Join/create a room
/rooms	List rooms
/leave	Return to lobby
/history	Show recent messages
🗂 File Upload & Sharing
Click Upload & Share

File is stored in:

uploads/<unique_id>_filename.ext
Chat posts a clickable link, e.g.:

http://localhost:8080/uploads/abcd_file.png
Works with images, PDFs, zips, anything.

🌐 Offline / LAN Usage
This app works 100% offline:

UI hosted locally

WebSocket is local (ws://localhost/ws)

No CDNs or cloud dependencies

All storage is local (users.json, uploads/)

Allow port 8080 if needed:

sudo firewall-cmd --add-port=8080/tcp --permanent
sudo firewall-cmd --reload
🔐 Security Notes
Passwords are salted + SHA-256

Good for local use only

Not production-grade security

Consider Argon2 if upgrading

🔧 Configuration
You can easily customize:

Max message history

Upload size limit

Room settings

Password hashing

Server port

All inside main.rs.

🧱 Example users.json
{
  "alice": {
    "salt": "alice",
    "hash": "c6f5b1633e93c2a4a093e..."
  }
}
🚀 Future Improvements (Optional)
Argon2 password hashing

Message editing/deleting

Admin controls

SQLite message storage

Typing Indicators

Themes/Dark mode

📄 License
MIT — Free to use, modify, and redistribute.

💬 Need More?
If you want, I can also generate:

index.html

main.rs

Dockerfile

Systemd service

Tailwind modern UI

Secure login with Argon2

Just ask 🚀


---

If you want this saved as a **file download**, tell me — I can generate a downloadable `README.md` file.


No file chosenNo file chosen
ChatGPT can make mistakes. Check important info. See Cookie Preferences.
