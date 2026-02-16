use egui::Ui;

use std::error::Error;
use std::sync::Arc;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};

use tokio::net::TcpStream;
use tokio::sync::Mutex;

use tokio_tungstenite::tungstenite::http::header::HeaderName;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
    MaybeTlsStream, WebSocketStream,
};

pub trait Game {
    fn name(&self) -> &str;
    fn ui(&mut self, ui: &mut Ui);
}

#[allow(clippy::needless_async)]
pub trait MultiplayerGame: Game {
    fn on_text(&mut self, str: String);
    fn set_sender(
        &mut self,
        sender: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    );
    fn set_reader(&mut self, reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>);
    /// Connect and start listening in a background task
    async fn connect(
        game: Arc<Mutex<Self>>,
        url: String,
        header_value: Option<(String, String)>,
    ) -> Result<(), Box<dyn Error>> {
        let mut request = url.clone().into_client_request()?;

        if let Some((name, value)) = header_value {
            let name = HeaderName::from_bytes(name.as_bytes())?;
            let value = value.parse()?;

            request.headers_mut().insert(name, value);
        }

        println!("Connecting…");
        let (ws_stream, _) = connect_async(request).await?;
        let (write, read) = ws_stream.split();

        let sender = Arc::new(Mutex::new(write));
        let game_clone = game.clone();
        let mut mut_game = game_clone.lock().await;
        mut_game.set_sender(sender);
        mut_game.set_reader(read);

        Ok(())
    }

    async fn wait_one_reply(
        &mut self,
        reader: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) {
        if let Some(msg) = reader.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received: {}", text);
                    self.on_text(text);
                }
                Ok(_) => {}
                Err(e) => eprintln!("WebSocket error: {}", e),
            }
        }
    }

    ///this fct sends the input to the dataset
    async fn send(
        sender: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
        text: String,
    ) -> Result<(), Box<dyn Error>> {
        if text.is_empty() {
            return Ok(());
        }

        let mut sender = sender.lock().await;
        sender.send(Message::Text(text.clone())).await?;
        println!("sendet: {text}");

        Ok(())
    }

    fn multipalyer_ui(&mut self, ui: &mut Ui, bot_level: bool, player_count: bool) {
        let available = ui.available_size();

        let button_width = available.x * 0.5;
        let button_height = available.y * 0.1;

        let play_local =
            egui::Button::new("Play Local").min_size(egui::vec2(button_width, button_height));
        let play_vs_bot =
            egui::Button::new("Play vs Bot").min_size(egui::vec2(button_width, button_height));
        let play_multiplayer =
            egui::Button::new("Play Online").min_size(egui::vec2(button_width, button_height));
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
        if ui.add(play_multiplayer).clicked() {
            self.multiplayer_button_clicked();
        }
    }

    fn player_count_slider(&mut self,ui: &mut Ui) -> u16;
    fn bot_level_slider(&mut self,ui: &mut Ui) -> u16;
    fn local_button_clicked(&mut self, player_counter: Option<u16>) -> Option<u16>;
    fn bot_button_clicked(&mut self, bot_level: Option<u16>) -> Option<u16>;
    fn multiplayer_button_clicked(&mut self);
}
