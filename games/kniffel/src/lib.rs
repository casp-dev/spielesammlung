mod kniffel;

use crate::kniffel::*;
use game_core::Game;
use kniffel::{next_player, throw_dice, YahtzeeGame};

enum Screen {
    Setup,
    InGame,
}

pub struct KniffelGame {
    screen: Screen,
    players: usize,
    player_buttons: Vec<Vec<Option<u32>>>,
    bots: usize,
    game: kniffel::Game,
}

impl Default for KniffelGame {
    fn default() -> Self {
        Self::new()
    }
}

impl KniffelGame {
    pub fn new() -> Self {
        Self {
            screen: Screen::Setup,
            players: 2,
            player_buttons: vec![vec![None; 13]; 2],
            bots: 0,
            game: <kniffel::Game as YahtzeeGame>::new(2).unwrap(),
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
        }
    }
}

impl KniffelGame {
    fn ui_setup(&mut self, ui: &mut egui::Ui) {
        ui.heading("Neues Kniffelspiel erstellen");
        ui.separator();

        for number_of_bots in 0..=3 {
            if self.bots == number_of_bots {
                ui.add(
                    egui::Slider::new(&mut self.players, 1..=4 - number_of_bots).text("Spieler"),
                );
            }
        }

        ui.separator();

        if self.players == 1 {
            ui.add(
                egui::Slider::new(&mut self.bots, 1..=3) //TODO: wenn player 1 dann muss bot auf mindestens 1 locken (0 unerreichbar), bei 4 spielern automatisch auf 0
                    .text("Anzahl Computergegner"),
            );
        } else {
            ui.add(
                egui::Slider::new(&mut self.bots, 0..=3) //TODO: wenn player 1 dann muss bot auf mindestens 1 locken (0 unerreichbar), bei 4 spielern automatisch auf 0
                    .text("Anzahl Computergegner"),
            );
        }

        ui.separator();

        if ui.button("Spiel starten").clicked() {
            self.game = <kniffel::Game as YahtzeeGame>::new(self.players + self.bots).unwrap();
            self.player_buttons = vec![vec![None; 13]; self.players + self.bots];
            self.screen = Screen::InGame;
        }
    }

    fn ui_game(&mut self, ui: &mut egui::Ui) {
        egui::SidePanel::left("kniffel_side_panel").show_inside(ui, |ui| {
            ui.heading("Punktetabelle");
            ui.separator();

            let active_players = self.players + self.bots;


            egui::ScrollArea::vertical().auto_shrink([false; 2])
                                        .show(ui, |ui| {egui::Grid::new("point_table")
                                        .spacing([20.0, 10.0])
                                        .show(ui, |ui| {
                    ui.label("Kategorie");

                    for i in 0..active_players {
                        ui.label(format!("Spieler {}", i + 1));
                    }

                    ui.end_row();

                    let upper_categories = [
                        (0_usize, "Einser", "Addiert die Augenzahl aller Würfel mit Zahl 1"),
                        (1, "Zweier", "Addiert die Augenzahl aller Würfel mit Zahl 2"),
                        (2, "Dreier", "Addiert die Augenzahl aller Würfel mit Zahl 3"),
                        (3, "Vierer", "Addiert die Augenzahl aller Würfel mit Zahl 4"),
                        (4, "Fünfer", "Addiert die Augenzahl aller Würfel mit Zahl 5"),
                        (5, "Sechser", "Addiert die Augenzahl aller Würfel mit Zahl 6"),
                    ];

                    for (category, label, hover) in upper_categories {
                        ui.label(label).on_hover_text(hover);
                        self.render_point_cells(ui, category);
                        ui.end_row();
                    }

                    ui.separator();
                    ui.end_row();

                    ui.label("Oben gesamt");
                    self.render_totals(ui, 0); //OBEN GESAMT FIX
                    ui.end_row();

                    ui.label("Oben mit Bonus").on_hover_text("Wenn mindestens 63 Punkte oben erreicht wurden, werden 35 Bonuspunkte addiert");
                    self.render_totals(ui, 1); //FIX
                    ui.end_row();

                    ui.separator();
                    ui.end_row();

                    let lower_categories = [
                        (6_usize, "Dreierpasch", "Mindestens drei gleiche Zahlen, die Augen aller Würfel werden addiert"),
                        (7, "Viererpasch", "Mindestens vier gleiche Zahlen, die Augen aller Würfel werden addiert"),
                        (8, "Full House", "25Pkt: Drei gleiche und zwei gleiche andere Zahlen"),
                        (9, "Kleine Straße", "30Pkt: Folge von vier aufeinander folgenden Würfeln"),
                        (10, "Große Straße", "40Pkt: Folge von fünf aufeinander folgenden Würfeln"),
                        (11, "Kniffel", "50Pkt: 5 gleiche Zahlen"),
                        (12, "Chance", "Summiert alle Augenzahlen"),
                    ];

                    for (category, label, hover) in lower_categories {
                        ui.label(label).on_hover_text(hover);
                        self.render_point_cells(ui, category);
                        ui.end_row();
                    }

                    self.render_static_cells(ui, "-");
                    ui.end_row();

                    ui.label("Punkte oben");
                    self.render_totals(ui, 1); //FIX
                    ui.end_row();

                    ui.label("Punkte unten");
                    self.render_totals(ui, 2); //FIX
                    ui.end_row();

                    ui.separator();
                    ui.end_row();

                    ui.label("Punkte gesamt");
                    self.render_totals(ui, 3); //FIX
                });

        });
    });

        ui.vertical_centered(|ui| {
            display_current_player(ui, &self.game.clone());

            ui.separator();

            ui.label(format!(
                "Wurf {}/3",
                self.game.current_player.number_of_throws
            ));

            ui.separator();

            if ui.button("Würfeln").clicked() && self.game.current_player.number_of_throws < 3 {
                throw_dice(&mut self.game);
            }

            ui.separator();

            if self.game.current_player.number_of_throws > 0 {
                egui::Grid::new("dice")
                    .spacing([50.0, 10.0])
                    .show(ui, |ui| {
                        self.render_dice(ui);
                    });
            }
        });

        ui.separator();

        //Spielfeld quasi wo die Würfel sich bewegen bla bla

        let outer_rect = ui.available_rect_before_wrap();
        let center = outer_rect.center();
        let size = egui::vec2(400.0, 300.0);

        let centered_rect = egui::Rect::from_center_size(center, size);

        ui.painter()
            .rect_filled(centered_rect, 5.0, egui::Color32::DARK_GREEN);

        //sichtbar sobald Punktetabelle voll ist
        
        if point_table_full(&self.game) {
            self.display_winner(ui);
        }
    }
}

impl KniffelGame {
    fn render_point_cells(&mut self, ui: &mut egui::Ui, category: usize) {
        let active_players = self.players + self.bots;

        for player_index in 0..active_players {
            let current_value = self
                .player_buttons
                .get(player_index)
                .and_then(|row| row.get(category))
                .copied()
                .flatten();

            let is_current_player = player_index == self.game.current_player_index;
            let is_filled = current_value.is_some();
            let category_label = if is_filled {
                format!("{} Pkt", current_value.unwrap())
            } else {
                "0".to_string()
            };

            let response = ui.add_enabled(
                is_current_player && !is_filled && (self.game.current_player.number_of_throws > 0),
                egui::Button::new(category_label),
            ); //fehlt: dass Würfel gewürfelt werden, ansonsten blocked bis 1. Wurf FIX

            if response.clicked() {
                let points = calculate_points(category, self.game.current_player.dice_throw);
                self.game = add_dice_point_table(&mut self.game, category).clone();

                if let Some(row) = self.player_buttons.get_mut(player_index) {
                    if let Some(slot) = row.get_mut(category) {
                        *slot = Some(points as u32);
                    }
                }

                if let Some(player) = self
                    .game
                    .all_players
                    .get_mut(self.game.current_player_index)
                {
                    player.point_table.points_thrown[category] = Some(points);
                }

                next_player(&mut self.game);
            }
        }
    }

    fn render_static_cells(&self, ui: &mut egui::Ui, label: &str) {
        let active_players = self.players + self.bots;

        for _ in 0..active_players {
            ui.label(label);
        }
    }

    fn render_totals(&self, ui: &mut egui::Ui, number: usize) {
        let active_players = self.players + self.bots;

        for player_index in 0..active_players {
            ui.label(format!(
                "{} Pkt",
                self.game.all_players[player_index].point_table.total_points[number]
            ));
        }
    }

    fn render_dice(&mut self, ui: &mut egui::Ui) {
        for dice_index in 0..5 {
            let dice = self.game.current_player.dice_throw[dice_index].eyes;
            let dice_button = {
                if self.game.current_player.dice_throw[dice_index].is_locked {
                    ui.button(format!("{} gesperrt", dice))
                } else {
                    ui.button(format!("{}", dice))
                }
            };

            if dice_button.clicked() {
                self.game = change_blocked_status_dice(&mut self.game, dice_index).clone();
            }
        }
    }

    fn display_winner(&mut self, ui:&mut egui::Ui) {
        let winner_tuple = self.game.winner().unwrap();
        if winner_tuple.0.len() == 1 {
            ui.label(format!("Der Gewinner mit {} Punkten ist Spieler {}", winner_tuple.1, (winner_tuple.0[0])+1));
        }
        else {
            ui.label(format!("Gleichstand zwischen Spielern {:?}, mit {} Punkten", winner_tuple.0, winner_tuple.1));
        }
    }
}

fn display_current_player(ui: &mut egui::Ui, game: &kniffel::Game) {
    let player_num = game.current_player_index + 1;
    ui.label(format!("Spieler {} ist dran", player_num));
}
