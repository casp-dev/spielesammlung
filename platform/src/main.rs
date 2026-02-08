use chess::ChessGame;
use eframe::egui;
use game_core::Game;
use go::GoGame;
use kniffel::KniffelGame;
use minesweeper::MinesweeperGame;

use egui::{Color32, RichText, Vec2};

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

                    ui.vertical_centered(|ui| {
                    ui.heading("Wähle ein Spiel:");
                    ui.add_space(20.0);

                ui.horizontal(|ui| {
                    let grid_width = 200.0 * 2.0 + 20.0; // 2 Buttons + Spacing
                    let app_center = (ui.available_width() - (grid_width)) / 2.0; // find middle of the app window
                    ui.add_space(app_center.max(0.0)); // move the grid from the left to the middle

                    egui::Grid::new("menu_grid")
                        .spacing([20.0, 20.0])
                        .show(ui, |ui| {

                            let text_chess = // Chess button
                                RichText::new("♛ Schach ♚").size(30.0).color(Color32::WHITE).strong();
                            let button_chess = egui::Button::new(text_chess)
                                .min_size(Vec2::new(200.0, 150.0))
                                .fill(Color32::LIGHT_BLUE);
                            if ui.add(button_chess).clicked() {
                                self.state = AppState::Playing(Box::new(ChessGame::new()));
                            }

                            let text_go = RichText::new("☯ Go ☯").size(30.0).color(Color32::WHITE).strong(); // Go button
                            let button_go = egui::Button::new(text_go)
                                .min_size(Vec2::new(200.0, 150.0))
                                .fill(Color32::DARK_BLUE);
                            if ui.add(button_go).clicked() {
                                self.state = AppState::Playing(Box::new(GoGame::new()));
                            }

                            ui.end_row();

                            let text_kniffel = // Kniffel Button
                                RichText::new("🎲 Kniffel 🎲").size(30.0).color(Color32::WHITE).strong();
                            let button_kniffel = egui::Button::new(text_kniffel)
                                .min_size(Vec2::new(200.0, 150.0))
                                .fill(Color32::DARK_BLUE);
                            if ui.add(button_kniffel).clicked() {
                                self.state = AppState::Playing(Box::new(KniffelGame::new()));
                            }

                            let text_minesweeper = RichText::new("💣 Minesweeper 🚩") // Minesweeper button
                                .size(20.0)
                                .color(Color32::WHITE)
                                .strong();
                            let button_minesweeper = egui::Button::new(text_minesweeper)
                                .min_size(Vec2::new(200.0, 150.0))
                                .fill(Color32::LIGHT_BLUE);
                            if ui.add(button_minesweeper).clicked() {
                                self.state = AppState::Playing(Box::new(MinesweeperGame::new()));
                            }
                        });
                });
                 });
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
