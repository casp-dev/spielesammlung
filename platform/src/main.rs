use chess::ChessGame;
use eframe::egui;
use game_core::Game;
use go::GoGame;
use kniffel::KniffelGame;
use minesweeper::MinesweeperGame;
mod multiplayer;
use multiplayer::Multiplayer;
mod websocketclient;

use std::error::Error;

enum AppState {
    Menu,
    Playing(Box<dyn Game>),
}

struct PlatformApp {
    state: AppState,
}

impl PlatformApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: AppState::Menu,
        }
    }
}

impl eframe::App for PlatformApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| match &mut self.state {
            AppState::Menu => {
                ui.heading("Spielesammlung");
                ui.label("Wähle ein Spiel:");

                if ui.button("Schach").clicked() {
                    self.state = AppState::Playing(Box::new(ChessGame::new()));
                }

                if ui.button("Go").clicked() {
                    self.state = AppState::Playing(Box::new(GoGame::new()));
                }

                if ui.button("Kniffel").clicked() {
                    self.state = AppState::Playing(Box::new(KniffelGame::new()));
                }
                if ui.button("Minesweeper").clicked() {
                    self.state = AppState::Playing(Box::new(MinesweeperGame::new()));
                }
            }
            AppState::Playing(game) => {
                if ui.button("Zurück zum Menü").clicked() {
                    self.state = AppState::Menu;
                    return;
                }

                ui.separator();
                game.ui(ui);
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Multiplayer WebSocket Test ---");
    println!("1) Create host");
    println!("2) Connect to existing key");
    println!("Choose (1 or 2):");

    use tokio::io::{AsyncBufReadExt, BufReader};
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    let choice = lines
        .next_line()
        .await?
        .unwrap_or_default()
        .trim()
        .to_string();

    let mut multiplayer = Multiplayer::new();

    match choice.as_str() {
        "1" => {
            println!("Creating host…");
            multiplayer.create_host().await?;
        }
        "2" => {
            println!("Enter room key:");
            let key = lines
                .next_line()
                .await?
                .unwrap_or_default()
                .trim()
                .to_string();

            multiplayer.connect_with(key).await?;
        }
        _ => {
            println!("Invalid choice");
            return Ok(());
        }
    }

    // IMPORTANT: keep process alive
    println!("Connected. Type messages and press Enter.");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
