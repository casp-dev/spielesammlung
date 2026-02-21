use chess::ChessGame;
use eframe::egui;
use game_core::Game;
use go::GoGame;
use kniffel::KniffelGame;
use minesweeper::MinesweeperGame;

use egui::{Color32, RichText, Vec2};

#[derive(PartialEq)]
enum AppTheme {
    Dark,
    Light,
}

enum AppState {
    Menu,
    Playing(Box<dyn Game>),
}

struct PlatformApp {
    state: AppState,
    theme: AppTheme,
}

impl PlatformApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: AppState::Menu,
            theme: AppTheme::Dark, // Default theme is dark
        }
    }
}

impl eframe::App for PlatformApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| match &mut self.state {

            AppState::Menu => {

            match self.theme {
                AppTheme::Light => {
                ctx.set_visuals(egui::Visuals::light());
                ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Light));
                }

                AppTheme::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
                ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));
                }
            }

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {

                    if ui.selectable_label(self.theme == AppTheme::Light, "Light").clicked() {
                        self.theme = AppTheme::Light;
                    }

                    if ui.selectable_label(self.theme == AppTheme::Dark, "Dark").clicked() {
                        self.theme = AppTheme::Dark;
                    }
                });
            });

            ui.separator();

            ui.vertical_centered(|ui| {

                    let grid_hight = 150.0 * 2.0 + 20.0 + 80.0; // 2 Buttons + Spacing + Heading
                    let app_center_hight = (ui.available_height() - (grid_hight)) / 2.0; // find middle of the app window
                    ui.add_space(app_center_hight.max(0.0)); // move the grid form the top to the middle

                    ui.heading("Wähle ein Spiel:");
                    ui.add_space(20.0);

                ui.horizontal(|ui| {

                    let grid_width = 200.0 * 2.0 + 20.0; // 2 Buttons + Spacing
                    let app_center_width = (ui.available_width() - (grid_width)) / 2.0; // find middle of the app window
                    
                    ui.add_space(app_center_width.max(0.0)); // move the grid from the left to the middle


                    egui::Grid::new("menu_grid")
                        .spacing([20.0, 20.0])
                        .show(ui, |ui| {

                            let text_chess = // Chess button
                                RichText::new("♛ Schach ♚").size(30.0).color(Color32::WHITE).strong();
                            let button_chess = egui::Button::new(text_chess)
                                .min_size(Vec2::new(200.0, 150.0))
                                .rounding(10.0)
                                .fill(Color32::LIGHT_BLUE);
                            let response = ui.add(button_chess);
                            if response.clicked() {
                                self.state = AppState::Playing(Box::new(ChessGame::new()));
                            }

                            let text_go = RichText::new("☯ Go ☯").size(30.0).color(Color32::WHITE).strong(); // Go button
                            let button_go = egui::Button::new(text_go)
                                .min_size(Vec2::new(200.0, 150.0))
                                .rounding(10.0)
                                .fill(Color32::DARK_BLUE);
                            let response = ui.add(button_go);
                            if response.clicked() {
                                self.state = AppState::Playing(Box::new(GoGame::new()));
                            }

                            ui.end_row();

                            let text_kniffel = // Kniffel Button
                                RichText::new("🎲 Kniffel 🎲").size(30.0).color(Color32::WHITE).strong();
                            let button_kniffel = egui::Button::new(text_kniffel)
                                .min_size(Vec2::new(200.0, 150.0))
                                .rounding(10.0)
                                .fill(Color32::DARK_BLUE);
                            let response = ui.add(button_kniffel);
                            if response.clicked() {
                                self.state = AppState::Playing(Box::new(KniffelGame::new()));
                            }

                            let text_minesweeper = RichText::new("💣 Minesweeper 🚩") // Minesweeper button
                                .size(20.0)
                                .color(Color32::WHITE)
                                .strong();
                            let button_minesweeper = egui::Button::new(text_minesweeper)
                                .min_size(Vec2::new(200.0, 150.0))
                                .rounding(10.0)
                                .fill(Color32::LIGHT_BLUE);
                            let response = ui.add(button_minesweeper);
                            if response.clicked() {
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

    let icon_data = include_bytes!("app_icon/image.png");
    let icon_image = image::load_from_memory(icon_data)
        .expect("Failed to load icon")
        .to_rgba8();
    let (icon_width, icon_height) = icon_image.dimensions();
    
    let icon = egui::IconData {
        rgba: icon_image.into_raw(),
        width: icon_width,
        height: icon_height,
    };
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0]) // Set the initial window size of the Platform Window
            .with_min_inner_size([600.0, 500.0]) // Set the minimum window size of the Platform Window
            .with_icon(icon), // Set the app icon
        ..Default::default()
    };
    eframe::run_native(
        "Spielesammlung",
        native_options,
        Box::new(|cc| Ok(Box::new(PlatformApp::new(cc)))),
    )
}
