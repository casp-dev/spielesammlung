use std::thread;
use std::time::Instant;

use egui::epaint::color;

use crate::meeples::{self, opposite_color, Color, Meeple, Type};
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
        possible_moves: &[[Option<Vec<(usize, usize)>>; 8]; 8],
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
        for check_meep in colored_meeple.show_moves(&chess_board, &last_move) {
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
                    .show_moves(&chess_board_clone, &(from_pos, to_pos))
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

// fn get_best_move_black(chess_board: &mut [[Option<Meeple>;8];8],last_move: &((usize,usize),(usize,usize)),level: u16,turn: Color, alpha: &mut f32, beta: &mut f32) -> Option<((usize,usize),(usize,usize),f32)> {
//     let mut alpha_new = *alpha;
//     let mut beta_new = *beta;
//     let mut best_move: Option<((usize, usize), (usize, usize), f32)> = None;
//     let (mut white,mut black) = get_meeples_from_color(chess_board, turn);
//     let run_color_meeples ;
//     let in_color_meeples ;
//     if turn == Color::White {
//         run_color_meeples = &mut black;
//         in_color_meeples = &mut white;
//     } else {
//         run_color_meeples = &mut white;
//         in_color_meeples = &mut black;
//     };
//     for colored_meeple in run_color_meeples.iter() {
//         for check_meep in colored_meeple.show_moves(&chess_board, last_move) {
//             let from_pos = colored_meeple.pos;
//             let to_pos = check_meep;
//             let to = chess_board[to_pos.0] [to_pos.1].take();
//             let mut add_meep:Option<Meeple> = None;

//             if let Some(index) = in_color_meeples.iter().position(|&x| x.pos == check_meep) {
//                 add_meep = Some(in_color_meeples.swap_remove(index));
//             }

//             //try
//             chess_board[to_pos.0] [to_pos.1] = chess_board[from_pos.0] [from_pos.1].take();
//             chess_board[to_pos.0] [to_pos.1].as_mut().unwrap().pos =to_pos;

//             //check if king can be hit
//             let mut can_hit = false;
//             for check_meeple in in_color_meeples.iter() {
//                 if check_meeple.show_moves(chess_board, &(colored_meeple.pos,check_meep)).contains(if colored_meeple.typ == Type::King {&check_meep} else {&run_color_meeples.last().unwrap().pos}) {
//                     can_hit = true;
//                     break;
//                 }
//             }
//             let mut best_hit = None;
//             if !can_hit {
//                 if level == 0 {
//                     let score = calculate_board(*chess_board);
//                     best_hit = Some((from_pos,to_pos,score));
//                 }
//                 if level != 0 {
//                     if let Some((_,_,opp_score)) = get_best_move_black(chess_board, &(from_pos,to_pos), level-1,opposite_color(turn), &mut alpha_new, &mut beta_new) {
//                         best_hit = Some((from_pos,to_pos,opp_score));
//                     }
//                 }
//             }

//             //undo
//             chess_board[from_pos.0] [from_pos.1] = chess_board[to_pos.0] [to_pos.1].take();
//             chess_board[from_pos.0] [from_pos.1].as_mut().unwrap().pos = from_pos;
//             chess_board[to_pos.0] [to_pos.1] = to;

//             if let Some(meep) = add_meep {
//                 in_color_meeples.push(meep);
//             }

//             if let Some(values) = best_hit {
//                 let score = values.2;

//                 if best_move.is_none() || (turn == Color::White && score > best_move.unwrap().2) || (turn == Color::Black && score < best_move.unwrap().2){
//                     best_move = Some(values);
//                 }

//                 if turn == Color::White {
//                     alpha_new = alpha_new.max(score);
//                 } else {
//                     beta_new = beta_new.min(score);
//                 }
//             }

//             if alpha_new >= beta_new || alpha >= beta {
//                 break;
//             }
//         }
//     }
//     best_move
// }

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
