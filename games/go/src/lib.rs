mod game;

use game::{Game, Stone};
use game_core::Game as CoreGame;
use game_core::MultiplayerGame;

use serde_json::Value;
use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

#[derive(PartialEq)]
enum GoGameState {
    Menu,
    WaitingForOpponent,
    Playing,
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 750.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Rust Go",
        options,
        Box::new(|_cc| Ok(Box::new(GoGame::default()))),
    )
}

pub struct GoGame {
    game: Game,
    status_message: String,
    game_state: GoGameState,
    client: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    room_key: String,
    multiplayer: bool,
    my_color: Option<Stone>,
}

impl Default for GoGame {
    fn default() -> Self {
        Self {
            game: Game::new(19),
            status_message: "Spiel gestartet. Schwarz ist am Zug.".to_owned(),
            game_state: GoGameState::Menu,
            client: None,
            room_key: String::new(),
            multiplayer: false,
            my_color: None,
        }
    }
}

impl GoGame {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CoreGame for GoGame {
    fn name(&self) -> &str {
        "Go"
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        match self.game_state {
            GoGameState::Menu => {
                ui.heading("Rust Go");
                self.multipalyer_ui(ui, false, false);
            }
            GoGameState::WaitingForOpponent => {
                ui.heading("Rust Go - Multiplayer");
                ui.label(format!("Room ID: {}", self.room_key));
                ui.label("Warte auf Gegner...");

                // Non-blocking: auf PlayerJoined msg warten
                if self.client.is_some() {
                    ui.ctx().request_repaint();

                    let received = match self.client.as_mut().unwrap().read() {
                        Ok(tungstenite::Message::Text(txt)) => Some(txt),
                        Err(tungstenite::Error::Io(ref e))
                            if e.kind() == std::io::ErrorKind::WouldBlock =>
                        {
                            None
                        }
                        _ => None,
                    };

                    if let Some(txt) = received {
                        if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                            if v.get("type").and_then(|t| t.as_str()) == Some("PlayerJoined") {
                                self.game_state = GoGameState::Playing;
                                self.status_message =
                                    "Gegner beigetreten! Schwarz ist am Zug.".to_owned();
                            }
                        }
                    }
                }

                if ui.button("Spiel starten").clicked() {
                    self.game_state = GoGameState::Playing;
                }
            }
            GoGameState::Playing => {
                ui.heading("Rust Go");

                ui.horizontal(|ui| {
                    ui.label(format!("Zug: {:?}", self.game.current_turn));
                    ui.label(format!("Schwarz gefangen: {}", self.game.captured_black));
                    ui.label(format!("Weiß gefangen: {}", self.game.captured_white));
                });

                if ui.button("Passen").clicked() {
                    self.game.pass();
                    if self.game.game_over {
                        let (b_score, w_score) = self.game.calculate_score();
                        self.status_message = format!(
                            "Spiel vorbei! Punkte: Schwarz {:.1}, Weiß {:.1}",
                            b_score, w_score
                        );
                    } else {
                        self.status_message =
                            format!("Gepasst. {:?} ist am Zug.", self.game.current_turn);
                    }
                }

                if ui.button("Spiel neustarten").clicked() {
                    self.game = Game::new(19);
                    self.status_message = "Spiel neugestartet. Schwarz ist am Zug.".to_owned();
                }

                ui.label(&self.status_message);

                if self.game.game_over {
                    let (b_score, w_score) = self.game.calculate_score();
                    ui.label(format!(
                        "Endstand:\nSchwarz: {:.1}\nWeiß: {:.1}",
                        b_score, w_score
                    ));
                    if b_score > w_score {
                        ui.label("Schwarz gewinnt!");
                    } else {
                        ui.label("Weiß gewinnt!");
                    }
                }

                let available_size = ui.available_size();
                let board_size = available_size.x.min(available_size.y) - 20.0;
                let (response, painter) = ui.allocate_painter(
                    egui::Vec2::new(board_size, board_size),
                    egui::Sense::click(),
                );

                let rect = response.rect;
                let grid_size = self.game.board.size;
                let cell_size = rect.width() / (grid_size as f32 + 1.0); // margin

                // Raster
                let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
                for i in 0..grid_size {
                    let pos = i as f32 * cell_size + cell_size;

                    // verticale Linien
                    painter.line_segment(
                        [
                            rect.min + egui::vec2(pos, cell_size),
                            rect.min + egui::vec2(pos, rect.height() - cell_size),
                        ],
                        stroke,
                    );

                    // Horizontale Linien
                    painter.line_segment(
                        [
                            rect.min + egui::vec2(cell_size, pos),
                            rect.min + egui::vec2(rect.width() - cell_size, pos),
                        ],
                        stroke,
                    );
                }

                // Draw stones
                for y in 0..grid_size {
                    for x in 0..grid_size {
                        if let Some(stone) = self.game.board.get(x, y) {
                            let center = rect.min
                                + egui::vec2(
                                    x as f32 * cell_size + cell_size,
                                    y as f32 * cell_size + cell_size,
                                );
                            let color = match stone {
                                Stone::Black => egui::Color32::BLACK,
                                Stone::White => egui::Color32::WHITE,
                            };
                            let stroke_color = match stone {
                                Stone::Black => egui::Color32::WHITE,
                                Stone::White => egui::Color32::BLACK,
                            };

                            painter.circle_filled(center, cell_size * 0.45, color);
                            painter.circle_stroke(
                                center,
                                cell_size * 0.45,
                                egui::Stroke::new(1.0, stroke_color),
                            );
                        }
                    }
                }

                // Klicks
                let my_turn = if self.multiplayer {
                    self.my_color == Some(self.game.current_turn)
                } else {
                    true
                };

                if response.clicked() && !self.game.game_over && my_turn {
                    if let Some(pos) = response.interact_pointer_pos() {
                        // pos zu Brett-Koordinaten umrechnen
                        let relative_pos = pos - rect.min;
                        let x_f = (relative_pos.x / cell_size) - 1.0;
                        let y_f = (relative_pos.y / cell_size) - 1.0;

                        let x = x_f.round() as i32;
                        let y = y_f.round() as i32;

                        if x >= 0 && x < grid_size as i32 && y >= 0 && y < grid_size as i32 {
                            match self.game.place_stone(x as usize, y as usize) {
                                Ok(_) => {
                                    self.status_message = format!(
                                        "Zug akzeptiert. {:?} ist am Zug.",
                                        self.game.current_turn
                                    );
                                    // Sendet move to server
                                    if self.multiplayer {
                                        let move_msg = format!(
                                            r#"{{ "type": "GameMove", "data": {{ "x": {}, "y": {} }} }}"#,
                                            x, y
                                        );
                                        let _ = self.send(&move_msg);
                                    }
                                }
                                Err(e) => {
                                    self.status_message = format!("Ungültiger Zug: {}", e);
                                }
                            }
                        }
                    }
                }

                // Non-blocking: auf mesg warten
                if self.multiplayer && self.client.is_some() {
                    ui.ctx().request_repaint();

                    let received = match self.client.as_mut().unwrap().read() {
                        Ok(tungstenite::Message::Text(txt)) => Some(txt),
                        Err(tungstenite::Error::Io(ref e))
                            if e.kind() == std::io::ErrorKind::WouldBlock =>
                        {
                            None
                        }
                        Err(e) => {
                            eprintln!("WebSocket error: {}", e);
                            None
                        }
                        _ => None,
                    };

                    if let Some(txt) = received {
                        self.on_text(txt);
                    }
                }
            } // Playing
        } // match
    }
}

impl eframe::App for GoGame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }
}

impl MultiplayerGame for GoGame {
    fn on_text(&mut self, msg: String) {
        // Parse GameMove von server
        if let Ok(v) = serde_json::from_str::<Value>(&msg) {
            if v.get("type").and_then(|t| t.as_str()) == Some("GameMove") {
                if let Some(data) = v.get("data") {
                    if let (Some(x), Some(y)) = (
                        data.get("x").and_then(|v| v.as_u64()),
                        data.get("y").and_then(|v| v.as_u64()),
                    ) {
                        let _ = self.game.place_stone(x as usize, y as usize);
                        self.status_message = format!("{:?} ist am Zug.", self.game.current_turn);
                    }
                }
            }
        }
    }

    fn set_client(&mut self, client: WebSocket<MaybeTlsStream<TcpStream>>) {
        self.client = Some(client);
    }

    fn get_client(&mut self) -> &mut WebSocket<MaybeTlsStream<TcpStream>> {
        self.client.as_mut().unwrap()
    }

    fn get_room_key_text(&mut self) -> &mut String {
        &mut self.room_key
    }

    fn set_room_key_text(&mut self, text: String) {
        self.room_key = text;
    }

    fn local_button_clicked(&mut self, player_counter: Option<u16>) -> Option<u16> {
        self.game_state = GoGameState::Playing;
        player_counter
    }

    fn bot_button_clicked(&mut self, bot_level: Option<u16>) -> Option<u16> {
        bot_level
    }

    fn create_host_button_clicked(&mut self) {
        // Verbindet und erstellt Raum
        if self
            .connect(String::from("ws://localhost:9000"), None)
            .is_err()
        {
            self.set_room_key_text(String::from("Connection failed"));
            return;
        }
        if self.send(r#"{ "type": "CreateRoom" }"#).is_err() {
            self.set_room_key_text(String::from("Communication error"));
            return;
        }
        let json_str = self.wait_one_reply();
        let v: Value = match serde_json::from_str(&json_str) {
            Ok(val) => val,
            Err(_) => {
                self.set_room_key_text(String::from("json parse failed"));
                return;
            }
        };
        let room_id = match v.get("room_id").and_then(|id| id.as_str()) {
            Some(id) => id.to_string(),
            None => {
                self.set_room_key_text(String::from("bad server response"));
                return;
            }
        };
        self.set_room_key_text(room_id);

        self.multiplayer = true;
        self.my_color = Some(Stone::Black);
        self.game_state = GoGameState::WaitingForOpponent;
        self.game = Game::new(19);
        self.status_message = "Schwarz ist am Zug.".to_owned();

        // non-blocking für spiel
        if let Some(ref client) = self.client {
            if let tungstenite::stream::MaybeTlsStream::Plain(ref tcp) = *client.get_ref() {
                let _ = tcp.set_nonblocking(true);
            }
        }
    }

    fn player_count_slider(&mut self, _ui: &mut egui::Ui) -> u16 {
        0
    }

    fn bot_level_slider(&mut self, _ui: &mut egui::Ui) -> u16 {
        0
    }

    fn start_multiplayer_game(&mut self) {
        self.multiplayer = true;
        self.my_color = Some(Stone::White);
        self.game_state = GoGameState::Playing;
        self.game = Game::new(19);
        self.status_message = "Multiplayer gestartet. Schwarz ist am Zug.".to_owned();

        // non-blocking für spiel
        if let Some(ref client) = self.client {
            if let tungstenite::stream::MaybeTlsStream::Plain(ref tcp) = *client.get_ref() {
                let _ = tcp.set_nonblocking(true);
            }
        }
    }
}
