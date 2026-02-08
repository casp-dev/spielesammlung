use std::collections::HashMap;
use tokio::sync::mpsc;

pub type PlayerId = u64;
pub type PlayerSender = mpsc::UnboundedSender<String>;

/// Ein Spielraum
pub struct Room {
    pub id: String,
    pub players: HashMap<PlayerId, PlayerSender>,
}

impl Room {
    pub fn new(id: String) -> Self {
        Self {
            id,
            players: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, player_id: PlayerId, sender: PlayerSender) {
        self.players.insert(player_id, sender);
    }

    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.players.remove(&player_id);
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }
}
