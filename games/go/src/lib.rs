mod game;

use egui::Ui;
use game_core::Game;

pub struct GoGame {
    game: game::Game,
}

impl GoGame {
    pub fn new() -> Self {
        Self {
            game: game::Game::new(19),
        }
    }
}

impl Game for GoGame {
    fn name(&self) -> &str {
        "Go"
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Go");
        ui.label(format!("Aktueller Zug: {:?}", self.game.current_turn));

        if ui.button("Pass").clicked() {
            //todo
        }
    }
}
