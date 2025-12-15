use egui::Ui;
use game_core::Game;

mod meeples;
use crate::{draw::draw_board, meeples::{Color, Meeple, Type}};

mod draw;

pub struct ChessGame {
    state: String,
    game_board: [[Option<Meeple>;8];8],
    possible_moves: [[Option<Vec<(usize,usize)>>;8];8],
    kings: ((usize,usize),(usize,usize)),               //(white,black)
    shown_moves: Option<Vec<(usize,usize)>>,
    logs: Vec<String>,  
}

impl ChessGame {
    pub fn new() -> Self {
        let state_ = "Initial Chess State".to_string();
        let mut chess_board:[[Option<Meeple>;8];8] = Default::default();
        let possible_moves_ = Default::default();
        let kings_:((usize,usize),(usize,usize)) = ((4,7),(4,0));
        let logs_ :Vec<String> = Vec::new();
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

        Self {state: state_,game_board: chess_board, possible_moves: possible_moves_, kings: kings_, shown_moves: None,logs: logs_}
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

    pub fn show_moves(&mut self,(x,y):(usize,usize)) -> Option<Vec<(usize,usize)>>{
        let mut return_vec = None;
        if let Some(meeple) = self.game_board[x][y] {
            return_vec = Some(meeple.show_moves(self.game_board));
        }
        self.shown_moves = return_vec.clone();
        return_vec
    }
    
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
