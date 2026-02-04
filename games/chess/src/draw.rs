use crate::{
    engine::Engine,
    meeples::{get_meeple_at, Color, Meeple, Type},
    ChessGame,
};

pub fn draw_start_window(ui: &mut egui::Ui, game: &mut ChessGame) {
    let screen_size = ui.available_size();
    let button_width = screen_size.x * 0.4;
    let button_height = screen_size.y * 0.2;
    let local_btn =
        egui::Button::new("Play local ♚ v ♔").min_size(egui::vec2(button_width, button_height));
    let bot_btn =
        egui::Button::new("Play vs Bot ♚ v ♔").min_size(egui::vec2(button_width, button_height));
    let multiplayer_btn = egui::Button::new("Multiplayer (soon) ♚ v ♔")
        .min_size(egui::vec2(button_width, button_height));
    if ui.add(local_btn).clicked() {
        game.state = "0.0".to_string();
    }
    ui.add(egui::Slider::new(&mut game.possible_bot_level, 1..=7).text("level"));
    if ui.add(bot_btn).clicked() {
        game.state = "0.0".to_string();
        let search_depth = match game.possible_bot_level {
            1..=2 => 2,
            3..=4 => 3,
            5..=6 => 4,
            _ => 5,
        };
        game.engine = Some(Engine::new(search_depth, Color::Black));
    }
    if ui.add(multiplayer_btn).clicked() {
        //todo multiplayer
    }
}
pub fn draw_board(ui: &mut egui::Ui, game: &mut ChessGame) {
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
                    if let Some(possible_moves) = &game.shown_moves {
                        if possible_moves.contains(&(x, y)) {
                            button_color = egui::Color32::from_rgb(100, 255, 100);
                        }
                    }

                    //clickable
                    let btn = egui::Button::new(egui::RichText::new(pos_text).size(32.0).color(
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
