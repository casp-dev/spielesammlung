use egui::Ui;

use serde_json::Value;
use std::error::Error;
use std::net::TcpStream;
use tungstenite::client::IntoClientRequest;
use tungstenite::http::header::HeaderName;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

pub trait Game {
    fn name(&self) -> &str;
    fn ui(&mut self, ui: &mut Ui);
}

pub trait MultiplayerGame: Game {
    fn on_text(&mut self, str: String);
    fn set_client(&mut self, client: WebSocket<MaybeTlsStream<TcpStream>>);
    fn get_client(&mut self) -> &mut WebSocket<MaybeTlsStream<TcpStream>>;
    fn connect(
        &mut self,
        url: String,
        header_value: Option<(String, String)>,
    ) -> Result<(), Box<dyn Error>> {
        let mut request = url.into_client_request()?;

        if let Some((name, value)) = header_value {
            let name = HeaderName::from_bytes(name.as_bytes())?;
            let value = value.parse()?;
            request.headers_mut().insert(name, value);
        }

        println!("Connecting…");

        let (ws_stream, _) = connect(request)?;

        println!("Connected to the server");

        self.set_client(ws_stream);
        Ok(())
    }

    fn send(&mut self, text: &str) -> Result<(), Box<dyn Error>> {
        if text.is_empty() {
            return Ok(());
        }
        self.get_client().send(Message::Text(text.to_string()))?;
        println!("Sent: {}", text);
        Ok(())
    }

    fn wait_one_reply(&mut self) -> String {
        match self.get_client().read() {
            Ok(Message::Text(txt)) => txt,
            Ok(_) => {
                eprintln!("Received non-text message");
                String::new()
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                String::new()
            }
        }
    }

    fn wait_one_reply_game(&mut self) {
        match self.get_client().read() {
            Ok(Message::Text(txt)) => {
                self.on_text(txt);
            }
            Ok(_) => {
                //eprintln!("Received non-text message");
            }
            Err(_) => {
                //eprintln!("WebSocket error: {}", e);
            }
        }
    }

    fn gamemode_selection_ui(&mut self, ui: &mut Ui, bot_level: bool, player_count: bool) {
        let available_width = ui.available_width();
        let available_height = ui.available_height();

        let button_width = (available_width * 0.3).clamp(300.0, 400.0);
        let button_height = (available_height * 0.08).clamp(50.0, 100.0);
        let button_spacing = 10.0;
        let buffer = 75.0;

        let total_buttons_height = (button_height * 3.0) + (button_spacing * 2.0);
        let center_offset = (ui.available_height() - total_buttons_height) / 2.0 - buffer;

        let button_text_size = 20.0;

        let estimated_bot_selection_width = 210.0; // estimated width of the bot slider + text + textfield
        let bot_selection_horizontal_offset = available_width - estimated_bot_selection_width;
        let estimated_multiplayer_element_height = 25.0; // used to move the bot slider to the top of the corner

        let gamemode_button_color = egui::Color32::from_rgb(0, 131, 255);

        let is_darkmode_on = ui.visuals().dark_mode;
        let frame_color = if is_darkmode_on {
            egui::Color32::from_gray(50)
        } else {
            egui::Color32::from_gray(220)
        };

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Schlüssel:").size(12.5));

            egui::Frame::none()
                .fill(frame_color)
                .rounding(5.0)
                .inner_margin(egui::vec2(8.0, 1.0))
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(self.get_room_key_text())
                            .desired_width(150.0)
                            .frame(false),
                    );
                });

            let join_btn = egui::Button::new(egui::RichText::new("Beitreten"));

            if ui.add(join_btn).clicked() {
                self.join_room();
            }
        });

        let mut bot_level_val = None;

        if bot_level {
            ui.horizontal(|ui| {
                ui.add_space(bot_selection_horizontal_offset);

                ui.vertical(|ui| {
                    ui.add_space(-estimated_multiplayer_element_height);
                    bot_level_val = Some(self.bot_level_slider(ui));
                });
            });
        }

        let mut player_count_val = None;

        if player_count {
            ui.vertical(|ui| {
                player_count_val = Some(self.player_count_slider(ui));
            });
        }

        ui.vertical_centered(|ui| {
            ui.add_space(center_offset);

            let play_local_button = egui::Button::new(
                egui::RichText::new("Lokal Spielen")
                    .size(button_text_size)
                    .color(egui::Color32::WHITE),
            )
            .fill(gamemode_button_color)
            .min_size(egui::vec2(button_width, button_height));

            if ui.add(play_local_button).clicked() {
                self.local_button_clicked(player_count_val);
            }

            ui.add_space(button_spacing);

            let play_vs_bot_button = egui::Button::new(
                egui::RichText::new(if let Some(level) = bot_level_val {
                    format!("Spiele gegen einen Bot (Level {})", level)
                } else {
                    "Spiele gegen einen Bot".to_string()
                })
                .size(button_text_size)
                .color(egui::Color32::WHITE),
            )
            .fill(gamemode_button_color)
            .min_size(egui::vec2(button_width, button_height));

            if ui.add(play_vs_bot_button).clicked() {
                self.bot_button_clicked(bot_level_val);
            }

            ui.add_space(button_spacing);

            let create_multiplayer_room_button = egui::Button::new(
                egui::RichText::new("Mehrspieler Raum erstellen")
                    .size(button_text_size)
                    .color(egui::Color32::WHITE),
            )
            .fill(gamemode_button_color)
            .min_size(egui::vec2(button_width, button_height));

            if ui.add(create_multiplayer_room_button).clicked() {
                self.create_host_button_clicked();
            }
        });
    }

    fn player_count_slider(&mut self, ui: &mut Ui) -> u16;
    fn bot_level_slider(&mut self, ui: &mut Ui) -> u16;
    fn local_button_clicked(&mut self, player_counter: Option<u16>) -> Option<u16>;
    fn bot_button_clicked(&mut self, bot_level: Option<u16>) -> Option<u16>;
    fn create_host_button_clicked(&mut self) {
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
    }
    fn start_multiplayer_game(&mut self);

    fn get_room_key_text(&mut self) -> &mut String;
    fn set_room_key_text(&mut self, text: String);

    fn waiting_screen_ui(&mut self, ui: &mut Ui, game_name: &str) {
        let is_dark = ui.visuals().dark_mode;
        let accent = egui::Color32::from_rgb(0, 131, 255);

        let available_width = ui.available_width();
        let card_width = available_width.min(420.0);

        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.15);
            ui.label(
                egui::RichText::new(format!("{} — Mehrspieler", game_name))
                    .size(26.0)
                    .strong()
                    .color(egui::Color32::BLACK),
            );

            ui.add_space(24.0);

            let card_fill = if is_dark {
                egui::Color32::from_gray(35)
            } else {
                egui::Color32::from_gray(245)
            };
            let card_stroke = if is_dark {
                egui::Stroke::new(1.0, egui::Color32::from_gray(60))
            } else {
                egui::Stroke::new(1.0, egui::Color32::from_gray(200))
            };

            egui::Frame::none()
                .fill(card_fill)
                .stroke(card_stroke)
                .rounding(12.0)
                .inner_margin(egui::vec2(32.0, 28.0))
                .show(ui, |ui| {
                    ui.set_width(card_width);
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Raum-Schlüssel").size(13.0).color(
                            if is_dark {
                                egui::Color32::from_gray(160)
                            } else {
                                egui::Color32::from_gray(100)
                            },
                        ));

                        ui.add_space(8.0);

                        let room_key = self.get_room_key_text().clone();
                        let id_bg = if is_dark {
                            egui::Color32::from_gray(22)
                        } else {
                            egui::Color32::WHITE
                        };

                        egui::Frame::none()
                            .fill(id_bg)
                            .rounding(8.0)
                            .inner_margin(egui::vec2(20.0, 12.0))
                            .stroke(egui::Stroke::new(1.5, accent.linear_multiply(0.4)))
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new(&room_key)
                                            .size(28.0)
                                            .strong()
                                            .monospace(),
                                    );
                                });
                            });

                        ui.add_space(12.0);

                        ui.label(
                            egui::RichText::new("Teile diesen Schlüssel mit deinem Gegner")
                                .size(12.5)
                                .italics()
                                .color(if is_dark {
                                    egui::Color32::from_gray(140)
                                } else {
                                    egui::Color32::from_gray(120)
                                }),
                        );
                    });
                });
        });
    }

    fn join_room(&mut self) {
        let room_key = self.get_room_key_text().clone();
        if room_key.is_empty() {
            self.set_room_key_text(String::from("Room key here"));
            return;
        }
        let url = format!("ws://localhost:9000/{}", self.get_room_key_text());
        let header_value = None;
        if self.connect(url, header_value).is_ok() {
            let join_msg = format!(r#"{{ "type": "JoinRoom", "room_id": "{}" }}"#, room_key);
            if self.send(&join_msg).is_err() {
                self.set_room_key_text(String::from("Communication error"));
                return;
            }
            let reply = self.wait_one_reply();
            println!("Joined room: {}", reply);
            self.start_multiplayer_game();
        } else {
            self.set_room_key_text(String::from("No Host for the key"));
        }
    }
}
