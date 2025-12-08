use egui::Ui;
use game_core::Game;

pub struct KniffelGame {
    state: String,
}

impl KniffelGame {
    pub fn new() -> Self {
        Self {
            state: "Initial Kniffel State".to_string(),
        }
    }
}

impl Game for KniffelGame {
    fn name(&self) -> &str {
        "Kniffel"
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Kniffel");
        ui.label(format!("Current State: {}", self.state));

        if ui.button("Make Move (e4)").clicked() {
            self.state = "Kniffel move: e4".to_string();
        }
    }
}
