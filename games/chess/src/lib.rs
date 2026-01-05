use std::{thread::{self, sleep}, time::Duration, usize};

use egui::Ui;
use game_core::Game;

mod meeples;
use crate::{draw::draw_board, meeples::{Color, Meeple, Type, opposite_color}};
//als nächtest evaluation, multiplayer und engine
mod draw;
//performance: moves woanders, im heap mehr zeug speichern, stack vergrößern und alles in neuem thread dafür tun
pub struct ChessGame {
    state: String,
    game_board: [[Option<Meeple>;8];8],
    possible_moves: [[Option<Vec<(usize,usize)>>;8];8],       
    shown_moves: Option<Vec<(usize,usize)>>,                    
    logs: Vec<((usize,usize),(usize,usize))>,  
    clicked_meeple: (usize,usize),
    turn: Color,
    pawn_mutate: bool,
}

impl ChessGame {
    pub fn new() -> Self {
        let state_ = "Initial Chess State".to_string();
        let mut chess_board:[[Option<Meeple>;8];8] = Default::default();
        let logs_  = vec![((42,42),(42,42))];
        let turn_ = Color::White;
        for x in 0..=7 {
            for y in 0..=7 {
                match y.clone() {
                    0 => chess_board[x][y] = Some(ChessGame::create_special_line((x.try_into().unwrap(),y.try_into().unwrap()), Color::Black)),
                    1 => chess_board[x][y] = Some(Meeple::new((x.try_into().unwrap(),y.try_into().unwrap()), Type::Pawn, Color::Black,1.0)),
                    6 => chess_board[x][y] = Some(Meeple::new((x.try_into().unwrap(),y.try_into().unwrap()), Type::Pawn, Color::White,1.0)),
                    7 => chess_board[x][y] = Some(ChessGame::create_special_line((x.try_into().unwrap(),y.try_into().unwrap()), Color::White)),
                    _ => continue,
                }
            }
        }
        let possible_moves_ = create_basic_possible_moves();
        Self {state: state_,game_board: chess_board, possible_moves: possible_moves_, shown_moves: None,logs: logs_, clicked_meeple: (42,42),turn: turn_,pawn_mutate: false}
    }
    
    fn create_special_line(cords: (usize,usize),color: Color) -> Meeple {
        match cords.0 {
            0 | 7 => Meeple::new(cords, Type::Rook, color,5.0),
            1 | 6 => Meeple::new(cords, Type::Knight, color,3.0),
            2 | 5 => Meeple::new(cords, Type::Bishop, color,3.0),
            3 => Meeple::new(cords, Type::Queen, color,9.0),
            4 => Meeple::new(cords, Type::King, color,0.0),
            _ => panic!("Something went wrong while creating a special row"),
        }
    }   

    pub fn show_moves(&mut self,(x,y):(usize,usize)) {
        self.shown_moves = self.possible_moves[x] [y].clone();
        self.clicked_meeple = (x,y);
    }

    pub fn move_meeple(&mut self,scnd: (usize,usize)) {
        let frst = self.clicked_meeple.clone();

        if self.check_casteling(frst, scnd) {
            self.casteling_meeple(frst, scnd);
        } else if self.check_en_passant(frst, scnd) {
            self.game_board[scnd.0] [frst.1] = None;
        }
        walk_and_replace(frst, scnd,&mut self.game_board);

        self.game_board[scnd.0] [scnd.1].as_mut().unwrap().move_counter += 1;
        self.logs.push((self.clicked_meeple,scnd));   
        self.shown_moves = Default::default();
        self.check_pawn_mutate(scnd); 
        self.turn = opposite_color(self.turn);
        self.possible_moves = self.get_all_possible_moves(); 
        println!("{}",calculate_board(self.game_board));
    }  

    fn check_casteling(&self, frst_pos: (usize,usize), scnd_pos: (usize,usize)) -> bool {
        let frst = self.game_board[frst_pos.0] [frst_pos.1].unwrap();
        let cmp_value = frst.pos.0 as i8 - scnd_pos.0 as i8 ;
        if frst.typ == Type::King && (cmp_value == 2 || cmp_value == -2) {
            return true;
        }
        false
    }

    fn casteling_meeple(&mut self, frst: (usize,usize),scnd: (usize,usize)) {     
        let cmp_value = frst.0 as i8 - scnd.0 as i8;
        if cmp_value < 0 {
            let new_rook_pos:(usize,usize) = (5,scnd.1);
            walk_and_replace((7,scnd.1), new_rook_pos,&mut self.game_board); 
        } else {
            let new_rook_pos:(usize,usize) = (3,scnd.1);
            walk_and_replace((0,scnd.1), new_rook_pos,&mut self.game_board);
        }
    }
    
    fn check_en_passant(&self, frst_pos: (usize,usize), scnd_pos: (usize,usize)) -> bool {
        let frst = self.game_board[frst_pos.0] [frst_pos.1].unwrap();
        if frst.typ == Type::Pawn && self.game_board[scnd_pos.0] [scnd_pos.1] == None {
            if let Some(opposite_color_pawn) = self.game_board[scnd_pos.0] [frst_pos.1] {
                if opposite_color_pawn.typ == Type::Pawn && opposite_color_pawn.color != frst.color {
                    return true;
                }
            }
        }
        false
    }

    fn check_pawn_mutate(&mut self, scnd_pos: (usize,usize)) {
        if let Some(pawn) = self.game_board[scnd_pos.0] [scnd_pos.1].as_mut() {
            if pawn.typ == Type::Pawn && ((pawn.color == Color::White && pawn.pos.1 == 0) || (pawn.color == Color::Black && pawn.pos.1 == 7)){
                self.pawn_mutate = true;
            }
        }
    }

    fn mutate_pawn(&mut self,mutate_into: Type) {
        let pawn_pos = self.logs.last().unwrap().1;
        if let Some(pawn) = self.game_board[pawn_pos.0] [pawn_pos.1].as_mut() {
            pawn.typ = mutate_into;
            pawn.value = match pawn.typ {
                Type::Queen => 9.0,
                Type::Rook => 5.0,
                Type::Bishop => 3.0,
                Type::Knight => 3.0,
                _ => panic!("This should not happen"),
            };
            println!("he changed");
        }
        self.pawn_mutate = false;
    }
    
    fn get_all_possible_moves(&mut self) -> [[Option<Vec<(usize,usize)>>;8];8] {
        let mut chess_board = self.game_board.clone();
        let turn = self.turn.clone();
        let last_move = self.logs.last().unwrap().clone();
        let handle= thread::spawn(move || {
            get_all_possible_moves(&mut chess_board,turn,&last_move)
        });
        handle.join().unwrap() 
    }
}

fn walk_and_replace(frst: (usize,usize), scnd: (usize,usize),chess_board: &mut [[Option<Meeple>;8];8]) {
    chess_board[scnd.0] [scnd.1] = chess_board[frst.0] [frst.1].take();
    chess_board[scnd.0] [scnd.1].as_mut().unwrap().pos =scnd;
}

fn get_all_possible_moves(chess_board: &mut [[Option<Meeple>;8];8],turn: Color,last_move: &((usize,usize),(usize,usize))) -> [[Option<Vec<(usize,usize)>>;8];8] {
    let mut ret_vec:[[Option<Vec<(usize,usize)>>;8];8] = Default::default();
    let mut colores = get_meeples_from_color(&chess_board, turn);
    let king = colores.0.last().unwrap().pos;
    let mut can_move = false;
    for colored_meeple in colores.0 {
        let can_hit = check_meeple_moves_valid(chess_board,last_move,&colored_meeple,&mut colores.1,king);
        if !can_move && can_hit.is_empty()  {
            can_move = true;
        }
        ret_vec[colored_meeple.pos.0] [colored_meeple.pos.1] = Some(can_hit);
    }

    if !can_move {
        println!("jmd hat gewonnen");
    }
    ret_vec
}

fn check_meeple_moves_valid(chess_board: &mut [[Option<Meeple>;8];8],last_move: &((usize,usize),(usize,usize)),meeple: &Meeple,check_color: &mut Vec<Meeple>,king: (usize,usize)) -> Vec<(usize,usize)>{
    let mut ret_vec:Vec<(usize,usize)> = Vec::new();
    for check_meep in meeple.show_moves(&chess_board, last_move) {
        if check_the_future(chess_board,&meeple, check_meep,check_color,king) {
            ret_vec.push(check_meep);
        } 
    }

    ret_vec
}

fn check_the_future(chess_board: &mut [[Option<Meeple>;8];8], current_meeple: &Meeple,to_move: (usize,usize),check_color: &mut Vec<Meeple>, king: (usize,usize)) -> bool {
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
    
    //undo
    chess_board[from_pos.0] [from_pos.1] = chess_board[to_pos.0] [to_pos.1].take();
    chess_board[from_pos.0] [from_pos.1].as_mut().unwrap().pos = from_pos;
    chess_board[to_pos.0] [to_pos.1] = to;

    if let Some(meep) = add_meep {
        check_color.push(meep);
    }
    !can_hit
}   

fn create_basic_possible_moves() -> [[Option<Vec<(usize,usize)>>;8];8] {
    let mut ret_vec:[[Option<Vec<(usize,usize)>>;8];8] = Default::default();
    for index in 0..8 {
        ret_vec[index] [6] = Some(vec![(index,5),(index,4)]);
    }
    ret_vec[1] [7] = Some(vec![(0,5),(2,5)]);
    ret_vec[6] [7] = Some(vec![(7,5),(5,5)]);
    ret_vec
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

fn calculate_meeple_positon(value: f32,pos: (usize,usize)) -> f32{
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

impl Game for ChessGame {
    fn name(&self) -> &str {
        "Chess"
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Chess");
        draw_board(ui, self);

    }
}
