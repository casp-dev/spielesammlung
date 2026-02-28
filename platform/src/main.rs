use chess::ChessGame;
use eframe::egui;
use game_core::Game;
use go::GoGame;
use kniffel::KniffelGame;
use minesweeper::MinesweeperGame;

use egui::{Image, ImageSource, Vec2};

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
    chess_image: ImageSource<'static>,
    go_image: ImageSource<'static>,
    kniffel_image: ImageSource<'static>,
    minesweeper_image: ImageSource<'static>,
}

impl PlatformApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self {
            state: AppState::Menu,
            theme: AppTheme::Dark, // Default theme is dark
            chess_image: egui::include_image!("images/chess.png"),
            go_image: egui::include_image!("images/GO.png"),
            kniffel_image: egui::include_image!("images/Kniffel.png"),
            minesweeper_image: egui::include_image!("images/Minesweeper.png"),
        }
    }
}

pub fn game_button(
    ui: &mut egui::Ui,
    game_image: impl Into<ImageSource<'static>>,
    height: f32,
    width: f32,
) -> egui::Response {
    let image = Image::new(game_image)
        .fit_to_exact_size(Vec2::new(width, height))
        .rounding(10.0);

    let button = egui::ImageButton::new(image);

    let response = ui
        .scope(|ui| {
            ui.spacing_mut().button_padding = Vec2::ZERO;
            ui.add(button)
        })
        .inner;

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    response
}

impl eframe::App for PlatformApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // better name: platform_ui
        egui::CentralPanel::default().show(ctx, |ui| match &mut self.state {
            AppState::Menu => {
                match self.theme {
                    AppTheme::Light => {
                        ctx.set_visuals(egui::Visuals::light()); // for the window content itself
                        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(
                            egui::SystemTheme::Light,
                        )); // for the os window frame
                    }

                    AppTheme::Dark => {
                        ctx.set_visuals(egui::Visuals::dark());
                        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(
                            egui::SystemTheme::Dark,
                        ));
                    }
                }

                let available_width = ui.available_width();
                let available_height = ui.available_height();

                let button_width = (available_width * 0.3).clamp(85.0, 400.0);
                let button_height = button_width;

                let heading_height = 10.0; // assumed height of "Wähle ein Spiel:"
                let spacing = 20.0;
                let buffer = 50.0;

                let grid_height =
                    (button_height * 2.0) + spacing + heading_height + spacing + buffer;
                let app_center_height = (available_height - grid_height) / 2.0;

                let grid_width = (button_width * 2.0) + spacing;
                let app_center_width = (available_width - grid_width) / 2.0;

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .selectable_label(self.theme == AppTheme::Light, "Light")
                            .clicked()
                        {
                            self.theme = AppTheme::Light;
                        }

                        if ui
                            .selectable_label(self.theme == AppTheme::Dark, "Dark")
                            .clicked()
                        {
                            self.theme = AppTheme::Dark;
                        }
                    });
                });

                ui.separator();

                ui.vertical_centered(|ui| {
                    ui.add_space(app_center_height); // move the grid form the top to the middle

                    ui.heading("Wähle ein Spiel:");
                    ui.add_space(spacing);

                    ui.horizontal(|ui| {
                        ui.add_space(app_center_width); // move the grid from the left to the middle

                        egui::Grid::new("menu_grid")
                            .spacing([spacing, spacing])
                            .show(ui, |ui| {
                                if game_button(
                                    ui,
                                    self.chess_image.clone(),
                                    button_height,
                                    button_width,
                                )
                                .clicked()
                                {
                                    self.state = AppState::Playing(Box::new(ChessGame::new()));
                                }

                                if game_button(
                                    ui,
                                    self.go_image.clone(),
                                    button_height,
                                    button_width,
                                )
                                .clicked()
                                {
                                    self.state = AppState::Playing(Box::new(GoGame::new()));
                                }

                                ui.end_row();

                                if game_button(
                                    ui,
                                    self.kniffel_image.clone(),
                                    button_height,
                                    button_width,
                                )
                                .clicked()
                                {
                                    self.state = AppState::Playing(Box::new(KniffelGame::new()));
                                }

                                if game_button(
                                    ui,
                                    self.minesweeper_image.clone(),
                                    button_height,
                                    button_width,
                                )
                                .clicked()
                                {
                                    self.state =
                                        AppState::Playing(Box::new(MinesweeperGame::new()));
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
    let icon_data = include_bytes!("images/app_icon.png");
    let icon_image = image::load_from_memory(icon_data)
        .expect("Failed to load app icon")
        .to_rgba8();
    let icon_width = icon_image.width();
    let icon_height = icon_image.height();

    let icon = egui::IconData {
        rgba: icon_image.into_raw(),
        width: icon_width,
        height: icon_height,
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([750.0, 695.0]) // Set the initial window size of the Platform Window
            .with_min_inner_size([750.0, 695.0]) // Set the minimum window size of the Platform Window
            .with_icon(icon), // Set the app icon
        ..Default::default()
    };

    eframe::run_native(
        "Spielesammlung",
        native_options,
        Box::new(|cc| Ok(Box::new(PlatformApp::new(cc)))),
    )
}
