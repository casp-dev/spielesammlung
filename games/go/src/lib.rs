use egui::Ui;
use game_core::Game;

pub struct GoGame {
    state: String,
}

impl GoGame {
    pub fn new() -> Self {
        Self {
            state: "Initial Go State".to_string(),
        }
    }
}

impl Game for GoGame {
    fn name(&self) -> &str {
        "Go"
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Go");
        ui.label(format!("Current State: {}", self.state));

        if ui.button("Make Move (e4)").clicked() {
            self.state = "Go move: e4".to_string();
        }
    }
}
