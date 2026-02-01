use std::thread;

use crate::meeples::{opposite_color, Color, Meeple, Type};
#[derive(Clone, Copy)]
pub struct Engine {
    pub level: u16,
    pub color: Color,
}

impl Engine {
    pub fn new(level_: u16, color: Color) -> Engine {
        Engine {
            level: level_,
            color: color,
        }
    }

    pub fn move_move(
        &self,
        chess_board: &mut [[Option<Meeple>; 8]; 8],
        last_move: &((usize, usize), (usize, usize)),
        turn: Color,
    ) -> ((usize, usize), (usize, usize)) {
        let chess_board_t = chess_board.clone();
        let turn_t = turn;
        let level_t = self.level;
        let last_move_t = *last_move;
        let meeples = get_meeples_from_color(&chess_board_t, turn);

        let handle = thread::spawn(move || {
            get_best_move_black(
                chess_board_t,
                last_move_t,
                level_t,
                turn_t,
                f32::NEG_INFINITY,
                    f32::INFINITY,
                meeples.0.clone(),
                meeples.1.clone(),
            )
        });

        let ret = handle.join().unwrap();

        if let Some((from, to, _)) = ret {
            (from, to)
        } else {
            ((42, 42), (42, 42))
        }
    }
}

///this fct does a minmax to get the best move with alpha beta pruning
fn get_best_move_black(
    chess_board: [[Option<Meeple>; 8]; 8],
    last_move: ((usize, usize), (usize, usize)),
    level: u16,
    turn: Color,
    mut alpha: f32,
    mut beta: f32,
    turn_meeples: Vec<Meeple>,
    opposite_turn_meeples: Vec<Meeple>,
) -> Option<((usize, usize), (usize, usize), f32)> {
    //it is runcolores turn and incolor is the opponent
    let mut best_move: Option<((usize, usize), (usize, usize), f32)> = None;
    let is_maximizing = turn == Color::White;

    for colored_meeple in turn_meeples.iter() {
        for check_meep in
            colored_meeple.show_moves(&chess_board, &last_move, &opposite_turn_meeples)
        {
            let mut chess_board_clone = chess_board.clone();
            let from_pos = colored_meeple.pos;
            let to_pos = check_meep;

            //does the move, hier fehlt die rochade!!!
            let mut from = chess_board_clone[from_pos.0][from_pos.1].take().unwrap();
            from.pos = to_pos;

            chess_board_clone[to_pos.0][to_pos.1] = Some(from);

            //updates the meeple positions
            let mut turn_meeples_clone = turn_meeples.clone();
            if let Some(index) = turn_meeples_clone.iter().position(|x| x.pos == from_pos) {
                turn_meeples_clone[index].pos = to_pos;
            }

            let mut opposite_turn_meeples_clone = opposite_turn_meeples.clone();
            if let Some(index) = opposite_turn_meeples_clone
                .iter()
                .position(|x| x.pos == to_pos)
            {
                opposite_turn_meeples_clone.remove(index);
            }

            //checks if the move is legal (king is not in check)
            let mut legal_move = true;
            for check_meeple in opposite_turn_meeples_clone.iter() {
                if check_meeple
                    .show_moves(&chess_board_clone, &(from_pos, to_pos), &turn_meeples_clone)
                    .contains(&turn_meeples_clone.last().unwrap().pos)
                {
                    legal_move = false;
                    break;
                }
            }

            if !legal_move {
                continue;
            }

            let score = if level == 0 {
                quiescence(
                    chess_board_clone,
                    (from_pos, to_pos),
                    opposite_color(turn),
                    alpha,
                    beta,
                    opposite_turn_meeples_clone,
                    turn_meeples_clone,
                )
            } else {
                get_score_black(
                    chess_board_clone,
                    (from_pos, to_pos),
                    level - 1,
                    opposite_color(turn),
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                    opposite_turn_meeples_clone,
                    turn_meeples_clone,
                )
            };

            // Update best move and alpha-beta values
            if let Some((_, _, best_score)) = best_move {
                if is_maximizing {
                    if score > best_score {
                        best_move = Some((from_pos, to_pos, score));
                        alpha = alpha.max(score);
                    }
                } else {
                    if score < best_score {
                        best_move = Some((from_pos, to_pos, score));
                        beta = beta.min(score);
                    }
                }
            } else {
                best_move = Some((from_pos, to_pos, score));
            }
        }
    }

    best_move
}

/// Quiescence search - continues evaluating capture moves to avoid horizon effect
fn quiescence(
    chess_board: [[Option<Meeple>; 8]; 8],
    last_move: ((usize, usize), (usize, usize)),
    turn: Color,
    mut alpha: f32,
    mut beta: f32,
    turn_meeples: Vec<Meeple>,
    opposite_turn_meeples: Vec<Meeple>,
) -> f32 {
    let stand_pat = calculate_board(chess_board);
    let is_maximizing = turn == Color::White;

    if is_maximizing {
        if stand_pat > alpha {
            alpha = stand_pat;
        }
    } else {
        if stand_pat < beta {
            beta = stand_pat;
        }
    }

    if beta <= alpha {
        return stand_pat;
    }

    for colored_meeple in turn_meeples.iter() {
        for to_pos in colored_meeple.show_moves(&chess_board, &last_move, &opposite_turn_meeples) {
            if chess_board[to_pos.0][to_pos.1].is_none() {
                continue;
            }

            let mut chess_board_clone = chess_board.clone();
            let from_pos = colored_meeple.pos;

            let mut from = chess_board_clone[from_pos.0][from_pos.1].take().unwrap();
            from.pos = to_pos;

            chess_board_clone[to_pos.0][to_pos.1] = Some(from);

            let mut turn_meeples_clone = turn_meeples.clone();
            if let Some(index) = turn_meeples_clone.iter().position(|x| x.pos == from_pos) {
                turn_meeples_clone[index].pos = to_pos;
            }

            let mut opposite_turn_meeples_clone = opposite_turn_meeples.clone();
            if let Some(index) = opposite_turn_meeples_clone
                .iter()
                .position(|x| x.pos == to_pos)
            {
                opposite_turn_meeples_clone.remove(index);
            }

            // Check if move is legal
            let mut legal_move = true;

            for check_meeple in opposite_turn_meeples_clone.iter() {
                if check_meeple
                    .show_moves(&chess_board_clone, &(from_pos, to_pos), &turn_meeples_clone)
                    .contains(&turn_meeples_clone.last().unwrap().pos)
                {
                    legal_move = false;
                    break;
                }
            }

            if !legal_move {
                continue;
            }

            let score = quiescence(
                chess_board_clone,
                (from_pos, to_pos),
                opposite_color(turn),
                alpha,
                beta,
                opposite_turn_meeples_clone,
                turn_meeples_clone,
            );

            if is_maximizing {
                if score > stand_pat {
                    alpha = alpha.max(score);
                }
            } else {
                if score < stand_pat {
                    beta = beta.min(score);
                }
            }

            if beta <= alpha {
                break;
            }
        }
    }

    stand_pat
}

fn get_score_black(
    chess_board: [[Option<Meeple>; 8]; 8],
    last_move: ((usize, usize), (usize, usize)),
    level: u16,
    turn: Color,
    mut alpha: f32,
    mut beta: f32,
    turn_meeples: Vec<Meeple>,
    opposite_turn_meeples: Vec<Meeple>,
) -> f32 {
    let mut best_score = if turn == Color::White {
        f32::NEG_INFINITY
    } else {
        f32::INFINITY
    };
    let is_maximizing = turn == Color::White;
    let mut has_legal_move = false;

    for colored_meeple in turn_meeples.iter() {
        for check_meep in
            colored_meeple.show_moves(&chess_board, &last_move, &opposite_turn_meeples)
        {
            let from_pos = colored_meeple.pos;
            let to_pos = check_meep;
            let mut chess_board_clone = chess_board.clone();

            //does the move
            let mut from = chess_board_clone[from_pos.0][from_pos.1].take().unwrap();
            from.pos = to_pos;

            chess_board_clone[to_pos.0][to_pos.1] = Some(from);

            //updates the meeple positions
            let mut turn_meeples_clone = turn_meeples.clone();
            if let Some(index) = turn_meeples_clone.iter().position(|x| x.pos == from_pos) {
                turn_meeples_clone[index].pos = to_pos;
            }

            let mut opposite_turn_meeples_clone = opposite_turn_meeples.clone();
            if let Some(index) = opposite_turn_meeples_clone
                .iter()
                .position(|x| x.pos == to_pos)
            {
                opposite_turn_meeples_clone.remove(index);
            }

            //checks if the move is legal (king is not in check)
            let mut legal_move = true;

            for check_meeple in opposite_turn_meeples_clone.iter() {
                if check_meeple
                    .show_moves(&chess_board_clone, &(from_pos, to_pos), &turn_meeples_clone)
                    .contains(&turn_meeples_clone.last().unwrap().pos)
                {
                    legal_move = false;
                    break;
                }
            }

            if !legal_move {
                continue;
            }

            has_legal_move = true;

            let score = if level == 0 {
                quiescence(
                    chess_board_clone,
                    (from_pos, to_pos),
                    opposite_color(turn),
                    alpha,
                    beta,
                    opposite_turn_meeples_clone,
                    turn_meeples_clone,
                )
            } else {
                get_score_black(
                    chess_board_clone,
                    (from_pos, to_pos),
                    level - 1,
                    opposite_color(turn),
                    alpha,
                    beta,
                    opposite_turn_meeples_clone,
                    turn_meeples_clone,
                )
            };

            // Update best score and alpha-beta values
            if is_maximizing {
                if score > best_score {
                    best_score = score;
                    alpha = alpha.max(score);
                }
            } else {
                if score < best_score {
                    best_score = score;
                    beta = beta.min(score);
                }
            }

            if beta <= alpha {
                break;
            }
        }
        if beta <= alpha {
            break;
        }
    }

    if !has_legal_move {
        let in_check = opposite_turn_meeples.iter().any(|m| {
            m.show_moves(&chess_board, &last_move, &turn_meeples)
                .contains(&turn_meeples.last().unwrap().pos)
        });

        return if in_check {
            if is_maximizing {
                -10000.0 - level as f32
            } else {
                10000.0 + level as f32
            }
        } else {
            0.0
        };
    }

    best_score
}

pub fn get_meeples_from_color(
    chess_board: &[[Option<Meeple>; 8]; 8],
    color_at_0: Color,
) -> (Vec<Meeple>, Vec<Meeple>) {
    let mut ret_vec: (Vec<Meeple>, Vec<Meeple>) = (Vec::new(), Vec::new());
    let mut kings: (Vec<Meeple>, Vec<Meeple>) = (Vec::new(), Vec::new());
    for y in 0..8 {
        for x in 0..8 {
            if let Some(meeple) = chess_board[x][y] {
                if meeple.color == color_at_0 {
                    if meeple.typ == Type::King {
                        kings.0.push(meeple);
                    } else {
                        ret_vec.0.push(meeple);
                    }
                } else {
                    if meeple.typ == Type::King {
                        kings.1.push(meeple);
                    } else {
                        ret_vec.1.push(meeple);
                    }
                }
            }
        }
    }
    ret_vec.0.append(&mut kings.0);
    ret_vec.1.append(&mut kings.1);
    ret_vec
}

pub fn calculate_board(chess_board: [[Option<Meeple>; 8]; 8]) -> f32 {
    chess_board
        .iter()
        .flat_map(|row| row.iter())
        .filter_map(|opt_meeple| *opt_meeple)
        .fold(0.0, |score, meeple| {
            let piece_score = meeple.value + calculate_meeple_positon(meeple.value, meeple.pos);
            match meeple.color {
                Color::White => score + piece_score,
                Color::Black => score - piece_score,
            }
        })
}

fn calculate_meeple_positon(value: f32, pos: (usize, usize)) -> f32 {
    const PAWN_WHITE: [[f32; 8]; 8] = [
        [0.2, 0.3, 0.0, 0.0, 0.0, 0.0, 0.3, 0.2],
        [0.2, 0.2, 0.0, 0.0, 0.0, 0.0, 0.2, 0.2],
        [-0.1, -0.2, -0.2, -0.2, -0.2, -0.2, -0.2, -0.1],
        [-0.2, -0.3, -0.3, -0.4, -0.4, -0.3, -0.3, -0.2],
        [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
        [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
        [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
        [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
    ];

    const PAWN_BLACK: [[f32; 8]; 8] = [
        [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
        [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
        [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
        [-0.2, -0.3, -0.3, -0.4, -0.4, -0.3, -0.3, -0.2],
        [-0.1, -0.2, -0.2, -0.2, -0.2, -0.2, -0.2, -0.1],
        [0.2, 0.2, 0.0, 0.0, 0.0, 0.0, 0.2, 0.2],
        [0.2, 0.3, 0.0, 0.0, 0.0, 0.0, 0.3, 0.2],
        [0.2, 0.3, 0.0, 0.0, 0.0, 0.0, 0.3, 0.2],
    ];

    const KNIGHT_BLACK: [[f32; 8]; 8] = [
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        [0.1, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.1],
        [0.05, 0.05, 0.1, 0.25, 0.25, 0.1, 0.05, 0.05],
        [0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0, 0.0],
        [0.05, -0.05, -0.1, 0.0, 0.0, -0.1, -0.05, 0.05],
        [0.05, 0.1, 0.1, -0.2, -0.2, 0.1, 0.1, 0.05],
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    ];

    const KNIGHT_WHITE: [[f32; 8]; 8] = [
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        [0.05, 0.1, 0.1, -0.2, -0.2, 0.1, 0.1, 0.05],
        [0.05, -0.05, -0.1, 0.0, 0.0, -0.1, -0.05, 0.05],
        [0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0, 0.0],
        [0.05, 0.05, 0.1, 0.25, 0.25, 0.1, 0.05, 0.05],
        [0.1, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.1],
        [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    ];

    const BISHOP_WHITE: [[f32; 8]; 8] = [
        [0.0, 0.0, 0.05, 0.1, 0.1, 0.05, 0.0, 0.0],
        [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
        [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
        [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
        [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
        [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
        [0.05, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.05],
        [0.0, 0.0, 0.05, 0.1, 0.1, 0.05, 0.0, 0.0],
    ];

    const BISHOP_BLACK: [[f32; 8]; 8] = [
        [0.0, 0.0, 0.05, 0.1, 0.1, 0.05, 0.0, 0.0],
        [0.05, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.05],
        [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
        [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
        [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
        [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
        [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
        [0.0, 0.0, 0.05, 0.1, 0.1, 0.05, 0.0, 0.0],
    ];

    const ROOK_WHITE: [[f32; 8]; 8] = [
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        [0.1, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.1],
        [0.05, 0.05, 0.1, 0.25, 0.25, 0.1, 0.05, 0.05],
        [0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0, 0.0],
        [0.05, -0.05, -0.1, 0.0, 0.0, -0.1, -0.05, 0.05],
        [0.05, 0.1, 0.1, -0.2, -0.2, 0.1, 0.1, 0.05],
        [0.3, 0.0, 0.0, 0.3, 0.0, 0.3, 0.0, 0.3],
    ];

    const ROOK_BLACK: [[f32; 8]; 8] = [
        [0.3, 0.0, 0.0, 0.3, 0.0, 0.3, 0.0, 0.3],
        [0.05, 0.1, 0.1, -0.2, -0.2, 0.1, 0.1, 0.05],
        [0.05, -0.05, -0.1, 0.0, 0.0, -0.1, -0.05, 0.05],
        [0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0, 0.0],
        [0.05, 0.05, 0.1, 0.25, 0.25, 0.1, 0.05, 0.05],
        [0.1, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.1],
        [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    ];

    const QUEEN_WHITE: [[f32; 8]; 8] = [
        [-0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2],
        [-0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.1],
        [-0.1, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.1],
        [-0.05, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.05],
        [0.0, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, 0.0],
        [-0.1, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05, -0.1],
        [-0.1, 0.0, 0.05, 0.0, 0.0, 0.0, 0.0, -0.1],
        [-0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2],
    ];

    const QUEEN_BLACK: [[f32; 8]; 8] = [
        [-0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2],
        [-0.1, 0.0, 0.05, 0.0, 0.0, 0.0, 0.0, -0.1],
        [-0.1, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05, -0.1],
        [0.0, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, 0.0],
        [-0.05, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.05],
        [-0.1, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.1],
        [-0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.1],
        [-0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2],
    ];

    const KING_POS_WHITE: [[f32; 8]; 8] = [
        [-0.2, -0.1, 0.0, 0.1, 0.1, 0.0, -0.1, -0.2],
        [-0.1, 0.0, 0.1, 0.2, 0.2, 0.1, 0.0, -0.1],
        [0.0, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.0],
        [0.1, 0.2, 0.3, 0.4, 0.4, 0.3, 0.2, 0.1],
        [0.1, 0.2, 0.3, 0.4, 0.4, 0.3, 0.2, 0.1],
        [0.0, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.0],
        [-0.1, 0.0, 0.1, 0.2, 0.2, 0.1, 0.0, -0.1],
        [-0.2, 0.2, 0.5, 0.1, 0.1, 0.0, 0.5, -0.2],
    ];

    const KING_POS_BLACK: [[f32; 8]; 8] = [
        [-0.2, 0.2, 0.5, 0.1, 0.1, 0.0, 0.5, -0.2],
        [-0.1, 0.0, 0.1, 0.2, 0.2, 0.1, 0.0, -0.1],
        [0.0, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.0],
        [0.1, 0.2, 0.3, 0.4, 0.4, 0.3, 0.2, 0.1],
        [0.1, 0.2, 0.3, 0.4, 0.4, 0.3, 0.2, 0.1],
        [0.0, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.0],
        [-0.1, 0.0, 0.1, 0.2, 0.2, 0.1, 0.0, -0.1],
        [-0.2, -0.1, 0.0, 0.1, 0.1, 0.0, -0.1, -0.2],
    ];

    match value {
        1.0 => PAWN_WHITE[pos.0][pos.1],
        1.1 => PAWN_BLACK[pos.0][pos.1],
        2.7 => KNIGHT_WHITE[pos.0][pos.1],
        2.8 => KNIGHT_BLACK[pos.0][pos.1],
        3.0 => BISHOP_WHITE[pos.0][pos.1],
        3.1 => BISHOP_BLACK[pos.0][pos.1],
        5.0 => ROOK_WHITE[pos.0][pos.1],
        5.1 => ROOK_BLACK[pos.0][pos.1],
        9.0 => QUEEN_WHITE[pos.0][pos.1],
        9.1 => QUEEN_BLACK[pos.0][pos.1],
        10.0 => KING_POS_WHITE[pos.0][pos.1],
        10.1 => KING_POS_BLACK[pos.0][pos.1],
        _ => 0.0,
    }
}
