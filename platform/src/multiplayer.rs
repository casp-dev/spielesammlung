use reqwest::Client;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

use crate::websocketclient::WebSocketClient;

pub struct Multiplayer {
    pub key: String,
    client: Client,
    websocket_client: Option<WebSocketClient>,
}

impl Multiplayer {
    pub fn new() -> Self {
        Multiplayer {
            key: String::new(),
            client: Client::new(),
            websocket_client: None,
        }
    }

    ///connect to the multiplayer server with a specific key
    pub async fn connect_with(&mut self, key: String) -> Result<(), Box<dyn std::error::Error>> {
        self.key = key;
        self.connect().await
    }

    ///connect to the multiplayer server
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Connecting to URL: {}", self.key);

        //connects the webclient with the key
        let ws_client = WebSocketClient::connect(self.key.clone(),format!("wss://usersockets.luckperms.net/{}",self.key),Some((String::from("Origin"),String::from("https://luckperms.net"))))
            .await
            .expect("Failed to connect to WebSocket server");

        self.websocket_client = Some(ws_client);

        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();

        while let Some(line) = lines.next_line().await? {
            self.websocket_client.as_mut().unwrap().send(line).await?;
        }

        Ok(())
    }

    ///create a multiplayer host
    pub async fn create_host(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = "https://usersockets.luckperms.net/create";
        println!("Creating multiplayer host...");
        let resp = self.client.get(url).send().await?;
        print!("Response Status: {}\n", resp.status());
        let message = resp.text().await?;
        self.key = self.seperate_key(message);
        println!("key: {}", self.key);
        self.connect().await
    }

    fn seperate_key(&mut self, mut key: String) -> String {
        key.replace_range(0..=7, "");
        key.replace_range(key.len() - 2..key.len(), "");
        key
    }

}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn test_create_host() {
//         let mut multiplayer = Multiplayer::new();
//         multiplayer.create_host().awa
//     }
// }
