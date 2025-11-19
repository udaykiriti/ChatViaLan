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
