use std::thread;
use std::time::Instant;

use crate::meeples::{ opposite_color, Color, Meeple, Type};
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
        let last_move_t = *last_move;
        let level_t = self.level;
        let start = Instant::now();
        let meeples = get_meeples_from_color(&chess_board_t, turn);
        let handle = thread::spawn(move || {
            get_best_move_black(
                chess_board_t,
                last_move_t,
                level_t,
                turn_t,
                f32::NEG_INFINITY,
                f32::INFINITY,
                meeples.0,
                meeples.1,
            )
        });
        let ret = handle.join().unwrap();
        let duration = start.elapsed();
        print!(
            "Engine thinking time: {:?} with the move {:?}\n",
            duration, ret
        );
        if let Some((from, to, _)) = ret {
            (from, to)
        } else {
            ((42, 42), (42, 42))
        }
    }
}

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
    for colored_meeple in turn_meeples.iter() {
        for check_meep in colored_meeple.show_moves(&chess_board, &last_move,&opposite_turn_meeples) {
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

            //checks if the move is legal
            let mut legal_move = true;
            for check_meeple in opposite_turn_meeples_clone.iter() {
                if check_meeple
                    .show_moves(&chess_board_clone, &(from_pos, to_pos),&turn_meeples)
                    .contains(&turn_meeples_clone.last().unwrap().pos)
                {
                    legal_move = false;
                    break;
                }
            }

            //gets the best move from deeper recursion or the end of the recursion
            let score = calculate_board(chess_board_clone);
            if legal_move {
                if level == 0 {
                    if let Some((_, _, best_score)) = best_move {
                        if score < best_score {
                            best_move = Some((from_pos, to_pos, score));
                        }
                    } else {
                        best_move = Some((from_pos, to_pos, score));
                    }
                } else {
                    if let Some((_, _, opp_score)) = get_best_move_black(
                        chess_board_clone,
                        (from_pos, to_pos),
                        level - 1,
                        opposite_color(turn),
                        alpha.clone(),
                        beta.clone(),
                        opposite_turn_meeples_clone,
                        turn_meeples_clone,
                    ) {
                        if let Some((_, _, best_score)) = best_move {
                            if opp_score < best_score {
                                best_move = Some((from_pos, to_pos, opp_score));
                            }
                        } else {
                            best_move = Some((from_pos, to_pos, score));
                        }
                    }
                }
            }
            if turn == Color::White {
                alpha = alpha.max(score);
            } else {
                beta = beta.min(score);
            }
            if beta <= alpha {
                break;
            }
        }
        if beta <= alpha {
            break;
        }
    }
    best_move
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
    let mut total_score = 0.0;
    for array in chess_board {
        for opt_meeple in array {
            if let Some(meeple) = opt_meeple {
                // if meeple.typ == Type::King {    hier noch zufügen dass er angegriffen wird
                //     if meeple.show_moves(&chess_board, &((0,0),(0,0))).len() == 0 {
                //         if meeple.color == Color::White {
                //             return 999.0;
                //         } else {
                //             return -999.0;
                //         }
                //     }
                // }

                match meeple.color {
                    Color::White => {
                        total_score += calculate_meeple_positon(meeple.value, meeple.pos);
                        total_score += meeple.value;
                    }
                    Color::Black => {
                        total_score -= calculate_meeple_positon(meeple.value, meeple.pos);
                        total_score -= meeple.value;
                    }
                }
            }
        }
    }
    total_score
}

fn calculate_meeple_positon(value: f32, pos: (usize, usize)) -> f32 {
    let pos_array = match value {
        0.0 => [
            [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
            [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
            [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
            [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
            [-0.2, -0.3, -0.3, -0.4, -0.4, -0.3, -0.3, -0.2],
            [-0.1, -0.2, -0.2, -0.2, -0.2, -0.2, -0.2, -0.1],
            [0.2, 0.2, 0.0, 0.0, 0.0, 0.0, 0.2, 0.2],
            [0.2, 0.3, 0.0, 0.0, 0.0, 0.0, 0.3, 0.2],
        ],
        0.1 => [
            [0.2, 0.3, 0.0, 0.0, 0.0, 0.0, 0.3, 0.2],
            [0.2, 0.2, 0.0, 0.0, 0.0, 0.0, 0.2, 0.2],
            [-0.1, -0.2, -0.2, -0.2, -0.2, -0.2, -0.2, -0.1],
            [-0.2, -0.3, -0.3, -0.4, -0.4, -0.3, -0.3, -0.2],
            [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
            [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
            [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
            [-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3],
        ],
        1.0 => [
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            [0.1, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.1],
            [0.05, 0.05, 0.1, 0.25, 0.25, 0.1, 0.05, 0.05],
            [0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0, 0.0],
            [0.05, -0.05, -0.1, 0.0, 0.0, -0.1, -0.05, 0.05],
            [0.05, 0.1, 0.1, -0.2, -0.2, 0.1, 0.1, 0.05],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ],
        1.1 => [
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.05, 0.1, 0.1, -0.2, -0.2, 0.1, 0.1, 0.05],
            [0.05, -0.05, -0.1, 0.0, 0.0, -0.1, -0.05, 0.05],
            [0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0, 0.0],
            [0.05, 0.05, 0.1, 0.25, 0.25, 0.1, 0.05, 0.05],
            [0.1, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.1],
            [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ],
        5.0 => [
            [0.0, 0.0, 0.05, 0.1, 0.1, 0.05, 0.0, 0.0],
            [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
            [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
            [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
            [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
            [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
            [0.05, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.05],
            [0.0, 0.0, 0.05, 0.1, 0.1, 0.05, 0.0, 0.0],
        ],
        5.1 => [
            [0.0, 0.0, 0.05, 0.1, 0.1, 0.05, 0.0, 0.0],
            [0.05, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.05],
            [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
            [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
            [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
            [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
            [-0.05, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, -0.05],
            [0.0, 0.0, 0.05, 0.1, 0.1, 0.05, 0.0, 0.0],
        ],

        9.0 => [
            [-0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2],
            [-0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.1],
            [-0.1, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.1],
            [-0.05, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.05],
            [0.0, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, 0.0],
            [-0.1, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05, -0.1],
            [-0.1, 0.0, 0.05, 0.0, 0.0, 0.0, 0.0, -0.1],
            [-0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2],
        ],
        9.1 => [
            [-0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2],
            [-0.1, 0.0, 0.05, 0.0, 0.0, 0.0, 0.0, -0.1],
            [-0.1, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05, -0.1],
            [0.0, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, 0.0],
            [-0.05, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.05],
            [-0.1, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.1],
            [-0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.1],
            [-0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2],
        ],
        _ => [
            [-0.2, -0.1, 0.0, 0.1, 0.1, 0.0, -0.1, -0.2],
            [-0.1, 0.0, 0.1, 0.2, 0.2, 0.1, 0.0, -0.1],
            [0.0, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.0],
            [0.1, 0.2, 0.3, 0.4, 0.4, 0.3, 0.2, 0.1],
            [0.1, 0.2, 0.3, 0.4, 0.4, 0.3, 0.2, 0.1],
            [0.0, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.0],
            [-0.1, 0.0, 0.1, 0.2, 0.2, 0.1, 0.0, -0.1],
            [-0.2, -0.1, 0.0, 0.1, 0.1, 0.0, -0.1, -0.2],
        ],
    };

    pos_array[pos.0][pos.1]
}
