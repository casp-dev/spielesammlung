use egui::Ui;

use std::error::Error;
use tungstenite::{connect, Message, WebSocket};
use tungstenite::client::IntoClientRequest;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::http::header::HeaderName;
use std::net::TcpStream;

pub trait Game {
    fn name(&self) -> &str;
    fn ui(&mut self, ui: &mut Ui);
}

#[allow(clippy::needless_async)]
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

    fn wait_one_reply(&mut self) {
        match self.get_client().read() {
            Ok(Message::Text(txt)) => {
                println!("Received: {}", txt);
            }
            Ok(_) => {}
            Err(e) => eprintln!("WebSocket error: {}", e),
        }
    }

    fn multipalyer_ui(&mut self, ui: &mut Ui, bot_level: bool, player_count: bool) {
        let available = ui.available_size();

        let button_width = available.x * 0.5;
        let button_height = available.y * 0.1;

        let play_local =
            egui::Button::new("Play Local").min_size(egui::vec2(button_width, button_height));
        let play_vs_bot =
            egui::Button::new("Play vs Bot").min_size(egui::vec2(button_width, button_height));
        let create_muliplayer_room = egui::Button::new("Create Multiplayer Room")
            .min_size(egui::vec2(button_width, button_height));
        if player_count {
            let count = self.player_count_slider(ui);
            if ui.add(play_local).clicked() {
                self.local_button_clicked(Some(count));
            }
        } else {
            if ui.add(play_local).clicked() {
                self.local_button_clicked(None);
            }
        }
        if bot_level {
            let level = self.bot_level_slider(ui);
            if ui.add(play_vs_bot).clicked() {
                self.bot_button_clicked(Some(level));
            }
        } else {
            if ui.add(play_vs_bot).clicked() {
                self.bot_button_clicked(None);
            }
        }
        ui.horizontal(|ui| {
            if ui.add(create_muliplayer_room).clicked() {
                self.create_host_button_clicked();
            }

            ui.add(
                egui::TextEdit::singleline(self.get_room_key_text())
                    .desired_width(button_width * 0.4),
            );

            if ui
                .add(
                    egui::Button::new("Join")
                        .min_size(egui::vec2(button_width * 0.1, button_height)),
                )
                .clicked()
            {
                let room = self.get_room_key_text().clone();
                self.join_room(room);
            }
        });
    }

    fn player_count_slider(&mut self, ui: &mut Ui) -> u16;
    fn bot_level_slider(&mut self, ui: &mut Ui) -> u16;
    fn local_button_clicked(&mut self, player_counter: Option<u16>) -> Option<u16>;
    fn bot_button_clicked(&mut self, bot_level: Option<u16>) -> Option<u16>;
    fn create_host_button_clicked(&mut self) {
        //TODO Server starten...
        self.set_room_key_text(String::from("Create Room key"));
    }
    fn start_multiplayer_game(&mut self);

    fn get_room_key_text(&mut self) -> &mut String;
    fn set_room_key_text(&mut self, text: String);

    fn join_room(&mut self, room: String) {
        let url = format!("ws://localhost:8080/{}", room);
        let header_value = None; 
        if self.connect(url, header_value).is_ok() {
            println!("Successfully connected to the room: {}", room);
            self.start_multiplayer_game();
        } else {
            eprintln!("Failed to connect to the room: {}", room);
        }
    }
}
