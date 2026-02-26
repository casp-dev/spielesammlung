use crate::{
    meeples::{get_meeple_at, Color, Meeple, Type},
    ChessGame,
};

pub fn draw_board(ui: &mut egui::Ui, game: &mut ChessGame) {
    let last_move = game.logs.last().unwrap().clone();

    let board_width = 8.0 * 60.0;
    let board_height = board_width;

    let available_width = ui.available_width();
    let available_height = ui.available_height();

    let left_offset = (available_width - board_width) / 2.0;
    let top_offset = (available_height - board_height) / 2.0;

    ui.horizontal(|ui| {
        if left_offset > 0.0 {
            ui.add_space(left_offset);
        }

        ui.vertical(|ui| {
            if top_offset > 0.0 {
                ui.add_space(top_offset);
            }

            egui::Grid::new("chess_board")
                .spacing(egui::vec2(0.0, 0.0))
                .show(ui, |ui| {
                    for y in 0..8 {
                        for x in 0..8 {
                            let meeple = get_meeple_at(&game.game_board, (x, y));

                            //string for the clickable
                            let pos_text = if let Some(m) = meeple {
                                get_piece_char(m)
                            } else {
                                " "
                            };

                            //background color
                            let is_lighted = (x + y) % 2 != 0;
                            let bg_color = if is_lighted {
                                egui::Color32::from_rgb(240, 217, 181)
                            } else {
                                egui::Color32::from_rgb(181, 136, 99)
                            };

                            //highlight color
                            let mut button_color = bg_color;
                            if (x, y) == last_move.0 || (x, y) == last_move.1 {
                                button_color = egui::Color32::from_rgb(246, 246, 105);
                            }
                            if let Some(possible_moves) = &game.shown_moves {
                                if possible_moves.contains(&(x, y)) {
                                    button_color = egui::Color32::from_rgb(100, 255, 100);
                                }
                            }

                            //clickable
                            let btn =
                                egui::Button::new(egui::RichText::new(pos_text).size(32.0).color(
                                    if let Some(m) = meeple {
                                        if m.color == Color::Black {
                                            egui::Color32::BLACK
                                        } else {
                                            egui::Color32::WHITE
                                        }
                                    } else {
                                        egui::Color32::BLACK
                                    },
                                ))
                                .fill(button_color)
                                .min_size(egui::vec2(60.0, 60.0));

                            if ui.add(btn).clicked() {
                                handle_click(
                                    game,
                                    (x, y),
                                    button_color == egui::Color32::from_rgb(100, 255, 100),
                                );
                            }
                        }
                        ui.end_row();
                    }
                });
            if !game.pawn_mutate {
                return;
            }
            egui::Grid::new("mutate pawn")
                .spacing(egui::vec2(0.0, 0.0))
                .show(ui, |ui| {
                    let draw_meeples = match game.turn {
                        Color::Black => ["♘", "♗", "♖", "♕"],
                        Color::White => ["♞", "♝", "♜", "♛"],
                    };
                    for btn_str in draw_meeples {
                        let btn = egui::Button::new(egui::RichText::new(btn_str).size(32.0).color(
                            if game.turn == Color::White {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::BLACK
                            },
                        ))
                        .fill(egui::Color32::GRAY)
                        .min_size(egui::vec2(60.0, 60.0));

                        if ui.add(btn).clicked() {
                            let mutate_into: Type;
                            match btn_str {
                                "♘" | "♞" => mutate_into = Type::Knight,
                                "♗" | "♝" => mutate_into = Type::Bishop,
                                "♖" | "♜" => mutate_into = Type::Rook,
                                "♕" | "♛" => mutate_into = Type::Queen,
                                _ => panic!("This should not happen"),
                            }
                            game.mutate_pawn(mutate_into);
                        }
                    }
                });
        });
    });
}

fn get_piece_char(meeple: Meeple) -> &'static str {
    match (meeple.color, meeple.typ) {
        (Color::White, Type::King) => "♔",
        (Color::White, Type::Queen) => "♕",
        (Color::White, Type::Rook) => "♖",
        (Color::White, Type::Bishop) => "♗",
        (Color::White, Type::Knight) => "♘",
        (Color::White, Type::Pawn) => "♙",
        (Color::Black, Type::King) => "♚",
        (Color::Black, Type::Queen) => "♛",
        (Color::Black, Type::Rook) => "♜",
        (Color::Black, Type::Bishop) => "♝",
        (Color::Black, Type::Knight) => "♞",
        (Color::Black, Type::Pawn) => "♟",
    }
}

fn handle_click(game: &mut ChessGame, pos: (usize, usize), highlighted: bool) {
    if game.pawn_mutate {
        return;
    }

    if highlighted {
        game.move_meeple(pos);
    } else {
        game.show_moves(pos);
    }
}
