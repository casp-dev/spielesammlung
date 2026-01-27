mod ai;
mod game;

use ai::get_best_move;
use game::{Game, Stone};
use game_core::Game as CoreGame;

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
}

impl Default for GoGame {
    fn default() -> Self {
        Self {
            game: Game::new(19), // Standard 19x19 brett
            status_message: "Spiel gestartet. Schwarz ist am Zug.".to_owned(),
            ai_stats_message: String::new(),
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

        if ui.button("AI Zug").clicked() && !self.game.game_over {
            let (best_move, stats) = get_best_move(&self.game, 1000);
            if let Some((x, y)) = best_move {
                match self.game.place_stone(x, y) {
                    Ok(_) => {
                        self.status_message = format!(
                            "AI spielt ({}, {}). {:?} ist am Zug.",
                            x, y, self.game.current_turn
                        );
                        // show top moves from MCTS
                        let top_moves_str: String = stats
                            .top_moves
                            .iter()
                            .take(3)
                            .map(|(m, v, s)| format!("({},{}):{}/{:.2}", m.0, m.1, v, s))
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
        let cell_size = rect.width() / (grid_size as f32 + 1.0); // margin

        // Raster
        let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
        for i in 0..grid_size {
            let pos = i as f32 * cell_size + cell_size;

            // verticale Linien
            painter.line_segment(
                [
                    rect.min + egui::vec2(pos, cell_size),
                    rect.min + egui::vec2(pos, rect.height() - cell_size),
                ],
                stroke,
            );

            // Horizontale Linien
            painter.line_segment(
                [
                    rect.min + egui::vec2(cell_size, pos),
                    rect.min + egui::vec2(rect.width() - cell_size, pos),
                ],
                stroke,
            );
        }

        // Steine
        for y in 0..grid_size {
            for x in 0..grid_size {
                if let Some(stone) = self.game.board.get(x, y) {
                    let center = rect.min
                        + egui::vec2(
                            x as f32 * cell_size + cell_size,
                            y as f32 * cell_size + cell_size,
                        );
                    let color = match stone {
                        Stone::Black => egui::Color32::BLACK,
                        Stone::White => egui::Color32::WHITE,
                    };
                    let stroke_color = match stone {
                        Stone::Black => egui::Color32::WHITE,
                        Stone::White => egui::Color32::BLACK,
                    };

                    painter.circle_filled(center, cell_size * 0.45, color);
                    painter.circle_stroke(
                        center,
                        cell_size * 0.45,
                        egui::Stroke::new(1.0, stroke_color),
                    );
                }
            }
        }

        // Hover
        if let Some(pos) = response.hover_pos() {
            if !self.game.game_over {
                let relative_pos = pos - rect.min;
                let x_f = (relative_pos.x / cell_size) - 1.0;
                let y_f = (relative_pos.y / cell_size) - 1.0;

                let x = x_f.round() as i32;
                let y = y_f.round() as i32;

                if x >= 0 && x < grid_size as i32 && y >= 0 && y < grid_size as i32 {
                    if self.game.board.get(x as usize, y as usize).is_none() {
                        let center = rect.min
                            + egui::vec2(
                                x as f32 * cell_size + cell_size,
                                y as f32 * cell_size + cell_size,
                            );
                        let color = match self.game.current_turn {
                            Stone::Black => egui::Color32::BLACK.linear_multiply(0.3),
                            Stone::White => egui::Color32::WHITE.linear_multiply(0.35),
                        };
                        let stroke_color = match self.game.current_turn {
                            Stone::Black => egui::Color32::WHITE.linear_multiply(0.35),
                            Stone::White => egui::Color32::BLACK.linear_multiply(0.35),
                        };
                        painter.circle_filled(center, cell_size * 0.45, color);
                        painter.circle_stroke(
                            center,
                            cell_size * 0.45,
                            egui::Stroke::new(1.0, stroke_color),
                        );
                    }
                }
            }
        }

        // Klicks
        if response.clicked() && !self.game.game_over {
            if let Some(pos) = response.interact_pointer_pos() {
                // pos zu Brett-Koordinaten umrechnen
                let relative_pos = pos - rect.min;
                let x_f = (relative_pos.x / cell_size) - 1.0;
                let y_f = (relative_pos.y / cell_size) - 1.0;

                let x = x_f.round() as i32;
                let y = y_f.round() as i32;

                if x >= 0 && x < grid_size as i32 && y >= 0 && y < grid_size as i32 {
                    match self.game.place_stone(x as usize, y as usize) {
                        Ok(_) => {
                            self.status_message =
                                format!("Zug akzeptiert. {:?} ist am Zug.", self.game.current_turn);
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
