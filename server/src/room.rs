use rand::Rng;
use std::collections::HashMap;
use tokio::sync::{RwLock, mpsc};

pub type PlayerId = u64;
pub type PlayerSender = mpsc::UnboundedSender<String>;

pub struct Room {
    pub players: HashMap<PlayerId, PlayerSender>,
}

impl Room {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, player_id: PlayerId, sender: PlayerSender) {
        self.players.insert(player_id, sender);
    }

    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.players.remove(&player_id);
    }

    pub fn broadcast(&self, message: &str, exclude: Option<PlayerId>) {
        for (id, sender) in &self.players {
            if Some(*id) != exclude {
                let _ = sender.send(message.to_string());
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }
}

pub struct RoomManager {
    rooms: RwLock<HashMap<String, Room>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_room(&self) -> String {
        let room_id = generate_room_id();
        let room = Room::new();

        let mut rooms = self.rooms.write().await;
        rooms.insert(room_id.clone(), room);

        room_id
    }

    pub async fn room_exists(&self, room_id: &str) -> bool {
        let rooms = self.rooms.read().await;
        rooms.contains_key(room_id)
    }
    pub async fn join_room(&self, room_id: &str, player_id: PlayerId, sender: PlayerSender) {
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.add_player(player_id, sender);
        }
    }

    pub async fn leave_room(&self, room_id: &str, player_id: PlayerId) {
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.remove_player(player_id);

            if room.is_empty() {
                rooms.remove(room_id);
                println!("[RoomManager] Raum {} wurde gelöscht (leer)", room_id);
            }
        }
    }

    pub async fn broadcast(&self, room_id: &str, message: &str, exclude: Option<PlayerId>) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_id) {
            room.broadcast(message, exclude);
        }
    }

    pub async fn player_count(&self, room_id: &str) -> usize {
        let rooms = self.rooms.read().await;
        rooms.get(room_id).map(|r| r.players.len()).unwrap_or(0)
    }
}

fn generate_room_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
