use game_core::Game;

use crate::{ChessGame, meeples::{Color, Meeple, Type, get_meeple_at}};

pub fn draw_board(ui: &mut egui::Ui, game: &mut ChessGame) {
    egui::Grid::new("chess_board")
        .spacing(egui::vec2(0.0, 0.0))
        .show(ui, |ui| {
            for y in 0..8 {
                for x in 0..8 {
                    let meeple = get_meeple_at(game.game_board, (x,y));

                    //string for the clickable
                    let pos_text = if let Some(m) = meeple {
                        get_piece_char(m)
                    } else {
                        " "
                    };

                    let bg_color = if (x + y) % 2 != 0 {
                        egui::Color32::from_rgb(240, 217, 181)
                    } else {
                        egui::Color32::from_rgb(181, 136, 99)
                    };

                    let mut button_color = bg_color;
                    if let Some(possible_moves) = &game.shown_moves {
                        if possible_moves.contains(&(x,y)) {
                            button_color = egui::Color32::from_rgb(100, 255, 100);
                        }
                    }

                    //clickable
                    let btn = egui::Button::new(
                        egui::RichText::new(pos_text)
                            .size(32.0)
                            .color(if let Some(m) = meeple{
                                if m.color == Color::Black { egui::Color32::BLACK } else { egui::Color32::WHITE }
                            } else {
                                egui::Color32::BLACK
                            })
                    )
                    .fill(button_color)
                    .min_size(egui::vec2(60.0, 60.0));

                    if ui.add(btn).clicked() {
                        handle_click(game, (x,y), true);
                    }
                }
                ui.end_row();
            }
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

fn handle_click(game: &mut ChessGame, (x_pos,y_pos): (usize,usize),highlighted: bool) {
    println!("{},{}", x_pos,y_pos)
}