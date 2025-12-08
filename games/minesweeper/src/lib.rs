use egui::Ui;
use game_core::Game;

pub struct MinesweeperGame {
    state: String,
}

impl MinesweeperGame {
    pub fn new() -> Self {
        Self {
            state: "Initial Minesweeper State".to_string(),
        }
    }
}

impl Game for MinesweeperGame {
    fn name(&self) -> &str {
        "Minesweeper"
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Minesweeper");
        ui.label(format!("Current State: {}", self.state));

        if ui.button("Make Move (e4)").clicked() {
            self.state = "Minesweeper move: e4".to_string();
        }
    }
}
