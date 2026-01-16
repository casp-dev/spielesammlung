use crate::minesweeper::{
    ActionKind, CellContent, CellState, Difficulty, Game as MSGame, Minesweeper,
};
use egui::{Color32, RichText, Ui, Vec2};
use game_core::Game;

pub enum GameState {
    ChoosingDifficulty,
    Playing(MSGame),
}

pub fn color_for_mines_nearby(i: u8) -> Color32 {
    match i {
        1 => Color32::BLUE,
        2 => Color32::DARK_GREEN,
        3 => Color32::RED,
        _ => Color32::BLACK,
    }
}

pub fn colors(n: u8) -> Color32 {
    // Not finished
    match n {
        _ => Color32::BLACK,
    }
}

pub struct MinesweeperGame {
    state: GameState,
}

impl MinesweeperGame {
    pub fn new() -> Self {
        Self {
            state: GameState::ChoosingDifficulty,
        }
    }
}

#[allow(unused_parens)]
#[allow(unused_must_use)]
impl Game for MinesweeperGame {
    fn name(&self) -> &str {
        "Minesweeper"
    }

    fn ui(&mut self, ui: &mut Ui) {
        match &mut self.state {
            GameState::ChoosingDifficulty => {
                ui.vertical_centered(|ui| {
                    ui.heading("Wähle eine Schwierigkeit");

                    ui.add_space(20.0);

                    let text_easy = RichText::new("Einfach").size(20.0).color(Color32::WHITE); // Easy
                    let button_easy = egui::Button::new(text_easy)
                        .min_size(Vec2::new(120.0, 50.0))
                        .fill(Color32::from_rgb(0, 100, 0));
                    if ui.add(button_easy).clicked() {
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Easy));
                    }

                    ui.add_space(10.0);

                    let text_medium = RichText::new("Mittel").size(20.0).color(Color32::WHITE); // Medium
                    let button_medium = egui::Button::new(text_medium)
                        .min_size(Vec2::new(120.0, 50.0))
                        .fill(Color32::from_rgb(255, 153, 0));
                    if ui.add(button_medium).clicked() {
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Medium));
                    }

                    ui.add_space(10.0);

                    let text_hard = RichText::new("Schwer").size(20.0).color(Color32::WHITE); // Hard
                    let button_hard = egui::Button::new(text_hard)
                        .min_size(Vec2::new(120.0, 50.0))
                        .fill(Color32::from_rgb(100, 0, 0));
                    if ui.add(button_hard).clicked() {
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Hard));
                    }

                    ui.add_space(10.0);

                    let text_expert = RichText::new("Experte").size(20.0).color(Color32::WHITE); // Expert
                    let button_expert = egui::Button::new(text_expert)
                        .min_size(Vec2::new(120.0, 50.0))
                        .fill(Color32::from_rgb(80, 0, 80));
                    if ui.add(button_expert).clicked() {
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Expert));
                    }
                });
            }

            GameState::Playing(game) => {

                let height = game.board.len();
                let width = game.board[0].len();

                ui.vertical_centered(|ui| {
                    for y in 0..height {
                        ui.horizontal(|ui| {
                            for x in 0..width {
                                if (game.board[y][x].cell_state == CellState::Unopened) {
                                    // unopened Cell
                                    let button =
                                        egui::Button::new("").min_size(Vec2::new(25.0, 25.0));
                                    let click_or_flag = ui.add(button);
                                    if click_or_flag.clicked() {
                                        MSGame::apply_action(game, ActionKind::Open(x, y));
                                        println!("Opened cell {}:{}", x, y);
                                    }
                                    if click_or_flag.secondary_clicked() {
                                        MSGame::apply_action(game, ActionKind::Flag(x, y));
                                        println!("Flaged cell {}:{}", x, y);
                                    }
                                }

                                if (game.board[y][x].cell_state == CellState::Opened) {
                                    // Opened Cell
                                    let (text, color) = match game.board[y][x].cell_content {
                                        CellContent::Blank => ("".to_string(), Color32::DARK_GRAY),
                                        CellContent::Mine => ("💣".to_string(), Color32::BLACK),
                                        CellContent::Number(i) => {
                                            (i.to_string(), color_for_mines_nearby(i))
                                        }
                                    };
                                    let button = egui::Button::new(text)
                                        .fill(color)
                                        .min_size(Vec2::new(25.0, 25.0));
                                    ui.add(button);
                                }

                                if (game.board[y][x].cell_state == CellState::Flagged) {
                                    // Flagged Cell
                                    let button =
                                        egui::Button::new("🚩").min_size(Vec2::new(25.0, 25.0));
                                    let unflag = ui.add(button);
                                    if unflag.secondary_clicked() {
                                        MSGame::apply_action(game, ActionKind::Flag(x, y));
                                        println!("Unflaged cell {}:{}", x, y);
                                    }
                                }
                            }
                        });
                    }

                    // NOT FINISHED 
                    ui.horizontal_centered(|ui| {
                        let flags_remaining_to_string =
                            format!("Flaggen übrig: {}", game.flag_count.to_string());
                        let text_flags_remaining = RichText::new(flags_remaining_to_string)
                            .size(20.0)
                            .color(Color32::WHITE);
                        let button_flags_remaining = egui::Button::new(text_flags_remaining)
                            .min_size(Vec2::new(120.0, 50.0))
                            .fill(Color32::from_rgb(80, 0, 80));
                        ui.add(button_flags_remaining);

                        let bombs_on_field_to_string =
                            format!("Minen auf dem Feld: {}", game.mine_count.to_string());
                        let text_bombs_on_field = RichText::new(bombs_on_field_to_string)
                            .size(20.0)
                            .color(Color32::WHITE);
                        let button_bombs_on_field = egui::Button::new(text_bombs_on_field)
                            .min_size(Vec2::new(120.0, 50.0))
                            .fill(Color32::from_rgb(80, 0, 80));
                        ui.add(button_bombs_on_field);
                    });
                });
            }
        }
    }
}
