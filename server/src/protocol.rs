use serde::{Deserialize, Serialize};

// von Client an Server
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    CreateRoom,
    JoinRoom { room_id: String },
    GameMove { data: serde_json::Value },
    LeaveRoom,
}

// von Server an Client
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    RoomCreated {
        room_id: String,
    },
    RoomJoined {
        room_id: String,
        player_number: usize,
    },
    PlayerJoined {
        player_number: usize,
    },
    GameMove {
        data: serde_json::Value,
    },
    PlayerLeft {
        player_number: usize,
    },
    Error {
        message: String,
    },
}

impl ServerMessage {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
