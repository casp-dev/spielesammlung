use crate::minesweeper::{
    ActionKind, CellContent, CellState, Difficulty, Game as MSGame, Minesweeper,
};
use egui::{Color32, RichText, Ui, Vec2};
use game_core::Game;

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

pub struct MinesweeperGame {
    state: GameState,
    current_difficulty: Option<Difficulty>,
}

impl MinesweeperGame {
    pub fn new() -> Self {
        Self {
            state: GameState::ChoosingDifficulty,
            current_difficulty: None,
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

                let height = game.board.len();
                let width = game.board[0].len();

                let mut re_choose_difficulty = false;
                let mut game_ended = false;

                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("🔙 Zurück").clicked() {
                            // Outsource because game state can not be changed here
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
                                    // unopened Cell

                                    // FOR TESTING: Uncomment the next 6 lines and comment out the line after to show mines
                                    // let text = if game.board[y][x].cell_content == CellContent::Mine {
                                    //     "💣"
                                    // } else {
                                    //     ""
                                    // };
                                    // let button = egui::Button::new(text).min_size(Vec2::new(25.0, 25.0));

                                    let button = egui::Button::new("").min_size(Vec2::new(25.0, 25.0));
                                    
                                    let click_or_flag = ui.add(button);

                                    if click_or_flag.clicked() {
                                        MSGame::apply_action(game, ActionKind::Open(x, y));
                                        // println!("Opened cell {}:{}", x, y);
                                    }
                                    if click_or_flag.secondary_clicked() {
                                        MSGame::apply_action(game, ActionKind::Flag(x, y));
                                        // println!("Flaged cell {}:{}", x, y);
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
                });

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

                ui.vertical_centered(|ui| { // copy of the board in the backround
                    
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
                                    let button =
                                        egui::Button::new("🚩").min_size(Vec2::new(25.0, 25.0));
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

                            let text_easy = // Show baord to inspect after game
                                RichText::new("Spielfeld anzeigen").size(20.0).color(Color32::WHITE);
                            let button_easy = egui::Button::new(text_easy)
                                .min_size(Vec2::new(200.0, 50.0))
                                .fill(Color32::from_rgb(0, 100, 0));
                            if ui.add(button_easy).clicked() {
                                close_popup = true;
                            }

                            ui.add_space(10.0);

                            let text_medium = // Re-choose difficulty
                                RichText::new("Andere Schwierigkeit").size(20.0).color(Color32::WHITE);
                            let button_medium = egui::Button::new(text_medium)
                                .min_size(Vec2::new(200.0, 50.0))
                                .fill(Color32::from_rgb(255, 153, 0));
                            if ui.add(button_medium).clicked() {
                                re_choose_difficulty = true;
                            }

                            ui.add_space(10.0);

                            let text_hard = // Try Again
                                RichText::new("Nochmal versuchen").size(20.0).color(Color32::WHITE);
                            let button_hard = egui::Button::new(text_hard)
                                .min_size(Vec2::new(200.0, 50.0))
                                .fill(Color32::from_rgb(100, 0, 0));
                            if ui.add(button_hard).clicked() {
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
