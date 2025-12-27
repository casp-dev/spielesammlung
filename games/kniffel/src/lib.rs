mod kniffel;

use egui::Ui;
use game_core::Game;
use kniffel::{YahtzeeGame, throw_dice};

enum Screen {
    Setup,
    InGame,
}

pub struct KniffelGame {
    state: String,
    screen: Screen,
    players: usize,
    bots: usize,
    game: Option<kniffel::Game>,
}

impl KniffelGame {
    pub fn new() -> Self {
        Self {
            state: "Initial Kniffel State".to_string(),
            screen: Screen::Setup,
            players: 2,
            bots: 0,
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
            egui::Slider::new(&mut self.players, 1..=4)
                .text("Spieler"),
        );

        ui.separator();

        ui.add(
            egui::Slider::new(&mut self.bots, 0..=3) //TODO: wenn player 1 dann muss bot auf mindestens 1 locken (0 unerreichbar), bei 4 spielern automatisch auf 0
                .text("Anzahl Computergegner"),
        );

        ui.separator();


        if ui.button("Spiel starten").clicked() {
            match <kniffel::Game as YahtzeeGame>::new(self.players + self.bots as usize) {
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
        egui::SidePanel::left("kniffel_side_panel").show_inside(ui, |ui| {
            ui.heading("Punktetabelle");
           
            let active_players = self.players + self.bots;

            egui::Grid::new("point_table")
                .spacing([20.0, 10.0])
                .show(ui, |ui| {
                ui.label("Kategorie");

                for i in 0..active_players {
                    ui.label(format!("Spieler {}", i + 1));
                }

                ui.end_row();

                ui.label("Einser").on_hover_text("Addiert die Augenzahl aller Würfel mit Zahl 1");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.label("Zweier").on_hover_text("Addiert die Augenzahl aller Würfel mit Zahl 2");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.label("Dreier").on_hover_text("Addiert die Augenzahl aller Würfel mit Zahl 3");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.label("Vierer").on_hover_text("Addiert die Augenzahl aller Würfel mit Zahl 4");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.label("Fünfer").on_hover_text("Addiert die Augenzahl aller Würfel mit Zahl 5");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.label("Sechser").on_hover_text("Addiert die Augenzahl aller Würfel mit Zahl 6");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.separator();
                ui.end_row();

                ui.label("Oben gesamt");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.label("Oben mit Bonus").on_hover_text("Wenn mindestens 63 Punkte oben erreicht wurden, werden 35 Bonuspunkte addiert");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.separator();
                ui.end_row();

                ui.label("Dreierpasch").on_hover_text("Mindestens drei gleiche Zahlen, die Augen aller Würfel werden addiert");
                fill_up_point_table(ui, active_players);                
                ui.end_row();

                ui.label("Viererpasch").on_hover_text("Mindestens vier gleiche Zahlen, die Augen aller Würfel werden addiert");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.label("Full House").on_hover_text("25Pkt: Drei gleiche und zwei gleiche andere Zahlen");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.label("Kleine Straße").on_hover_text("30Pkt: Folge von vier aufeinander folgenden Würfeln");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.label("Große Straße").on_hover_text("40Pkt: Folge von fünf aufeinander folgenden Würfeln");
                fill_up_point_table(ui, active_players);
                ui.end_row();
                
                ui.label("Kniffel").on_hover_text("50Pkt: 5 gleiche Zahlen");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.label("Chance").on_hover_text("Summiert alle Augenzahlen");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.separator();
                ui.end_row();

                ui.label("Punkte oben");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.label("Punkte unten");
                fill_up_point_table(ui, active_players);
                ui.end_row();

                ui.separator();
                ui.end_row();

                ui.label("Punkte gesamt");
                fill_up_point_table(ui, active_players);
            });

        });

        //während Punkttabelle nocht nicht voll ist das folgende sichtbar

        //if !point_table_full() {
        if true {
        ui.heading("Kniffel Hauptbereich");
        ui.label("Spieler x ist dran");
        ui.label("Wurf x/3"); 
        if ui.button("Würfeln").clicked() {
            if let Some(ref mut game) = self.game {
                throw_dice(game);
            }
        }

        //Spielfeld quasi wo die Würfel sich bewegen bla bla

        ui.label("Zur Seite gelegte Würfel:");
        //Kasten mit den Würfeln
        }

        //sichtbar sobald Punktetabelle voll ist
        
        //if point_table_full() {
        //    ui.label("Der Gewinner mit x Punkten ist Spieler x");
        //}

    }

}

fn fill_up_point_table(ui: &mut egui::Ui, players: usize) {
    for _ in 0..players {
        ui.button("Punktzahl");
    }
}

fn point_table_full() -> bool {
    unimplemented!();
}
