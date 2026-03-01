mod bot;
mod kniffel;

use crate::bot::*;
use crate::kniffel::*;
use game_core::{Game, MultiplayerGame};
use kniffel::{next_player, throw_dice, YahtzeeGame};

use serde_json::Value;
use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

#[derive(PartialEq)]
enum Screen {
    Menu,
    LocalSetup,
    InGame,
    WaitingForOpponent,
}

pub struct KniffelGame {
    screen: Screen,
    players: usize,
    player_buttons: Vec<Vec<Option<u32>>>,
    bots: usize,
    game: kniffel::Game,
    client: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    multiplayer: Option<usize>,
    room_key: String,
}

impl Default for KniffelGame {
    fn default() -> Self {
        Self::new()
    }
}

impl KniffelGame {
    pub fn new() -> Self {
        Self {
            screen: Screen::Menu,
            players: 2,
            player_buttons: vec![vec![None; 13]; 2],
            bots: 0,
            game: <kniffel::Game as YahtzeeGame>::new(2, 0).unwrap(),
            client: None,
            multiplayer: None,
            room_key: String::new(),
        }
    }
}

impl Game for KniffelGame {
    fn name(&self) -> &str {
        "Kniffel"
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        match self.screen {
            Screen::Menu => self.ui_menu(ui),
            Screen::LocalSetup => self.ui_local_setup(ui),
            Screen::InGame => self.ui_game(ui),
            Screen::WaitingForOpponent => self.ui_waiting(ui),
        }
    }
}

impl KniffelGame {
    fn ui_menu(&mut self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let available_height = ui.available_height();

        let button_width = (available_width * 0.3).clamp(300.0, 400.0);
        let button_height = (available_height * 0.08).clamp(50.0, 100.0);
        let button_spacing = 10.0;
        let buffer = 75.0;

        let total_buttons_height = (button_height * 2.0) + button_spacing;
        let center_offset = (ui.available_height() - total_buttons_height) / 2.0 - buffer;

        let text_size = 20.0;
        let button_color = egui::Color32::from_rgb(0, 131, 255);

        ui.horizontal(|ui| {
            ui.label("Schlüssel:");
            ui.add(egui::TextEdit::singleline(&mut self.room_key).desired_width(150.0));
            if ui.button("Beitreten").clicked() {
                self.join_room();
            }
        });

        ui.vertical_centered(|ui| {
            ui.add_space(center_offset);

            let play_local_button = egui::Button::new(
                egui::RichText::new("Lokal Spielen")
                    .size(text_size)
                    .color(egui::Color32::WHITE),
            )
            .fill(button_color)
            .min_size(egui::vec2(button_width, button_height));
            if ui.add(play_local_button).clicked() {
                self.screen = Screen::LocalSetup;
            }

            ui.add_space(button_spacing);

            let create_multiplayer_room_button = egui::Button::new(
                egui::RichText::new("Mehrspieler Raum erstellen")
                    .size(text_size)
                    .color(egui::Color32::WHITE),
            )
            .fill(button_color)
            .min_size(egui::vec2(button_width, button_height));
            if ui.add(create_multiplayer_room_button).clicked() {
                self.create_host_button_clicked();
            }
        });
    }

    fn ui_local_setup(&mut self, ui: &mut egui::Ui) {
        ui.heading("Neues Kniffelspiel erstellen");
        ui.separator();

        for number_of_bots in 0..=3 {
            if self.bots == number_of_bots {
                ui.add(
                    egui::Slider::new(&mut self.players, 1..=4 - number_of_bots).text("Spieler"),
                );
            }
        }

        if self.players == 1 {
            ui.add(egui::Slider::new(&mut self.bots, 1..=3).text("Anzahl Computergegner"));
        } else {
            ui.add(egui::Slider::new(&mut self.bots, 0..=3).text("Anzahl Computergegner"));
        }

        ui.separator();

        if ui.button("Spiel starten").clicked() {
            self.game = <kniffel::Game as YahtzeeGame>::new(self.players, self.bots).unwrap();
            self.player_buttons = vec![vec![None; 13]; self.players + self.bots];
            self.screen = Screen::InGame;
        }

        ui.separator();

        if ui.button("Zurück").clicked() {
            self.screen = Screen::Menu;
        }
    }

    fn ui_waiting(&mut self, ui: &mut egui::Ui) {
        ui.heading("Rust Kniffel - Multiplayer");
        ui.label(format!("Room ID: {}", self.room_key));
        ui.label("Warte auf Gegner...");

        // Non-blocking: auf PlayerJoined msg warten
        if let Some(client) = &mut self.client {
            ui.ctx().request_repaint();

            let received = match client.read() {
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
                        self.start_multiplayer_game();
                    }
                }
            }
        }
    }

    fn ui_game(&mut self, ui: &mut egui::Ui) {
        // Check for incoming multiplayer messages
        if self.multiplayer.is_some() && self.client.is_some() {
            self.check_multiplayer_messages();
        }

        self.process_bot_turns();

        egui::SidePanel::left("kniffel_side_panel").show_inside(ui, |ui| {
            ui.heading("Punktetabelle");
            ui.separator();

            let active_players = self.players + self.bots;


            egui::ScrollArea::vertical().auto_shrink([false; 2])
                                        .show(ui, |ui| {egui::Grid::new("point_table")
                                        .spacing([20.0, 10.0])
                                        .show(ui, |ui| {
                    ui.label("Kategorie");

                    for i in 0..active_players {
                        ui.label(format!("Spieler {}", i + 1));
                    }

                    ui.end_row();

                    let upper_categories = [
                        (0_usize, "Einser", "Addiert die Augenzahl aller Würfel mit Zahl 1"),
                        (1, "Zweier", "Addiert die Augenzahl aller Würfel mit Zahl 2"),
                        (2, "Dreier", "Addiert die Augenzahl aller Würfel mit Zahl 3"),
                        (3, "Vierer", "Addiert die Augenzahl aller Würfel mit Zahl 4"),
                        (4, "Fünfer", "Addiert die Augenzahl aller Würfel mit Zahl 5"),
                        (5, "Sechser", "Addiert die Augenzahl aller Würfel mit Zahl 6"),
                    ];

                    for (category, label, hover) in upper_categories {
                        ui.label(label).on_hover_text(hover);
                        self.render_point_cells(ui, category);
                        ui.end_row();
                    }

                    self.render_seperator_cells(ui);
                    ui.separator();
                    ui.end_row();

                    ui.label("Oben gesamt");
                    self.render_totals(ui, 0);
                    ui.end_row();

                    ui.label("Oben mit Bonus").on_hover_text("Wenn mindestens 63 Punkte oben erreicht wurden, werden 35 Bonuspunkte addiert");
                    self.render_totals(ui, 1);
                    ui.end_row();

                    self.render_seperator_cells(ui);
                    ui.separator();
                    ui.end_row();

                    let lower_categories = [
                        (6_usize, "Dreierpasch", "Mindestens drei gleiche Zahlen, die Augen aller Würfel werden addiert"),
                        (7, "Viererpasch", "Mindestens vier gleiche Zahlen, die Augen aller Würfel werden addiert"),
                        (8, "Full House", "25Pkt: Drei gleiche und zwei gleiche andere Zahlen"),
                        (9, "Kleine Straße", "30Pkt: Folge von vier aufeinander folgenden Würfeln"),
                        (10, "Große Straße", "40Pkt: Folge von fünf aufeinander folgenden Würfeln"),
                        (11, "Kniffel", "50Pkt: 5 gleiche Zahlen"),
                        (12, "Chance", "Summiert alle Augenzahlen"),
                    ];

                    for (category, label, hover) in lower_categories {
                        ui.label(label).on_hover_text(hover);
                        self.render_point_cells(ui, category);
                        ui.end_row();
                    }

                    self.render_seperator_cells(ui);
                    ui.separator();
                    ui.end_row();

                    ui.label("Punkte oben");
                    self.render_totals(ui, 1);
                    ui.end_row();

                    ui.label("Punkte unten");
                    self.render_totals(ui, 2);
                    ui.end_row();
                    self.render_seperator_cells(ui);

                    ui.separator();
                    ui.end_row();

                    ui.label("Punkte gesamt");
                    self.render_totals(ui, 3);
                });

        });
    });

        ui.vertical_centered(|ui| {
            display_current_player(ui, &self.game.clone());

            ui.separator();

            ui.label(format!(
                "Wurf {}/3",
                self.game.current_player.number_of_throws
            ));

            ui.separator();

            if ui.button("Würfeln").clicked() && self.game.current_player.number_of_throws < 3 {
                // Check if it's the local player's turn in multiplayer
                let can_throw = if let Some(local_player_index) = self.multiplayer {
                    local_player_index == self.game.current_player_index
                } else {
                    true
                };

                if can_throw {
                    throw_dice(&mut self.game);

                    // Send dice throw to opponent
                    if self.multiplayer.is_some() {
                        self.send_dice_throw();
                    }
                }
            }

            ui.separator();

            if self.game.current_player.number_of_throws > 0 {
                egui::Grid::new("dice")
                    .spacing([50.0, 10.0])
                    .show(ui, |ui| {
                        self.render_dice(ui);
                    });
            }
        });

        //sichtbar sobald Punktetabelle voll ist

        if point_table_full(&self.game) {
            self.display_winner(ui);
        }
    }
}

impl KniffelGame {
    fn process_bot_turns(&mut self) {
        while self.game.current_player.is_bot() {
            let bot_index = self.game.current_player_index;

            bot_game_turn(&mut self.game);

            //aktualisiere player buttons; finde neu gefüllte Kategorie
            for cat in 0..13 {
                if self.player_buttons[bot_index][cat].is_none()
                    && self.game.all_players[bot_index].point_table.points_thrown[cat].is_some()
                {
                    if let Some(points) =
                        self.game.all_players[bot_index].point_table.points_thrown[cat]
                    {
                        self.player_buttons[bot_index][cat] = Some(points as u32);
                    }
                }
            }
            next_player(&mut self.game);
        }
    }

    fn render_point_cells(&mut self, ui: &mut egui::Ui, category: usize) {
        let active_players = self.players + self.bots;

        for player_index in 0..active_players {
            let current_value = self
                .player_buttons
                .get(player_index)
                .and_then(|row| row.get(category))
                .copied()
                .flatten();

            let is_current_player = player_index == self.game.current_player_index;
            let is_filled = current_value.is_some();

            // In multiplayer mode, only allow interaction if it's the local player's turn
            let is_local_player_turn = if let Some(local_player_index) = self.multiplayer {
                local_player_index == player_index && is_current_player
            } else {
                is_current_player
            };

            let category_label = if is_filled {
                format!("{} Pkt", current_value.unwrap())
            } else {
                "0".to_string()
            };

            let response = ui.add_enabled(
                is_local_player_turn
                    && !is_filled
                    && (self.game.current_player.number_of_throws > 0),
                egui::Button::new(category_label),
            ); //fehlt: dass Würfel gewürfelt werden, ansonsten blocked bis 1. Wurf FIX

            if response.clicked() {
                let points = calculate_points(category, self.game.current_player.dice_throw);
                self.game = add_dice_point_table(&mut self.game, category).clone();

                if let Some(row) = self.player_buttons.get_mut(player_index) {
                    if let Some(slot) = row.get_mut(category) {
                        *slot = Some(points as u32);
                    }
                }

                if let Some(player) = self
                    .game
                    .all_players
                    .get_mut(self.game.current_player_index)
                {
                    player.point_table.points_thrown[category] = Some(points);
                }

                // Send move to opponent in multiplayer
                if self.multiplayer.is_some() {
                    self.send_category_selection(category, points);
                }

                next_player(&mut self.game);

                self.process_bot_turns();
            }
        }
    }

    fn render_seperator_cells(&self, ui: &mut egui::Ui) {
        let active_players = self.players + self.bots;

        for _ in 0..active_players {
            ui.separator();
        }
    }

    fn render_totals(&self, ui: &mut egui::Ui, number: usize) {
        let active_players = self.players + self.bots;

        for player_index in 0..active_players {
            ui.label(format!(
                "{} Pkt",
                self.game.all_players[player_index].point_table.total_points[number]
            ));
        }
    }

    fn render_dice(&mut self, ui: &mut egui::Ui) {
        for dice_index in 0..5 {
            let dice = self.game.current_player.dice_throw[dice_index].eyes;
            let dice_button = {
                if self.game.current_player.dice_throw[dice_index].is_locked {
                    ui.button(format!("{} gesperrt", dice))
                } else {
                    ui.button(format!("{}", dice))
                }
            };

            if dice_button.clicked() {
                // Check if it's the local player's turn in multiplayer
                let can_lock = if let Some(local_player_index) = self.multiplayer {
                    local_player_index == self.game.current_player_index
                } else {
                    true
                };

                if can_lock {
                    self.game = change_blocked_status_dice(&mut self.game, dice_index).clone();

                    // Send dice lock status to opponent
                    if self.multiplayer.is_some() {
                        self.send_dice_lock(dice_index);
                    }
                }
            }
        }
    }

    fn display_winner(&mut self, ui: &mut egui::Ui) {
        let winner_tuple = self.game.winner().unwrap();
        if winner_tuple.0.len() == 1 {
            ui.label(format!(
                "Der Gewinner mit {} Punkten ist Spieler {}",
                winner_tuple.1,
                (winner_tuple.0[0]) + 1
            ));
        } else {
            ui.label(format!(
                "Gleichstand zwischen Spielern {:?}, mit {} Punkten",
                winner_tuple.0, winner_tuple.1
            ));
        }
    }
}

fn display_current_player(ui: &mut egui::Ui, game: &kniffel::Game) {
    let player_num = game.current_player_index + 1;
    ui.label(format!("Spieler {} ist dran", player_num));
}

impl MultiplayerGame for KniffelGame {
    fn on_text(&mut self, str: String) {
        println!("Received: {}", str);

        let v: Value = match serde_json::from_str(&str) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to parse JSON: {}", e);
                return;
            }
        };

        let msg_type = match v.get("type").and_then(|t| t.as_str()) {
            Some(t) => t,
            None => return,
        };

        match msg_type {
            "DiceThrow" => {
                // Receive opponent's dice throw
                if let Some(dice_array) = v["data"]["dice"].as_array() {
                    for (i, dice_val) in dice_array.iter().enumerate() {
                        if i < 5 {
                            if let Some(eyes) = dice_val.as_u64() {
                                self.game.current_player.dice_throw[i].eyes = eyes as u8;
                            }
                        }
                    }
                }
                if let Some(throws) = v["data"]["number_of_throws"].as_u64() {
                    self.game.current_player.number_of_throws = throws as u8;
                }
            }
            "DiceLock" => {
                // Receive opponent's dice lock status
                if let Some(dice_index) = v["data"]["dice_index"].as_u64() {
                    let idx = dice_index as usize;
                    if idx < 5 {
                        self.game = change_blocked_status_dice(&mut self.game, idx).clone();
                    }
                }
            }
            "CategorySelection" => {
                // Receive opponent's category selection
                if let Some(category) = v["data"]["category"].as_u64() {
                    let cat = category as usize;
                    if let Some(points) = v["data"]["points"].as_u64() {
                        let pts = points as u8;

                        self.game = add_dice_point_table(&mut self.game, cat).clone();

                        let player_index = self.game.current_player_index;
                        if let Some(row) = self.player_buttons.get_mut(player_index) {
                            if let Some(slot) = row.get_mut(cat) {
                                *slot = Some(pts as u32);
                            }
                        }

                        if let Some(player) = self.game.all_players.get_mut(player_index) {
                            player.point_table.points_thrown[cat] = Some(pts);
                        }

                        next_player(&mut self.game);
                    }
                }
            }
            _ => {}
        }
    }

    fn local_button_clicked(&mut self, _player_counter: Option<u16>) -> Option<u16> {
        // Switch to local setup screen
        self.screen = Screen::LocalSetup;
        None
    }

    fn bot_button_clicked(&mut self, _bot_level: Option<u16>) -> Option<u16> {
        // Switch to local setup screen
        self.screen = Screen::LocalSetup;
        None
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

    fn player_count_slider(&mut self, _ui: &mut egui::Ui) -> u16 {
        2 // Kniffel multiplayer is always 2 players
    }

    fn bot_level_slider(&mut self, _ui: &mut egui::Ui) -> u16 {
        0 // No bot levels in multiplayer
    }

    fn start_multiplayer_game(&mut self) {
        // Set non-blocking mode
        if let Some(ref client) = self.client {
            if let tungstenite::stream::MaybeTlsStream::Plain(ref tcp) = *client.get_ref() {
                let _ = tcp.set_nonblocking(true);
            }
        }

        // Initialize game for 2 players, no bots
        self.game = <kniffel::Game as YahtzeeGame>::new(2, 0).unwrap();
        self.player_buttons = vec![vec![None; 13]; 2];
        self.players = 2;
        self.bots = 0;

        // Determine if this player is player 0 or 1
        if self.screen == Screen::WaitingForOpponent {
            // Host is player 0
            self.multiplayer = Some(0);
            println!("I am player 0 (host)");
        } else {
            // Joiner is player 1
            self.multiplayer = Some(1);
            println!("I am player 1 (joiner)");
        }

        self.screen = Screen::InGame;
    }

    fn create_host_button_clicked(&mut self) {
        // Connect and create room
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

        self.screen = Screen::WaitingForOpponent;

        // Set non-blocking mode for waiting
        if let Some(ref client) = self.client {
            if let tungstenite::stream::MaybeTlsStream::Plain(ref tcp) = *client.get_ref() {
                let _ = tcp.set_nonblocking(true);
            }
        }
    }
}

impl KniffelGame {
    fn check_multiplayer_messages(&mut self) {
        if self.client.is_none() {
            return;
        }

        // Non-blocking read
        match self.client.as_mut().unwrap().read() {
            Ok(tungstenite::Message::Text(txt)) => {
                self.on_text(txt);
            }
            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No message available, this is fine
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
            }
            _ => {}
        }
    }

    fn send_dice_throw(&mut self) {
        if self.client.is_none() {
            return;
        }

        let dice_values: Vec<u8> = self
            .game
            .current_player
            .dice_throw
            .iter()
            .map(|d| d.eyes)
            .collect();

        let msg = format!(
            r#"{{ "type": "GameMove", "data": {{ "type": "DiceThrow", "dice": {:?}, "number_of_throws": {} }} }}"#,
            dice_values, self.game.current_player.number_of_throws
        );

        if let Err(e) = self.send(&msg) {
            eprintln!("Failed to send dice throw: {}", e);
        }
    }

    fn send_dice_lock(&mut self, dice_index: usize) {
        if self.client.is_none() {
            return;
        }

        let msg = format!(
            r#"{{ "type": "GameMove", "data": {{ "type": "DiceLock", "dice_index": {} }} }}"#,
            dice_index
        );

        if let Err(e) = self.send(&msg) {
            eprintln!("Failed to send dice lock: {}", e);
        }
    }

    fn send_category_selection(&mut self, category: usize, points: u8) {
        if self.client.is_none() {
            return;
        }

        let msg = format!(
            r#"{{ "type": "GameMove", "data": {{ "type": "CategorySelection", "category": {}, "points": {} }} }}"#,
            category, points
        );

        if let Err(e) = self.send(&msg) {
            eprintln!("Failed to send category selection: {}", e);
        }
    }
}
