use crate::minesweeper::{
    ActionKind, CellContent, CellState, Difficulty, Game as MSGame, Minesweeper,
};

use egui::{Color32, RichText, Ui, Vec2};
use game_core::Game;

// Debug flag: Set to true to show mine locations on unopened cells
const DEBUG_SHOW_MINES: bool = false;

pub enum GameState {
    ChoosingDifficulty,
    Playing(MSGame),
    WinnerOrLoserPopup {
        won: bool,
        picked_difficulty: Difficulty,
        game: MSGame,
        show_popup: bool,
    },
}

pub fn color_for_mines_nearby(i: u8) -> Color32 {
    match i {
        1 => Color32::BLUE,
        2 => Color32::DARK_GREEN,
        3 => Color32::RED,
        _ => Color32::BLACK,
    }
}

/// Returns a list of all neighboring cells for a given position
pub fn neighbors(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut neighbor_coords = Vec::new();

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = x as isize + dx;
            let ny = y as isize + dy;

            if nx >= 0 && ny >= 0 {
                let nx = nx as usize;
                let ny = ny as usize;

                if ny < height && nx < width {
                    neighbor_coords.push((ny, nx));
                }
            }
        }
    }
    neighbor_coords
}

pub struct MinesweeperGame {
    state: GameState,
    current_difficulty: Option<Difficulty>,
    saved_highlights: Vec<(usize, usize)>, // List of cells to highlight in the next render (neighbors of pressed number cell)
    last_opened_cell: Option<(usize, usize)>,
}

impl MinesweeperGame {
    pub fn new() -> Self {
        Self {
            state: GameState::ChoosingDifficulty,
            current_difficulty: None,
            saved_highlights: Vec::new(),
            last_opened_cell: None,
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
                    ui.heading("Wähle eine Schwierigkeit:");

                    ui.add_space(20.0);

                    let text_easy = RichText::new("Einfach").size(20.0).color(Color32::WHITE); // Easy
                    let button_easy = egui::Button::new(text_easy)
                        .min_size(Vec2::new(120.0, 50.0))
                        .fill(Color32::from_rgb(0, 100, 0));
                    if ui.add(button_easy).clicked() {
                        self.current_difficulty = Some(Difficulty::Easy);
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Easy));
                    }

                    ui.add_space(10.0);

                    let text_medium = RichText::new("Mittel").size(20.0).color(Color32::WHITE); // Medium
                    let button_medium = egui::Button::new(text_medium)
                        .min_size(Vec2::new(120.0, 50.0))
                        .fill(Color32::from_rgb(255, 153, 0));
                    if ui.add(button_medium).clicked() {
                        self.current_difficulty = Some(Difficulty::Medium);
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Medium));
                    }

                    ui.add_space(10.0);

                    let text_hard = RichText::new("Schwer").size(20.0).color(Color32::WHITE); // Hard
                    let button_hard = egui::Button::new(text_hard)
                        .min_size(Vec2::new(120.0, 50.0))
                        .fill(Color32::from_rgb(100, 0, 0));
                    if ui.add(button_hard).clicked() {
                        self.current_difficulty = Some(Difficulty::Hard);
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Hard));
                    }

                    ui.add_space(10.0);

                    let text_expert = RichText::new("Experte").size(20.0).color(Color32::WHITE); // Expert
                    let button_expert = egui::Button::new(text_expert)
                        .min_size(Vec2::new(120.0, 50.0))
                        .fill(Color32::from_rgb(80, 0, 80));
                    if ui.add(button_expert).clicked() {
                        self.current_difficulty = Some(Difficulty::Expert);
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Expert));
                    }
                });
            }

            GameState::Playing(game) => {

                let mut might_be_over: Option<(usize, usize)> = None;

                let height = game.board.len();
                let width = game.board[0].len();

                let mut re_choose_difficulty = false;
                let mut game_ended = false;

                let mut neighbors_to_highlight: Option<Vec<(usize, usize)>> = None; // Store neighbors to highlight in next frame
                let mut number_cell_pressed = false; // Track if any number cell is currently being pressed

                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("🔙 Zurück").clicked() {
                            re_choose_difficulty = true; // Outsource because game state can not be changed here
                        }

                        ui.separator();
                        ui.label(RichText::new(format!("🚩: {}", game.flag_count)).size(15.0));
                        ui.separator();
                        ui.label(RichText::new(format!("💣: {}", game.mine_count)).size(15.0));
                    });

                    ui.add_space(10.0);

                    for y in 0..height {
                        ui.horizontal(|ui| {
                            for x in 0..width {

                                let is_highlighted = self.saved_highlights.contains(&(y, x));

                                if (game.board[y][x].cell_state == CellState::Unopened) {
                                    // Unopened Cell
                                    let text = if DEBUG_SHOW_MINES && game.board[y][x].cell_content == CellContent::Mine {
                                        "💣"
                                    } else {
                                        ""
                                    };

                                    let button = if is_highlighted {
                                        egui::Button::new(text)
                                            .min_size(Vec2::new(25.0, 25.0))
                                            .fill(Color32::GRAY)
                                    } else {
                                        egui::Button::new(text).min_size(Vec2::new(25.0, 25.0))
                                    };

                                    let click_or_flag = ui.add(button);
                                    
                                    if (click_or_flag.clicked()) {
                                        might_be_over = Some((y ,x)); // Store as (y, x) to match board indexing [y][x] and popup comparison
                                        MSGame::apply_action(game, ActionKind::Open(x, y));
                                        // println!("Opened cell {}:{}", x, y);
                                    }
                                    if (click_or_flag.secondary_clicked()) {
                                        MSGame::apply_action(game, ActionKind::Flag(x, y));
                                        // println!("Flagged cell {}:{}", x, y);
                                    }
                                }

                                if (game.board[y][x].cell_state == CellState::Opened) {
                                    // Opened Cell
                                    match game.board[y][x].cell_content {
                                        CellContent::Blank => {
                                            let fill_color = Color32::DARK_GRAY;
                                            let button = egui::Button::new("")
                                                .fill(fill_color)
                                                .min_size(Vec2::new(25.0, 25.0));
                                            ui.add(button);
                                        }
                                        CellContent::Mine => {
                                            let fill_color = Color32::BLACK;
                                            let text = RichText::new("💣").color(Color32::WHITE);
                                            let button = egui::Button::new(text)
                                                .fill(fill_color)
                                                .min_size(Vec2::new(25.0, 25.0));
                                            ui.add(button);
                                        }
                                        CellContent::Number(i) => {
                                            let text = RichText::new(i.to_string())
                                                .color(Color32::WHITE)
                                                .size(20.0);
                                            let button = egui::Button::new(text)
                                                .fill(color_for_mines_nearby(i))
                                                .min_size(Vec2::new(25.0, 25.0));

                                            let show_neighbors = ui.add(button);

                                            if show_neighbors.is_pointer_button_down_on() {
                                                neighbors_to_highlight =
                                                    Some(neighbors(x, y, width, height));
                                                number_cell_pressed = true;
                                            }
                                        }
                                    }
                                }

                                if (game.board[y][x].cell_state == CellState::Flagged) {
                                    // Flagged Cell
                                    let text = RichText::new("🚩").color(Color32::RED);
                                    let button = if is_highlighted {
                                        egui::Button::new(text)
                                            .min_size(Vec2::new(25.0, 25.0))
                                            .fill(Color32::GRAY)
                                    } else {
                                        egui::Button::new(text).min_size(Vec2::new(25.0, 25.0))
                                    };
                                    let unflag = ui.add(button);

                                    if unflag.secondary_clicked() {
                                        MSGame::apply_action(game, ActionKind::Flag(x, y));
                                        // println!("Unflagged cell {}:{}", x, y);
                                    }
                                }
                            }
                        });
                    }
                });

                self.last_opened_cell = might_be_over;

                if let Some(new_highlights) = neighbors_to_highlight {
                    // Apply neighbor highlighting for next frame
                    self.saved_highlights = new_highlights;
                } else if !number_cell_pressed {
                    // Clear highlighting when no button is pressed
                    self.saved_highlights.clear();
                }

                if (game.game_over || game.game_won) {
                    game_ended = true;
                }
                if re_choose_difficulty {
                    self.state = GameState::ChoosingDifficulty;
                } else if game_ended {
                    self.state = GameState::WinnerOrLoserPopup {
                        won: game.game_won,
                        picked_difficulty: self.current_difficulty.unwrap_or(Difficulty::Easy),
                        game: game.clone(),
                        show_popup: true,
                    }
                }
            }

            GameState::WinnerOrLoserPopup {
                won,
                picked_difficulty,
                game,
                show_popup,
            } => {
                let height = game.board.len();
                let width = game.board[0].len();

                let get_picked_difficulty = *picked_difficulty;
                let mut close_popup = false;
                let mut re_choose_difficulty = false;
                let mut retry = false;

                ui.vertical_centered(|ui| {
                    // copy of the board in the background
                    ui.horizontal(|ui| {
                        if ui.button("🔙 Zurück").clicked() {
                            re_choose_difficulty = true;
                        }

                        ui.separator();
                        ui.label(RichText::new(format!("🚩: {}", game.flag_count)).size(15.0));
                        ui.separator();
                        ui.label(RichText::new(format!("💣: {}", game.mine_count)).size(15.0));
                    });

                    ui.add_space(10.0);

                    for y in 0..height {
                        ui.horizontal(|ui| {
                            for x in 0..width {
                                if (game.board[y][x].cell_state == CellState::Unopened) {
                                    let button =
                                        egui::Button::new("").min_size(Vec2::new(25.0, 25.0));
                                    let _click_or_flag = ui.add(button);
                                }

                                if (game.board[y][x].cell_state == CellState::Opened) {

                                    match game.board[y][x].cell_content {

                                        CellContent::Blank => {
                                            let mut button = egui::Button::new("")
                                                .fill(Color32::DARK_GRAY)
                                                .min_size(Vec2::new(25.0, 25.0));

                                            if (self.last_opened_cell == Some((y, x))) {
                                                button = button.stroke(egui::Stroke::new(3.0, Color32::GOLD));
                                            }
                                            
                                            ui.add(button);
                                        }

                                        CellContent::Mine => {
                                            let text = RichText::new("💣").color(Color32::WHITE);
                                            let mut button = egui::Button::new(text)
                                                .fill(Color32::BLACK)
                                                .min_size(Vec2::new(25.0, 25.0));
                                            
                                            if (self.last_opened_cell == Some((y, x))) {
                                                button = button.stroke(egui::Stroke::new(3.0, Color32::RED));
                                            }
                                            
                                            ui.add(button);
                                        }

                                        CellContent::Number(i) => {
                                            let fill_color = color_for_mines_nearby(i);
                                            let text = RichText::new(i.to_string())
                                                .color(Color32::WHITE)
                                                .size(20.0);
                                            let mut button = egui::Button::new(text)
                                                .fill(fill_color)
                                                .min_size(Vec2::new(25.0, 25.0));
                                            
                                            if (self.last_opened_cell == Some((y, x))) {
                                                button = button.stroke(egui::Stroke::new(3.0, Color32::GOLD));
                                            }
                                            
                                            ui.add(button);
                                        }
                                    }
                                }

                                if (game.board[y][x].cell_state == CellState::Flagged) {
                                    let text = RichText::new("🚩").color(Color32::RED);
                                    let button =
                                        egui::Button::new(text).min_size(Vec2::new(25.0, 25.0));
                                    let _unflag = ui.add(button);
                                }
                            }
                        });
                    }
                });

                let window_text: &str;

                if (*won) {
                    window_text = "🎉 YOU WON! 🎉";
                } else {
                    window_text = "💣 YOU LOST! 💣";
                }
                if (*show_popup) {
                    egui::Window::new(window_text) // Popup to display options after a won/ lost game
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ui.ctx(), |ui| {

                        ui.vertical_centered(|ui| {

                            let text_show_board = // Show board to inspect after game
                                RichText::new("Spielfeld anzeigen").size(20.0).color(Color32::WHITE);
                            let button_show_board = egui::Button::new(text_show_board)
                                .min_size(Vec2::new(200.0, 50.0))
                                .fill(Color32::from_rgb(0, 100, 0));
                            if ui.add(button_show_board).clicked() {
                                close_popup = true;
                            }

                            ui.add_space(10.0);

                            let text_rechoose_difficulty = // Re-choose difficulty
                                RichText::new("Andere Schwierigkeit").size(20.0).color(Color32::WHITE);
                            let button_rechoose_difficulty = egui::Button::new(text_rechoose_difficulty)
                                .min_size(Vec2::new(200.0, 50.0))
                                .fill(Color32::from_rgb(255, 153, 0));
                            if ui.add(button_rechoose_difficulty).clicked() {
                                re_choose_difficulty = true;
                            }

                            ui.add_space(10.0);

                            let text_try_again = // Try Again
                                RichText::new("Nochmal versuchen").size(20.0).color(Color32::WHITE);
                            let button_try_again = egui::Button::new(text_try_again)
                                .min_size(Vec2::new(200.0, 50.0))
                                .fill(Color32::from_rgb(100, 0, 0));
                            if ui.add(button_try_again).clicked() {
                                retry = true;
                        }
                    });
                });
                }

                if re_choose_difficulty {
                    self.state = GameState::ChoosingDifficulty;
                } else if retry {
                    self.state = GameState::Playing(MSGame::new_game(get_picked_difficulty));
                } else if close_popup {
                    self.state = GameState::WinnerOrLoserPopup {
                        won: *won,
                        picked_difficulty: get_picked_difficulty,
                        game: game.clone(),
                        show_popup: false,
                    };
                }
            }
        }
    }
}
