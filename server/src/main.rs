mod protocol;
mod room;

use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message;

use protocol::{ClientMessage, ServerMessage};
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

    // TODO: Nachrichten aus Channel an WebSocket senden
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Raum-Tracking für diesen Spieler
    let mut current_room: Option<String> = None;

    // Empfangs-Loop
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                println!("[Server] Spieler {} sagt: {}", player_id, text);

                let msg: ClientMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        let err = ServerMessage::Error {
                            message: format!("Ungültiges JSON: {}", e),
                        };
                        let _ = tx.send(err.to_json());
                        continue;
                    }
                };

                match msg {
                    ClientMessage::CreateRoom => {
                        // Falls schon in Raum, verlassen
                        if let Some(ref old_room) = current_room {
                            state.rooms.leave_room(old_room, player_id).await;
                        }

                        let room_id = state.rooms.create_room().await;
                        state.rooms.join_room(&room_id, player_id, tx.clone()).await;
                        current_room = Some(room_id.clone());

                        println!(
                            "[Server] Spieler {} hat Raum {} erstellt",
                            player_id, room_id
                        );

                        let response = ServerMessage::RoomCreated { room_id };
                        let _ = tx.send(response.to_json());
                    }

                    ClientMessage::JoinRoom { .. } => {
                        // TODO: implementieren
                        let err = ServerMessage::Error {
                            message: "JoinRoom noch nicht implementiert".to_string(),
                        };
                        let _ = tx.send(err.to_json());
                    }

                    ClientMessage::GameMove { .. } => {
                        // TODO: implementieren
                        let err = ServerMessage::Error {
                            message: "GameMove noch nicht implementiert".to_string(),
                        };
                        let _ = tx.send(err.to_json());
                    }

                    ClientMessage::LeaveRoom => {
                        // TODO: implementieren
                        let err = ServerMessage::Error {
                            message: "LeaveRoom noch nicht implementiert".to_string(),
                        };
                        let _ = tx.send(err.to_json());
                    }
                }
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

    // Aufräumen: Raum verlassen bei Disconnect
    if let Some(ref room_id) = current_room {
        state.rooms.leave_room(room_id, player_id).await;
        println!(
            "[Server] Spieler {} hat Raum {} verlassen (disconnect)",
            player_id, room_id
        );
    }

    send_task.abort();
    println!("[Server] Spieler {} disconnected", player_id);
}
