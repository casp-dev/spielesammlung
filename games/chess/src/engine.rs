
use std::thread;

use crate::meeples::{Color,Meeple,Type, opposite_color};
#[derive(Clone,Copy)]
pub struct Engine {
    pub level: u16,
    pub color: Color,
}

impl Engine {
    pub fn new(level_: u16,color: Color) -> Engine {
        Engine { level: level_, color: color }
    }

    pub fn move_move(&self, chess_board: &mut [[Option<Meeple>;8];8], last_move: &((usize,usize),(usize,usize)),turn: Color,possible_moves: &[[Option<Vec<(usize,usize)>>;8];8]) -> ((usize,usize),(usize,usize)) {
        let mut chess_board_t = chess_board.clone();
        let turn_t = turn;  
        let last_move_t = *last_move;  
        let level_t = self.level;  
        let handle = thread::spawn(move || {
            get_all_possible_moves(&mut chess_board_t, &last_move_t, level_t, turn_t)
        });
        let ret = handle.join().unwrap();
        if let Some((from, to, score)) = ret {
            (from, to)
        } else {
            ((42, 42), (42, 42))
        }
    
    }
}

fn get_all_possible_moves(chess_board: &mut [[Option<Meeple>;8];8],last_move: &((usize,usize),(usize,usize)),level: u16,turn: Color) -> Option<((usize,usize),(usize,usize),f32)> {
    let mut ret_val: Option<((usize, usize), (usize, usize), f32)> = None;
    let mut colores = get_meeples_from_color(&chess_board, turn);
    let king = colores.0.last().unwrap().pos;
    for colored_meeple in colores.0 {
        if let Some(values) = check_meeple_moves_valid(chess_board, last_move, &colored_meeple, &mut colores.1, king, level, turn) {
            if let Some(current_value) = ret_val {
                if current_value.2 < values.2 {
                    ret_val = Some(values);
                }
            } else {
                ret_val = Some(values);
            }
        }
    }
    ret_val
}

fn check_meeple_moves_valid(chess_board: &mut [[Option<Meeple>;8];8],last_move: &((usize,usize),(usize,usize)),meeple: &Meeple,check_color: &mut Vec<Meeple>,king: (usize,usize),level: u16,turn: Color) -> Option<((usize,usize),(usize,usize),f32)> {
    let mut ret_val:Option<((usize,usize),(usize,usize),f32)> = None;
    for check_meep in meeple.show_moves(&chess_board, last_move) {
        if let Some(values) = check_the_future(chess_board,&meeple, check_meep,check_color,king,level,turn) {
            if let Some(current_value) = ret_val {
                if current_value.2 < values.2 {
                    ret_val = Some(values);
                }
            } else {
                ret_val = Some(values);
            }
        }
    }
    ret_val
}

fn check_the_future(chess_board: &mut [[Option<Meeple>;8];8], current_meeple: &Meeple,to_move: (usize,usize),check_color: &mut Vec<Meeple>, king: (usize,usize),level: u16,turn: Color) -> Option<((usize,usize),(usize,usize),f32)> {
    //vars
    let king_king = if current_meeple.typ == Type::King {
        to_move
    } else {
        king
    };
    let mut add_meep:Option<Meeple> = None;
    if let Some(index) = check_color.iter().position(|&x| x.pos == to_move) {
        add_meep = Some(check_color.swap_remove(index));
    }
    let from_pos = current_meeple.pos;
    let to_pos = to_move;
    let to = chess_board[to_pos.0] [to_pos.1].take();
        
    //try
    walk_and_replace(from_pos, to_pos, chess_board);

    //check if king can be hit
    let mut can_hit = false;
    for check_meeple in check_color.iter() {
        if check_meeple.show_moves(chess_board, &(current_meeple.pos,to_move)).contains(&king_king) {
            can_hit = true;
            break;
        }
    }
    let mut best_hit = None;
    if !can_hit {
        if level == 0 {
            let score = calculate_board(*chess_board);
            best_hit = Some((from_pos,to_pos,score));
        }
        if level != 0 {
            if let Some((_,_,opp_score)) = get_all_possible_moves(chess_board, &(from_pos,to_pos), level-1,opposite_color(turn)) {
                best_hit = Some((from_pos,to_pos,opp_score));
            } 
        }
    }

    //undo
    chess_board[from_pos.0] [from_pos.1] = chess_board[to_pos.0] [to_pos.1].take();
    chess_board[from_pos.0] [from_pos.1].as_mut().unwrap().pos = from_pos;
    chess_board[to_pos.0] [to_pos.1] = to;

    if let Some(meep) = add_meep {
        check_color.push(meep);
    }
    
    best_hit
}   

pub fn get_meeples_from_color(chess_board:&[[Option<Meeple>;8];8],color_at_0: Color) -> (Vec<Meeple>,Vec<Meeple>) {
    let mut ret_vec:(Vec<Meeple>,Vec<Meeple>) = (Vec::new(),Vec::new());
    let mut kings:(Vec<Meeple>,Vec<Meeple>) = (Vec::new(),Vec::new());
    for y in 0..8 {
        for x in 0..8 {
            if let Some(meeple) = chess_board[x] [y] {
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

fn walk_and_replace(frst: (usize,usize), scnd: (usize,usize),chess_board: &mut [[Option<Meeple>;8];8]) {
    chess_board[scnd.0] [scnd.1] = chess_board[frst.0] [frst.1].take();
    chess_board[scnd.0] [scnd.1].as_mut().unwrap().pos =scnd;
}

pub fn calculate_board(chess_board:[[Option<Meeple>;8];8]) -> f32 {
    let mut total_score = 0.0;
    for col in chess_board {
        for opt_meeple in col {
            if let Some(meeple) = opt_meeple {
                if meeple.color == Color::White {
                    total_score += calculate_meeple_positon(meeple.value,meeple.pos);
                    total_score += meeple.value;
                } else {
                    total_score -= calculate_meeple_positon(meeple.value+0.1,meeple.pos);
                    total_score -= meeple.value;
                }
            }
        }
    }
    total_score
}

fn calculate_meeple_positon(value: f32,pos: (usize,usize)) -> f32 {
    let pos_array = match value {
        0.1 => [[0.5, 0.2, 0.0, 0.0, -0.1, -0.2, -0.3, -0.3],[0.4, 0.2, 0.0, 0.0, 0.0, -0.1, -0.2, -0.2],[0.1, 0.1, 0.1, 0.0, 0.0, 0.0, -0.1, -0.1],[0.2, 0.1, 0.0, 0.0, 0.0, 0.0, -0.1, -0.1],
        [0.2, 0.1, 0.0, 0.0, 0.0, 0.0, -0.1, -0.1],[0.1, 0.1, 0.1, 0.0, 0.0, 0.0, -0.1, -0.1],[0.4, 0.2, 0.0, 0.0, 0.0, -0.1, -0.2, -0.2],[0.5, 0.2, 0.0, 0.0, -0.1, -0.2, -0.3, -0.3]],
        0.0 => [[-0.3,-0.3,-0.2,-0.1,0.0,0.0,0.2,0.5],[-0.2,-0.2,-0.1,0.0,0.0,0.0,0.2,0.4],[-0.1,-0.1,0.0,0.0,0.0,0.1,0.1,0.1],[-0.1,-0.1,0.0,0.0,0.0,0.0,0.1,0.2],
        [-0.1,-0.1,0.0,0.0,0.0,0.0,0.1,0.2],[-0.1,-0.1,0.0,0.0,0.0,0.1,0.1,0.1],[-0.2,-0.2,-0.1,0.0,0.0,0.0,0.2,0.4],[-0.3,-0.3,-0.2,-0.1,0.0,0.0,0.2,0.5]],
        _ => [[-0.5,-0.4,-0.3,-0.2,-0.2,-0.3,-0.4,-0.5],[-0.3,-0.2,-0.1,-0.1,-0.1,-0.1,-0.2,-0.3],[0.0,0.0,0.1,0.2,0.2,0.1,0.0,0.0],[0.2,0.3,0.4,0.5,0.5,0.4,0.3,0.2],
        [0.2,0.3,0.4,0.5,0.5,0.4,0.3,0.2],[0.0,0.0,0.1,0.2,0.2,0.1,0.0,0.0],[-0.3,-0.2,-0.1,-0.1,-0.1,-0.1,-0.2,-0.3],[-0.5,-0.4,-0.3,-0.2,-0.2,-0.3,-0.4,-0.5]],
    };
    pos_array[pos.0] [pos.1]
}