use egui::Ui;
use game_core::Game;

pub struct ChessGame {
    state: String,
}

impl ChessGame {
    pub fn new() -> Self {
        Self {
            state: "Initial Chess State".to_string(),
        }
    }
}

impl Game for ChessGame {
    fn name(&self) -> &str {
        "Chess"
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Chess");
        ui.label(format!("Current State: {}", self.state));

        if ui.button("Make Move (e4)").clicked() {
            self.state = "Chess move: e4".to_string();
        }
    }
}
