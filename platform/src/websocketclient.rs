use std::error::Error;
use std::sync::Arc;

use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};

use tokio::net::TcpStream;
use tokio::sync::Mutex;

use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
    MaybeTlsStream, WebSocketStream,
};

pub struct WebSocketClient {
    sender: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
}

impl WebSocketClient {
    /// Connect and start listening in a background task
    pub async fn connect(room_key: &str) -> Result<Self, Box<dyn Error>> {
        let mut request =
            format!("wss://usersockets.luckperms.net/{room_key}").into_client_request()?;

        request
            .headers_mut()
            .insert("Origin", "https://luckperms.net".parse()?);

        println!("Connecting…");
        let (ws_stream, _) = connect_async(request).await?;
        let (write, mut read) = ws_stream.split();

        let sender = Arc::new(Mutex::new(write));

        //listen, hier schnittstelle zu game implementieren
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("Received: {}", text);
                    }
                    Ok(Message::Binary(bin)) => {
                        println!("Received binary ({} bytes)", bin.len());
                    }
                    Ok(Message::Close(_)) => {
                        println!("Connection closed");
                        break;
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(WebSocketClient { sender })
    }

    ///this fct sends the input to the dataset
    pub async fn send(&self, text: String) -> Result<(), Box<dyn Error>> {
        if text.is_empty() {
            return Ok(());
        }

        let mut sender = self.sender.lock().await;
        sender.send(Message::Text(text.to_string())).await?;
        println!("sendet: {text}");

        Ok(())
    }
}
    // /// Connect and start listening in a background task
    // pub async fn connect(room_key: &str,on_text: fn(String)) -> Result<Self, Box<dyn Error>> {
    //     let mut request =
    //         format!("wss://usersockets.luckperms.net/{room_key}").into_client_request()?;

    //     //build a request the server understands
    //     request
    //         .headers_mut()
    //         .insert("Origin", "https://luckperms.net".parse()?);

    //     println!("Connecting…");
    //     let (ws_stream, _) = connect_async(request).await?;
    //     let (write, mut read) = ws_stream.split();

    //     let sender = Arc::new(Mutex::new(write));

    //     //listen, hier schnittstelle zu game implementieren
    //     tokio::spawn(async move {
    //         while let Some(msg) = read.next().await {
    //             match msg {
    //                 Ok(Message::Text(text)) => {
    //                     if text != "close" {
    //                         println!("Received: {text}");
    //                         on_text(text);
    //                     } else {
    //                         break;
    //                     }
    //                 }
    //                 Ok(Message::Binary(bin)) => {
    //                     println!("Received binary ({} bytes)", bin.len());
    //                 }
    //                 Ok(Message::Close(_)) => {
    //                     println!("Connection closed");
    //                     break;
    //                 }
    //                 Err(e) => {
    //                     eprintln!("WebSocket error: {}", e);
    //                     break;
    //                 }
    //                 _ => {}
    //             }
    //         }
    //     });

    //     Ok(WebSocketClient { sender })
    // }
