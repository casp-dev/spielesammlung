mod ai;
mod game;

use ai::get_best_move;
use game::{Game, Stone};
use game_core::Game as CoreGame;

#[allow(dead_code)]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 750.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Rust Go",
        options,
        Box::new(|_cc| Ok(Box::new(GoGame::default()))),
    )
}

pub struct GoGame {
    game: Game,
    status_message: String,
    ai_stats_message: String,
    ai_enabled: bool,
}

impl Default for GoGame {
    fn default() -> Self {
        Self {
            game: Game::new(19), // Standard 19x19 brett
            status_message: "Spiel gestartet. Schwarz ist am Zug.".to_owned(),
            ai_stats_message: String::new(),
            ai_enabled: false,
        }
    }
}

impl GoGame {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CoreGame for GoGame {
    fn name(&self) -> &str {
        "Go"
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Rust Go");

        ui.horizontal(|ui| {
            ui.label(format!("Zug: {:?}", self.game.current_turn));
            ui.label(format!("Schwarz gefangen: {}", self.game.captured_black));
            ui.label(format!("Weiß gefangen: {}", self.game.captured_white));
        });

        if ui.button("Passen").clicked() {
            self.game.pass();
            if self.game.game_over {
                let (b_score, w_score) = self.game.calculate_score();
                self.status_message = format!(
                    "Spiel vorbei! Punkte: Schwarz {:.1}, Weiß {:.1}",
                    b_score, w_score
                );
            } else {
                self.status_message = format!("Gepasst. {:?} ist am Zug.", self.game.current_turn);
            }
        }

        ui.checkbox(&mut self.ai_enabled, "AI Gegner");

        if ui.button("Spiel neustarten").clicked() {
            self.game = Game::new(19);
            self.status_message = "Spiel neugestartet. Schwarz ist am Zug.".to_owned();
            self.ai_stats_message.clear();
        }

        ui.label(&self.status_message);
        if !self.ai_stats_message.is_empty() {
            ui.label(&self.ai_stats_message);
        }

        if self.game.game_over {
            let (b_score, w_score) = self.game.calculate_score();
            ui.label(format!(
                "Endstand:\nSchwarz: {:.1}\nWeiß: {:.1}",
                b_score, w_score
            ));
            if b_score > w_score {
                ui.label("Schwarz gewinnt!");
            } else {
                ui.label("Weiß gewinnt!");
            }
        }

        let available_size = ui.available_size();
        let board_size = available_size.x.min(available_size.y) - 20.0;
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(board_size, board_size),
            egui::Sense::click(),
        );

        let rect = response.rect;
        let grid_size = self.game.board.size;

        // farbe
        painter.rect_filled(
            rect,
            egui::Rounding::same(20.0),
            egui::Color32::from_rgb(222, 184, 135),
        );

        let padding = board_size * 0.05;
        let grid_rect = rect.shrink(padding);
        let cell_size = grid_rect.width() / (grid_size as f32 - 1.0);

        // Raster
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50));
        for i in 0..grid_size {
            let pos = i as f32 * cell_size;

            // verticale Linien
            painter.line_segment(
                [
                    grid_rect.min + egui::vec2(pos, 0.0),
                    grid_rect.min + egui::vec2(pos, grid_rect.height()),
                ],
                stroke,
            );

            // Horizontale Linien
            painter.line_segment(
                [
                    grid_rect.min + egui::vec2(0.0, pos),
                    grid_rect.min + egui::vec2(grid_rect.width(), pos),
                ],
                stroke,
            );
        }

        // hoshi punkte
        if grid_size == 19 {
            let stars = [3, 9, 15];
            for &y in &stars {
                for &x in &stars {
                    let center =
                        grid_rect.min + egui::vec2(x as f32 * cell_size, y as f32 * cell_size);
                    painter.circle_filled(center, cell_size * 0.15, egui::Color32::BLACK);
                }
            }
        }

        // Steine
        for y in 0..grid_size {
            for x in 0..grid_size {
                if let Some(stone) = self.game.board.get(x, y) {
                    let center =
                        grid_rect.min + egui::vec2(x as f32 * cell_size, y as f32 * cell_size);
                    let stone_radius = cell_size * 0.45;

                    // Schatten
                    painter.circle_filled(
                        center + egui::vec2(2.0, 2.0),
                        stone_radius,
                        egui::Color32::from_black_alpha(50),
                    );

                    match stone {
                        Stone::Black => {
                            painter.circle_filled(center, stone_radius, egui::Color32::BLACK);
                            // Glanz
                            painter.circle_filled(
                                center - egui::vec2(stone_radius * 0.3, stone_radius * 0.3),
                                stone_radius * 0.2,
                                egui::Color32::from_white_alpha(30),
                            );
                        }
                        Stone::White => {
                            painter.circle_filled(center, stone_radius, egui::Color32::WHITE);
                            painter.circle_stroke(
                                center,
                                stone_radius,
                                egui::Stroke::new(1.0, egui::Color32::GRAY),
                            );
                        }
                    };
                }
            }
        }

        // Hover
        if let Some(pos) = response.hover_pos() {
            if !self.game.game_over && rect.contains(pos) {
                let relative_pos = pos - grid_rect.min;
                let x_f = relative_pos.x / cell_size;
                let y_f = relative_pos.y / cell_size;

                let x = x_f.round() as i32;
                let y = y_f.round() as i32;

                if x >= 0 && x < grid_size as i32 && y >= 0 && y < grid_size as i32 {
                    if self.game.board.get(x as usize, y as usize).is_none() {
                        let center =
                            grid_rect.min + egui::vec2(x as f32 * cell_size, y as f32 * cell_size);
                        let color = match self.game.current_turn {
                            Stone::Black => egui::Color32::from_black_alpha(100),
                            Stone::White => egui::Color32::from_white_alpha(100),
                        };
                        painter.circle_filled(center, cell_size * 0.4, color);
                    }
                }
            }
        }

        // Klicks
        if response.clicked() && !self.game.game_over {
            if let Some(pos) = response.interact_pointer_pos() {
                // pos zu Brett-Koordinaten umrechnen
                let relative_pos = pos - grid_rect.min;
                let x_f = relative_pos.x / cell_size;
                let y_f = relative_pos.y / cell_size;

                let x = x_f.round() as i32;
                let y = y_f.round() as i32;

                if x >= 0 && x < grid_size as i32 && y >= 0 && y < grid_size as i32 {
                    match self.game.place_stone(x as usize, y as usize) {
                        Ok(_) => {
                            self.status_message =
                                format!("Zug akzeptiert. {:?} ist am Zug.", self.game.current_turn);

                            // AI auto-play
                            if self.ai_enabled && !self.game.game_over {
                                let (best_move, stats) = get_best_move(&self.game, 1000);
                                if let Some((x, y)) = best_move {
                                    match self.game.place_stone(x, y) {
                                        Ok(_) => {
                                            self.status_message = format!(
                                                "AI spielt ({}, {}). {:?} ist am Zug.",
                                                x, y, self.game.current_turn
                                            );
                                            let top_moves_str: String = stats
                                                .top_moves
                                                .iter()
                                                .take(3)
                                                .map(|(m, v, s)| {
                                                    format!("({},{}):{}/{:.2}", m.0, m.1, v, s)
                                                })
                                                .collect::<Vec<_>>()
                                                .join(", ");
                                            self.ai_stats_message = format!(
                                                "MCTS: {} Iterationen, Top: {}",
                                                stats.iterations, top_moves_str
                                            );
                                        }
                                        Err(e) => {
                                            self.status_message = format!("AI Zug ungültig: {}", e);
                                        }
                                    }
                                } else {
                                    self.game.pass();
                                    self.status_message = "AI passt.".to_owned();
                                }
                            }
                        }
                        Err(e) => {
                            self.status_message = format!("Ungültiger Zug: {}", e);
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for GoGame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }
}
