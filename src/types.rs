//! Core data types and type aliases for the chat server.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use serde::{Deserialize, Serialize};

/// Sender channel for WebSocket messages to a client.
pub type Tx = mpsc::UnboundedSender<warp::ws::Message>;

/// Connected clients map: client_id -> Client
pub type Clients = Arc<Mutex<HashMap<String, Client>>>;

/// Per-room message histories: room_name -> VecDeque<HistoryItem>
pub type Histories = Arc<Mutex<HashMap<String, VecDeque<HistoryItem>>>>;

/// Registered users: username -> password_hash
pub type Users = Arc<Mutex<HashMap<String, String>>>;

/// Represents a connected client.
#[derive(Clone)]
pub struct Client {
    pub name: String,
    pub tx: Tx,
    pub logged_in: bool,
    pub room: String,
}

/// A single message in the chat history.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryItem {
    pub from: String,
    pub text: String,
    pub ts: u64,
}

/// Messages sent from server to client.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Outgoing {
    System { text: String },
    Msg { from: String, text: String, ts: u64 },
    List { users: Vec<String> },
    History { items: Vec<HistoryItem> },
}

/// Messages received from client.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Incoming {
    Cmd { cmd: String },
    Msg { text: String },
}
