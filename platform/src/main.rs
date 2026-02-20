use chess::ChessGame;
use eframe::egui;
use game_core::Game;
use go::GoGame;
use kniffel::KniffelGame;
use minesweeper::MinesweeperGame;

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

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Spielesammlung",
        native_options,
        Box::new(|cc| Ok(Box::new(PlatformApp::new(cc)))),
    )
}
