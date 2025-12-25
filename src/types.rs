//! Core data types and type aliases for the chat server.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};
use serde::{Deserialize, Serialize};
use dashmap::DashMap;

/// Sender channel for WebSocket messages to a client.
pub type Tx = mpsc::UnboundedSender<warp::ws::Message>;

/// Connected clients map: client_id -> Client
pub type Clients = Arc<DashMap<String, Client>>;

/// Per-room message histories: room_name -> VecDeque<HistoryItem>
pub type Histories = Arc<RwLock<HashMap<String, VecDeque<HistoryItem>>>>;

/// Private message histories: key (sorted usernames) -> VecDeque<HistoryItem>
pub type PrivateHistories = Arc<RwLock<HashMap<String, VecDeque<HistoryItem>>>>;

/// Registered users: username -> password_hash
pub type Users = Arc<DashMap<String, String>>;

/// Represents a connected client.
#[derive(Clone)]
pub struct Client {
    pub name: String,
    pub tx: Tx,
    pub logged_in: bool,
    pub room: String,
    pub last_message_times: Vec<Instant>, // For rate limiting
    pub is_typing: bool,
    pub last_read_msg_id: Option<String>, // For read receipts
    pub last_active: Instant,             // For online status
}

/// A single message in the chat history.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryItem {
    pub id: String,           // Unique message ID
    pub from: String,
    pub text: String,
    pub ts: u64,
    #[serde(default)]
    pub reactions: HashMap<String, Vec<String>>,  // emoji -> [usernames]
    #[serde(default)]
    pub edited: bool,
    #[serde(default)]
    pub deleted: bool,
}

/// Messages sent from server to client.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Outgoing {
    System { text: String },
    Msg { id: String, from: String, text: String, ts: u64, reactions: HashMap<String, Vec<String>>, edited: bool },
    List { users: Vec<String> },
    History { items: Vec<HistoryItem> },
    Typing { users: Vec<String> },
    Reaction { msg_id: String, emoji: String, user: String, added: bool },
    Edit { msg_id: String, new_text: String },
    Delete { msg_id: String },
    ReadReceipt { user: String, last_msg_id: String },
    Mention { from: String, text: String, mentioned: String },
    RoomList { rooms: Vec<RoomInfo> },
    Status { user: String, status: String },
    LinkPreview { msg_id: String, title: String, description: String, image: String, url: String },
}

/// Room info for available rooms list.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomInfo {
    pub name: String,
    pub members: usize,
}

/// Messages received from client.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Incoming {
    Cmd { cmd: String },
    Msg { text: String },
    Typing { is_typing: bool },
    React { msg_id: String, emoji: String },
    Edit { msg_id: String, new_text: String },
    Delete { msg_id: String },
    MarkRead { last_msg_id: String },
}
