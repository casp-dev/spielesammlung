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
pub trait MultiplayerGame {
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
}
