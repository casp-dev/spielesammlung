use std::usize;

use egui::Ui;
use game_core::Game;

mod meeples;
use crate::{draw::draw_board, meeples::{Color, Meeple, Type, opposite_color}};

mod draw;
//show_moves: die ganzen checks, castelling: auslagern, wenn es geht, turn 

pub struct ChessGame {
    state: String,
    game_board: [[Option<Meeple>;8];8],
    possible_moves: [[Option<Vec<(usize,usize)>>;8];8],
    kings: ((usize,usize),(usize,usize)),               //(white,black)
    shown_moves: Option<Vec<(usize,usize)>>,
    logs: Vec<((usize,usize),(usize,usize))>,  
    clicked_meeple: (usize,usize),
    turn: Color,
}

impl ChessGame {
    pub fn new() -> Self {
        let state_ = "Initial Chess State".to_string();
        let mut chess_board:[[Option<Meeple>;8];8] = Default::default();
        let kings_:((usize,usize),(usize,usize)) = ((4,7),(4,0));
        let logs_  = vec![((42,42),(42,42))];
        let turn_ = Color::White;
        for x in 0..=7 {
            for y in 0..=7 {
                match y.clone() {
                    0 => chess_board[x][y] = Some(ChessGame::create_special_line((x.try_into().unwrap(),y.try_into().unwrap()), Color::Black)),
                    1 => chess_board[x][y] = Some(Meeple::new((x.try_into().unwrap(),y.try_into().unwrap()), Type::Pawn, Color::Black)),
                    6 => chess_board[x][y] = Some(Meeple::new((x.try_into().unwrap(),y.try_into().unwrap()), Type::Pawn, Color::White)),
                    7 => chess_board[x][y] = Some(ChessGame::create_special_line((x.try_into().unwrap(),y.try_into().unwrap()), Color::White)),
                    _ => continue,
                }
            }
        }
        let possible_moves_ = create_basic_possible_moves();

        Self {state: state_,game_board: chess_board, possible_moves: possible_moves_, kings: kings_, shown_moves: None,logs: logs_, clicked_meeple: (42,42),turn: turn_}
    }
    
    fn create_special_line(cords: (usize,usize),color: Color) -> Meeple {
        match cords.0 {
            0 | 7 => Meeple::new(cords, Type::Rook, color),
            1 | 6 => Meeple::new(cords, Type::Knight, color),
            2 | 5 => Meeple::new(cords, Type::Bishop, color),
            3 => Meeple::new(cords, Type::Queen, color),
            4 => Meeple::new(cords, Type::King, color),
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
        self.walk_and_replace(frst, scnd);

        self.game_board[scnd.0] [scnd.1].as_mut().unwrap().move_counter += 1;
        self.logs.push((self.clicked_meeple,scnd));   
        self.shown_moves = Default::default();
        self.check_pawn_mutate(frst,scnd); 
        self.turn = opposite_color(self.turn);
        self.possible_moves = self.get_all_possible_moves(); 
    }  

    fn walk_and_replace(&mut self,frst: (usize,usize), scnd: (usize,usize)) {
        let frst_meeple = self.game_board[frst.0] [frst.1].as_mut().unwrap();
        
        //meeple
        frst_meeple.pos = scnd.clone();
        self.game_board[scnd.0] [scnd.1] = Some(*frst_meeple);

        //spot
        self.game_board[frst.0] [frst.1] = None;
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
            self.walk_and_replace((7,scnd.1), new_rook_pos); 
        } else {
            let new_rook_pos:(usize,usize) = (3,scnd.1);
            self.walk_and_replace((0,scnd.1), new_rook_pos);
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

    fn check_pawn_mutate(&mut self, frst_pos: (usize,usize), scnd_pos: (usize,usize)) {
        if let Some(pawn) = self.game_board[scnd_pos.0] [scnd_pos.1].as_mut() {
            if pawn.typ == Type::Pawn && ((pawn.color == Color::White && pawn.pos.1 == 0) || (pawn.color == Color::Black && pawn.pos.1 == 7)){
                pawn.typ = Type::Queen;
            }
        }
    }
    
    fn get_all_possible_moves(&mut self) -> [[Option<Vec<(usize,usize)>>;8];8] {
        let mut ret_vec:[[Option<Vec<(usize,usize)>>;8];8] = Default::default();
        let mut colores = get_meeples_from_color(self.game_board, self.turn);
        let king = colores.0.last().unwrap().pos;
        let mut can_move = false;
        for colored_meeple in colores.0 {
            if let Some(m) = self.check_meeple_moves_valid(colored_meeple,&colores.1,king) {
                ret_vec[colored_meeple.pos.0] [colored_meeple.pos.1] = Some(m);
                can_move = true;
            }
        }
        if !can_move {
            println!("jmd hat gewonnen");
        }
        ret_vec
    }
    

    fn check_meeple_moves_valid(&mut self, meeple: Meeple,check_color: &Vec<Meeple>,king: (usize,usize)) -> Option<Vec<(usize,usize)>>{
        let mut ret_vec:Vec<(usize,usize)> = Vec::new();
        for check_meep in meeple.show_moves(self.game_board, *self.logs.clone().last().unwrap()) {
            if let Some(pos) = self.check_the_future(meeple, check_meep,check_color,king) {
                ret_vec.push(pos);
            } 
        }
        
        if ret_vec.is_empty() {
            return None;
        }
        Some(ret_vec)
    }

    fn check_the_future(&mut self, current_meeple: Meeple,to_move: (usize,usize),check_color: &Vec<Meeple>, king: (usize,usize)) -> Option<(usize,usize)> {
        let king_king = if current_meeple.typ == Type::King {
            to_move.clone()
        } else {
            king
        };
        let mut check_colore = check_color.clone();
        if let Some(index) = check_colore.iter().position(|&x| x.pos == to_move) {
            check_colore.remove(index);
        }
        let from_pos = current_meeple.pos.clone();
        let to_pos = to_move.clone();

        let from = self.game_board[from_pos.0] [from_pos.1].clone();
        let to = self.game_board[to_pos.0] [to_pos.1].take();
        
        self.walk_and_replace(from_pos, to_pos);
        let can_hit = meeples_can_hit_meeple(&check_colore, king_king, self.game_board, (current_meeple.pos,to_move));
        self.game_board[from_pos.0] [from_pos.1] = from;
        self.game_board[to_pos.0] [to_pos.1] = to;

        if can_hit {
            return None;
        }
        Some(to_move)
    }
}


pub fn get_meeples_from_color(chess_board:[[Option<Meeple>;8];8],color_at_0: Color) -> (Vec<Meeple>,Vec<Meeple>) {
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

fn create_basic_possible_moves() -> [[Option<Vec<(usize,usize)>>;8];8] {
    let mut ret_vec:[[Option<Vec<(usize,usize)>>;8];8] = Default::default();
    for index in 0..8 {
        ret_vec[index] [6] = Some(vec![(index,5),(index,4)]);
    }
    ret_vec[1] [7] = Some(vec![(0,5),(2,5)]);
    ret_vec[6] [7] = Some(vec![(7,5),(5,5)]);
    ret_vec
}

fn meeples_can_hit_meeple(meeples: &Vec<Meeple>, meeple: (usize,usize), chess_board:[[Option<Meeple>;8];8], last_move: ((usize,usize),(usize,usize))) -> bool{
    for check_meeple in meeples {
        if check_meeple.show_moves(chess_board, last_move).contains(&meeple) {
            return true;
        }
    }
    false
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
