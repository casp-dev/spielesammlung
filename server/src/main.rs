mod room;

use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message;

use room::RoomManager;

// Globaler Server-State
pub struct ServerState {
    rooms: RoomManager,
    next_player_id: RwLock<u64>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            rooms: RoomManager::new(),
            next_player_id: RwLock::new(1),
        }
    }

    pub async fn get_next_player_id(&self) -> u64 {
        let mut id = self.next_player_id.write().await;
        let current = *id;
        *id += 1;
        current
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(ServerState::new());
    let addr = "0.0.0.0:9000";
    let listener = TcpListener::bind(addr).await.expect("Kann nicht binden");

    println!("Game Server läuft auf {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        println!("[Server] Neue Verbindung: {}", addr);
        let state = state.clone();
        tokio::spawn(handle_connection(stream, addr, state));
    }
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, state: Arc<ServerState>) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("[Server] WebSocket-Fehler für {}: {}", addr, e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let player_id = state.get_next_player_id().await;

    println!("[Server] Spieler {} verbunden ({})", player_id, addr);

    // Channel für ausgehende Nachrichten an diesen Client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Task: Nachrichten aus Channel an WebSocket senden
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Empfangs-Loop (noch ohne Protokoll)
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                println!("[Server] Spieler {} sagt: {}", player_id, text);
                // TODO: Protokoll-Verarbeitung
                let _ = tx.send(format!("ECHO:{}", text));
            }
            Ok(Message::Close(_)) => {
                println!("[Server] Spieler {} hat Verbindung geschlossen", player_id);
                break;
            }
            Err(e) => {
                eprintln!("[Server] Fehler von Spieler {}: {}", player_id, e);
                break;
            }
            _ => {}
        }
    }

    send_task.abort();
    println!("[Server] Spieler {} disconnected", player_id);
}
