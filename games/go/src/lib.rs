mod ai;
mod game;

use ai::get_best_move;
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

#[allow(dead_code)]
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
    ai_stats_message: String,
    ai_enabled: bool,
    // Multiplayer
    game_state: GoGameState,
    client: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    room_key: String,
    multiplayer: bool,
    my_color: Option<Stone>,
}

impl Default for GoGame {
    fn default() -> Self {
        Self {
            game: Game::new(19), // Standard 19x19 brett
            status_message: "Spiel gestartet. Schwarz ist am Zug.".to_owned(),
            ai_stats_message: String::new(),
            ai_enabled: false,
            // Multiplayer
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
                self.multiplayer_ui(ui, false, false);
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
                let available_size = ui.available_size();
                let board_size = (available_size.y - 20.0).min(available_size.x - 200.0);

                ui.horizontal(|ui| {
                    // === Board (links) ===
                    let (response, painter) = ui.allocate_painter(
                        egui::Vec2::new(board_size, board_size),
                        egui::Sense::click(),
                    );

                    let rect = response.rect;
                    let grid_size = self.game.board.size;

                    // farbe
                    painter.rect_filled(
                        rect,
                        egui::Rounding::same(20.0),
                        egui::Color32::from_rgb(222, 184, 135),
                    );

                    let padding = board_size * 0.05;
                    let grid_rect = rect.shrink(padding);
                    let cell_size = grid_rect.width() / (grid_size as f32 - 1.0);

                    // Raster
                    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50));
                    for i in 0..grid_size {
                        let pos = i as f32 * cell_size;

                        painter.line_segment(
                            [
                                grid_rect.min + egui::vec2(pos, 0.0),
                                grid_rect.min + egui::vec2(pos, grid_rect.height()),
                            ],
                            stroke,
                        );

                        painter.line_segment(
                            [
                                grid_rect.min + egui::vec2(0.0, pos),
                                grid_rect.min + egui::vec2(grid_rect.width(), pos),
                            ],
                            stroke,
                        );
                    }

                    // hoshi punkte
                    if grid_size == 19 {
                        let stars = [3, 9, 15];
                        for &y in &stars {
                            for &x in &stars {
                                let center = grid_rect.min
                                    + egui::vec2(x as f32 * cell_size, y as f32 * cell_size);
                                painter.circle_filled(
                                    center,
                                    cell_size * 0.15,
                                    egui::Color32::BLACK,
                                );
                            }
                        }
                    }

                    // Steine
                    for y in 0..grid_size {
                        for x in 0..grid_size {
                            if let Some(stone) = self.game.board.get(x, y) {
                                let center = grid_rect.min
                                    + egui::vec2(x as f32 * cell_size, y as f32 * cell_size);
                                let stone_radius = cell_size * 0.45;

                                // Schatten
                                painter.circle_filled(
                                    center + egui::vec2(2.0, 2.0),
                                    stone_radius,
                                    egui::Color32::from_black_alpha(50),
                                );

                                match stone {
                                    Stone::Black => {
                                        painter.circle_filled(
                                            center,
                                            stone_radius,
                                            egui::Color32::BLACK,
                                        );
                                        painter.circle_filled(
                                            center
                                                - egui::vec2(
                                                    stone_radius * 0.3,
                                                    stone_radius * 0.3,
                                                ),
                                            stone_radius * 0.2,
                                            egui::Color32::from_white_alpha(30),
                                        );
                                    }
                                    Stone::White => {
                                        painter.circle_filled(
                                            center,
                                            stone_radius,
                                            egui::Color32::WHITE,
                                        );
                                        painter.circle_stroke(
                                            center,
                                            stone_radius,
                                            egui::Stroke::new(1.0, egui::Color32::GRAY),
                                        );
                                    }
                                };
                            }
                        }
                    }

                    // Letzter Zug Mark
                    if let Some((lx, ly)) = self.game.last_move {
                        if let Some(stone) = self.game.board.get(lx, ly) {
                            let center = grid_rect.min
                                + egui::vec2(lx as f32 * cell_size, ly as f32 * cell_size);
                            let marker_radius = cell_size * 0.1;
                            let marker_color = match stone {
                                Stone::Black => egui::Color32::WHITE,
                                Stone::White => egui::Color32::BLACK,
                            };
                            painter.circle_filled(center, marker_radius, marker_color);
                        }
                    }

                    // Hover
                    if let Some(pos) = response.hover_pos() {
                        if !self.game.game_over && rect.contains(pos) {
                            let relative_pos = pos - grid_rect.min;
                            let x_f = relative_pos.x / cell_size;
                            let y_f = relative_pos.y / cell_size;

                            let x = x_f.round() as i32;
                            let y = y_f.round() as i32;

                            if x >= 0
                                && x < grid_size as i32
                                && y >= 0
                                && y < grid_size as i32
                            {
                                if self.game.board.get(x as usize, y as usize).is_none() {
                                    let center = grid_rect.min
                                        + egui::vec2(x as f32 * cell_size, y as f32 * cell_size);
                                    let color = match self.game.current_turn {
                                        Stone::Black => egui::Color32::from_black_alpha(100),
                                        Stone::White => egui::Color32::from_white_alpha(100),
                                    };
                                    painter.circle_filled(center, cell_size * 0.4, color);
                                }
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
                            let relative_pos = pos - grid_rect.min;
                            let x_f = relative_pos.x / cell_size;
                            let y_f = relative_pos.y / cell_size;

                            let x = x_f.round() as i32;
                            let y = y_f.round() as i32;

                            if x >= 0
                                && x < grid_size as i32
                                && y >= 0
                                && y < grid_size as i32
                            {
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

                                        // AI auto-play (nur local)
                                        if self.ai_enabled
                                            && !self.multiplayer
                                            && !self.game.game_over
                                        {
                                            let (best_move, stats) =
                                                get_best_move(&self.game, 1000);
                                            if let Some((x, y)) = best_move {
                                                match self.game.place_stone(x, y) {
                                                    Ok(_) => {
                                                        self.status_message = format!(
                                                            "AI spielt ({}, {}). {:?} ist am Zug.",
                                                            x, y, self.game.current_turn
                                                        );
                                                        let top_moves_str: String = stats
                                                            .top_moves
                                                            .iter()
                                                            .take(3)
                                                            .map(|(m, v, s)| {
                                                                format!(
                                                                    "({},{}):{}/{:.2}",
                                                                    m.0, m.1, v, s
                                                                )
                                                            })
                                                            .collect::<Vec<_>>()
                                                            .join(", ");
                                                        self.ai_stats_message = format!(
                                                            "MCTS: {} Iterationen, Top: {}",
                                                            stats.iterations, top_moves_str
                                                        );
                                                    }
                                                    Err(e) => {
                                                        self.status_message =
                                                            format!("AI Zug ungültig: {}", e);
                                                    }
                                                }
                                            } else {
                                                self.game.pass();
                                                self.status_message = "AI passt.".to_owned();
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        self.status_message =
                                            format!("Ungültiger Zug: {}", e);
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

                    ui.add_space(12.0);

                    // === Info-Panel (rechts) ===
                    ui.vertical(|ui| {
                        ui.set_min_width(170.0);

                        // Zug-Anzeige
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let turn_text = match self.game.current_turn {
                                    Stone::Black => "Schwarz",
                                    Stone::White => "Weiß",
                                };
                                let (stone_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(14.0, 14.0),
                                    egui::Sense::hover(),
                                );
                                let c = stone_rect.center();
                                match self.game.current_turn {
                                    Stone::Black => {
                                        ui.painter()
                                            .circle_filled(c, 7.0, egui::Color32::BLACK);
                                        ui.painter().circle_stroke(
                                            c,
                                            7.0,
                                            egui::Stroke::new(
                                                1.0,
                                                egui::Color32::from_gray(80),
                                            ),
                                        );
                                    }
                                    Stone::White => {
                                        ui.painter()
                                            .circle_filled(c, 7.0, egui::Color32::WHITE);
                                        ui.painter().circle_stroke(
                                            c,
                                            7.0,
                                            egui::Stroke::new(1.0, egui::Color32::GRAY),
                                        );
                                    }
                                }
                                ui.label(
                                    egui::RichText::new(format!("{} am Zug", turn_text))
                                        .strong(),
                                );
                            });
                        });

                        ui.add_space(4.0);

                        // Gefangene
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Gefangen").strong());
                            ui.horizontal(|ui| {
                                let (b_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(10.0, 10.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(
                                    b_rect.center(),
                                    5.0,
                                    egui::Color32::BLACK,
                                );
                                ui.painter().circle_stroke(
                                    b_rect.center(),
                                    5.0,
                                    egui::Stroke::new(
                                        0.5,
                                        egui::Color32::from_gray(80),
                                    ),
                                );
                                ui.label(format!("× {}", self.game.captured_black));

                                ui.add_space(8.0);

                                let (w_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(10.0, 10.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(
                                    w_rect.center(),
                                    5.0,
                                    egui::Color32::WHITE,
                                );
                                ui.painter().circle_stroke(
                                    w_rect.center(),
                                    5.0,
                                    egui::Stroke::new(0.5, egui::Color32::GRAY),
                                );
                                ui.label(format!("× {}", self.game.captured_white));
                            });
                        });

                        ui.add_space(8.0);

                        // Buttons
                        let btn_size = egui::vec2(170.0, 28.0);

                        if ui
                            .add_sized(btn_size, egui::Button::new("Passen"))
                            .clicked()
                        {
                            self.game.pass();
                            if self.game.game_over {
                                let (b_score, w_score) = self.game.calculate_score();
                                self.status_message = format!(
                                    "Spiel vorbei! Schwarz {:.1} — Weiß {:.1}",
                                    b_score, w_score
                                );
                            } else {
                                self.status_message = format!(
                                    "Gepasst. {:?} ist am Zug.",
                                    self.game.current_turn
                                );
                            }
                        }

                        ui.add_space(4.0);

                        if ui
                            .add_sized(btn_size, egui::Button::new("Neustarten"))
                            .clicked()
                        {
                            self.game = Game::new(19);
                            self.status_message =
                                "Spiel neugestartet. Schwarz ist am Zug.".to_owned();
                            self.ai_stats_message.clear();
                        }

                        ui.add_space(4.0);
                        if !self.multiplayer {
                            ui.checkbox(&mut self.ai_enabled, "AI Gegner");
                        }

                        ui.add_space(8.0);

                        // Status
                        if !self.status_message.is_empty()
                            || !self.ai_stats_message.is_empty()
                        {
                            ui.group(|ui| {
                                ui.label(&self.status_message);
                                if !self.ai_stats_message.is_empty() {
                                    ui.label(
                                        egui::RichText::new(&self.ai_stats_message).small(),
                                    );
                                }
                            });
                        }

                        // Spiel-Ende
                        if self.game.game_over {
                            ui.add_space(8.0);
                            let (b_score, w_score) = self.game.calculate_score();
                            let winner = if b_score > w_score {
                                "Schwarz gewinnt!"
                            } else {
                                "Weiß gewinnt!"
                            };
                            ui.group(|ui| {
                                ui.label(
                                    egui::RichText::new(winner).strong().size(16.0),
                                );
                                ui.label(format!(
                                    "Schwarz: {:.1}  ·  Weiß: {:.1}",
                                    b_score, w_score
                                ));
                            });
                        }
                    });
                });
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
