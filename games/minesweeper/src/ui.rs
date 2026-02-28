use crate::minesweeper::{
    ActionKind, CellContent, CellState, Difficulty, Game as MSGame, Minesweeper,
};

use egui::{Color32, RichText, Ui, Vec2};
use game_core::Game;

const DEBUG_SHOW_MINES: bool = false; // Debug flag: Set to true to show mine locations on unopened cells

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

#[allow(unused_parens)]
pub fn neighbors(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    // Returns a list of all neighboring cells for a given position
    let mut neighbor_coords = Vec::new();

    for dy in -1..=1 {
        for dx in -1..=1 {
            if (dx == 0 && dy == 0) {
                continue;
            }

            let nx = x as isize + dx;
            let ny = y as isize + dy;

            if (nx >= 0 && ny >= 0) {
                let nx = nx as usize;
                let ny = ny as usize;

                if (ny < height && nx < width) {
                    neighbor_coords.push((ny, nx));
                }
            }
        }
    }
    return neighbor_coords;
}
pub fn button_colors(difficulty_text: &str) -> Color32 {
    match difficulty_text {
        "Einfach" => Color32::from_rgb(0, 100, 0),
        "Mittel" => Color32::from_rgb(255, 153, 0),
        "Schwer" => Color32::from_rgb(100, 0, 0),
        "Experte" => Color32::from_rgb(80, 0, 80),
        "Spielfeld anzeigen" => Color32::from_rgb(0, 100, 0),
        "Andere Schwierigkeit" => Color32::from_rgb(255, 153, 0),
        "Nochmal versuchen" => Color32::from_rgb(100, 0, 0),
        _ => Color32::BLACK,
    }
}

pub fn add_button(ui: &mut egui::Ui, text: &str, width: f32, height: f32) -> egui::Response {
    let button_text = RichText::new(text).size(20.0).color(Color32::WHITE);
    let button_easy = egui::Button::new(button_text)
        .min_size(Vec2::new(width, height))
        .fill(button_colors(text));

    return ui.add(button_easy);
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
impl Game for MinesweeperGame {
    fn name(&self) -> &str {
        return "Minesweeper";
    }

    fn ui(&mut self, ui: &mut Ui) {
        match &mut self.state {
            GameState::ChoosingDifficulty => {
                let available_width = ui.available_width();
                let available_height = ui.available_height();

                let button_width = (available_width * 0.3).clamp(120.0, 400.0);
                let button_height = (available_height * 0.08).clamp(50.0, 100.0);
                let heading_height = 30.0;
                let spacing = 10.0;

                let total_content_height =
                    (heading_height + spacing * 2.0) + (button_height * 4.0) + (spacing * 3.0);
                let vertical_spacing = ((available_height - total_content_height) / 2.25).max(20.0); // Center vertically with minimum 20 Pixels top spacing

                ui.add_space(vertical_spacing);

                ui.vertical_centered(|ui| {
                    ui.heading("Wähle eine Schwierigkeit:");

                    ui.add_space(spacing * 2.0);

                    if (add_button(ui, "Einfach", button_width, button_height).clicked() == true) {
                        self.current_difficulty = Some(Difficulty::Easy);
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Easy));
                    }

                    ui.add_space(spacing);

                    if (add_button(ui, "Mittel", button_width, button_height).clicked() == true) {
                        self.current_difficulty = Some(Difficulty::Medium);
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Medium));
                    }

                    ui.add_space(spacing);

                    if (add_button(ui, "Schwer", button_width, button_height).clicked() == true) {
                        self.current_difficulty = Some(Difficulty::Hard);
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Hard));
                    }

                    ui.add_space(spacing);

                    if (add_button(ui, "Experte", button_width, button_height).clicked() == true) {
                        self.current_difficulty = Some(Difficulty::Expert);
                        self.state = GameState::Playing(MSGame::new_game(Difficulty::Expert));
                    }
                });
            }

            GameState::Playing(game) => {
                let mut latest_clicked_cell: Option<(usize, usize)> = None;

                let height = game.board.len();
                let width = game.board[0].len();

                let mut re_choose_difficulty = false;
                let mut game_ended = false;

                let mut neighbors_to_highlight: Option<Vec<(usize, usize)>> = None; // Store neighbors to highlight in next frame
                let mut number_cell_pressed = false; // Track if any number cell is currently being pressed

                ui.horizontal(|ui| {
                    if (ui.button("🔙 Zurück").clicked() == true) {
                        re_choose_difficulty = true;
                    }

                    ui.separator();
                    ui.label(RichText::new(format!("🚩: {}", game.flag_count)).size(15.0));
                    ui.separator();
                    ui.label(RichText::new(format!("💣: {}", game.mine_count)).size(15.0));
                });

                let available_width = ui.available_width();
                let available_height = ui.available_height();

                let cell_size = 25.0;

                let spacing_x = ui.spacing().item_spacing.x;
                let spacing_y = ui.spacing().item_spacing.y;

                let board_width =
                    (width as f32 * cell_size) + ((width as f32 - 1.0).max(0.0) * spacing_x);
                let board_height =
                    (height as f32 * cell_size) + ((height as f32 - 1.0).max(0.0) * spacing_y);

                let vertical_offset = if (board_height < available_height) {
                    (available_height - board_height) / 2.0
                } else {
                    10.0
                };
                let horizontal_offset = if (board_width < available_width) {
                    (available_width - board_width) / 2.0
                } else {
                    0.0
                };

                ui.add_space(vertical_offset);

                for y in 0..height {
                    ui.horizontal(|ui| {
                        ui.add_space(horizontal_offset);

                        for x in 0..width {
                            let is_highlighted = self.saved_highlights.contains(&(y, x));

                            if (game.board[y][x].cell_state == CellState::Unopened) {
                                let text = if (DEBUG_SHOW_MINES == true
                                    && game.board[y][x].cell_content == CellContent::Mine)
                                {
                                    "💣"
                                } else {
                                    ""
                                };

                                let button = if (is_highlighted == true) {
                                    egui::Button::new(text)
                                        .min_size(Vec2::new(cell_size, cell_size))
                                        .fill(Color32::GRAY)
                                } else {
                                    egui::Button::new(text)
                                        .min_size(Vec2::new(cell_size, cell_size))
                                };

                                let click_or_flag = ui.add(button);

                                if click_or_flag.clicked() {
                                    latest_clicked_cell = Some((y, x)); // Store as (y, x) to match board indexing [y][x] and popup comparison
                                    let _ = MSGame::apply_action(game, ActionKind::Open(x, y));
                                }
                                if click_or_flag.secondary_clicked() {
                                    let _ = MSGame::apply_action(game, ActionKind::Flag(x, y));
                                }
                            }

                            if (game.board[y][x].cell_state == CellState::Opened) {
                                match game.board[y][x].cell_content {
                                    CellContent::Blank => {
                                        let fill_color = Color32::DARK_GRAY;
                                        let button = egui::Button::new("")
                                            .fill(fill_color)
                                            .min_size(Vec2::new(cell_size, cell_size));
                                        ui.add(button);
                                    }
                                    CellContent::Mine => {
                                        let fill_color = Color32::BLACK;
                                        let text = RichText::new("💣").color(Color32::WHITE);
                                        let button = egui::Button::new(text)
                                            .fill(fill_color)
                                            .min_size(Vec2::new(cell_size, cell_size));
                                        ui.add(button);
                                    }
                                    CellContent::Number(i) => {
                                        let text = RichText::new(i.to_string())
                                            .color(Color32::WHITE)
                                            .size(20.0);
                                        let button = egui::Button::new(text)
                                            .fill(color_for_mines_nearby(i))
                                            .min_size(Vec2::new(cell_size, cell_size));

                                        let show_neighbors = ui.add(button);

                                        if (show_neighbors.is_pointer_button_down_on() == true) {
                                            neighbors_to_highlight =
                                                Some(neighbors(x, y, width, height));
                                            number_cell_pressed = true;
                                        }
                                    }
                                }
                            }

                            if (game.board[y][x].cell_state == CellState::Flagged) {
                                let text = RichText::new("🚩").color(Color32::RED);
                                let button = if (is_highlighted == true) {
                                    egui::Button::new(text)
                                        .min_size(Vec2::new(cell_size, cell_size))
                                        .fill(Color32::GRAY)
                                } else {
                                    egui::Button::new(text)
                                        .min_size(Vec2::new(cell_size, cell_size))
                                };

                                let unflag = ui.add(button);

                                if unflag.secondary_clicked() {
                                    let _ = MSGame::apply_action(game, ActionKind::Flag(x, y));
                                }
                            }
                        }
                    });
                }

                self.last_opened_cell = latest_clicked_cell;

                if let Some(new_highlights) = neighbors_to_highlight {
                    // Apply neighbor highlighting for next frame
                    self.saved_highlights = new_highlights;
                } else if (number_cell_pressed == false) {
                    self.saved_highlights.clear();
                }

                if (game.game_over == true || game.game_won == true) {
                    game_ended = true;
                }

                if (re_choose_difficulty == true) {
                    self.state = GameState::ChoosingDifficulty;
                } else if (game_ended == true) {
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

                ui.horizontal(|ui| {
                    if (ui.button("🔙 Zurück").clicked() == true) {
                        re_choose_difficulty = true;
                    }

                    ui.separator();
                    ui.label(RichText::new(format!("🚩: {}", game.flag_count)).size(15.0));
                    ui.separator();
                    ui.label(RichText::new(format!("💣: {}", game.mine_count)).size(15.0));
                });

                let available_width = ui.available_width();
                let available_height = ui.available_height();

                let cell_size = 25.0;

                let spacing_x = ui.spacing().item_spacing.x;
                let spacing_y = ui.spacing().item_spacing.y;

                let board_width =
                    (width as f32 * cell_size) + ((width as f32 - 1.0).max(0.0) * spacing_x);
                let board_height =
                    (height as f32 * cell_size) + ((height as f32 - 1.0).max(0.0) * spacing_y);

                let vertical_offset = if (board_height < available_height) {
                    (available_height - board_height) / 2.0
                } else {
                    10.0
                };
                let horizontal_offset = if (board_width < available_width) {
                    (available_width - board_width) / 2.0
                } else {
                    0.0
                };

                ui.add_space(vertical_offset);

                for y in 0..height {
                    ui.horizontal(|ui| {
                        ui.add_space(horizontal_offset);

                        for x in 0..width {
                            if (game.board[y][x].cell_state == CellState::Unopened) {
                                let button =
                                    egui::Button::new("").min_size(Vec2::new(cell_size, cell_size));
                                let _click_or_flag = ui.add(button);
                            }

                            if (game.board[y][x].cell_state == CellState::Opened) {
                                match game.board[y][x].cell_content {
                                    CellContent::Blank => {
                                        let mut button = egui::Button::new("")
                                            .fill(Color32::DARK_GRAY)
                                            .min_size(Vec2::new(cell_size, cell_size));

                                        if (self.last_opened_cell == Some((y, x))) {
                                            button = button
                                                .stroke(egui::Stroke::new(3.0, Color32::GOLD));
                                        }

                                        ui.add(button);
                                    }

                                    CellContent::Mine => {
                                        let text = RichText::new("💣").color(Color32::WHITE);
                                        let mut button = egui::Button::new(text)
                                            .fill(Color32::BLACK)
                                            .min_size(Vec2::new(cell_size, cell_size));

                                        if (self.last_opened_cell == Some((y, x))) {
                                            button =
                                                button.stroke(egui::Stroke::new(3.0, Color32::RED));
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
                                            .min_size(Vec2::new(cell_size, cell_size));

                                        if (self.last_opened_cell == Some((y, x))) {
                                            button = button
                                                .stroke(egui::Stroke::new(3.0, Color32::GOLD));
                                        }

                                        ui.add(button);
                                    }
                                }
                            }

                            if (game.board[y][x].cell_state == CellState::Flagged) {
                                let text = RichText::new("🚩").color(Color32::RED);
                                let button = egui::Button::new(text)
                                    .min_size(Vec2::new(cell_size, cell_size));
                                let _unflag = ui.add(button);
                            }
                        }
                    });
                }

                let window_text = if (*won == true) {
                    "🎉 YOU WON! 🎉"
                } else {
                    "💣 YOU LOST! 💣"
                };
                if (*show_popup == true) {
                    let popup_button_width = 200.0;
                    let popup_button_height = 50.0;
                    let spacing = 10.0;

                    egui::Window::new(window_text)
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ui.ctx(), |ui| {
                            ui.vertical_centered(|ui| {
                                if (add_button(
                                    ui,
                                    "Spielfeld anzeigen",
                                    popup_button_width,
                                    popup_button_height,
                                )
                                .clicked()
                                    == true)
                                {
                                    close_popup = true;
                                }

                                ui.add_space(spacing);

                                if (add_button(
                                    ui,
                                    "Andere Schwierigkeit",
                                    popup_button_width,
                                    popup_button_height,
                                )
                                .clicked()
                                    == true)
                                {
                                    re_choose_difficulty = true;
                                }

                                ui.add_space(spacing);

                                if (add_button(
                                    ui,
                                    "Nochmal versuchen",
                                    popup_button_width,
                                    popup_button_height,
                                )
                                .clicked()
                                    == true)
                                {
                                    retry = true;
                                }
                            });
                        });
                }

                if (re_choose_difficulty == true) {
                    self.state = GameState::ChoosingDifficulty;
                } else if (retry == true) {
                    self.state = GameState::Playing(MSGame::new_game(get_picked_difficulty));
                } else if (close_popup == true) {
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
