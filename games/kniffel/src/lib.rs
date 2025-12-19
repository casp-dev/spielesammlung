mod kniffel;

use egui::Ui;
use game_core::Game;
use kniffel::YahtzeeGame;

enum Screen {
    Setup,
    InGame,
    GameOver,
}

pub struct KniffelGame {
    state: String,
    screen: Screen,
    players: usize,
    game: Option<kniffel::Game>,
}

impl KniffelGame {
    pub fn new() -> Self {
        Self {
            state: "Initial Kniffel State".to_string(),
            screen: Screen::Setup,
            players: 2,
            game: None,
        }
    }
}

impl Game for KniffelGame {
    fn name(&self) -> &str {
        "Kniffel"
    }
    
    fn ui(&mut self, ui: &mut egui::Ui) {
        match self.screen {
            Screen::Setup => self.ui_setup(ui),
            Screen::InGame => self.ui_game(ui),
            Screen::GameOver => self.ui_game_over(ui),
        }
    } 
}

impl KniffelGame {
    fn ui_setup(&mut self, ui: &mut egui::Ui) {
        ui.heading("Neues Kniffel-Spiel");

        ui.add(
            egui::Slider::new(&mut self.players, 2..=4)
                .text("Spieler"),
        );

        ui.separator();

        if ui.button("Spiel starten").clicked() {
            match <kniffel::Game as YahtzeeGame>::new(self.players as usize) {
                Ok(game) => {
                    self.game = Some(game);
                    self.screen = Screen::InGame;
                }
                Err(e) => {
                    ui.label(format!("Fehler: {}", e));
                }
            }
        }
    }

    fn ui_game(&mut self, ui: &mut egui::Ui) {
        unimplemented!();
    }

    fn ui_game_over(&mut self, ui: &mut egui::Ui) {
        unimplemented!();
    }
}
