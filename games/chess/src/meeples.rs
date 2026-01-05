use std::vec;

#[derive(Copy, Clone, PartialEq,Debug)]
pub enum Color {
    White,Black,
} 

pub fn opposite_color(color: Color) -> Color {
    if color == Color::White {
        return Color::Black;
    }
    Color::White
}

#[derive(Copy, Clone, PartialEq,Debug)]
pub enum Type {
    Pawn,Knight,Bishop,Rook,Queen,King,
}

#[derive(Copy, Clone, PartialEq,Debug)]
///This struct has the Meeple in it with the position (the pos is also in the field) 
///and the type and the color of the meeple
pub struct Meeple {
    pub pos: (usize,usize),
    pub typ: Type,
    pub color: Color,
    pub move_counter: u128,
    pub value: f32,
}

impl Meeple {
    pub fn new(point: (usize,usize),meeple_type: Type, meeple_color: Color,meeple_value: f32) -> Meeple {
        Meeple { pos: point, typ: meeple_type, color: meeple_color, move_counter: 0, value: meeple_value}
    }

    ///This function returns a Vec<> with the positions where the Meeple can go to
    ///it runs seperate functions for each type of Meeple
    pub fn show_moves(&self,chess_board:&[[Option<Meeple>;8];8],last_move: &((usize,usize),(usize,usize))) -> Vec<(usize,usize)> {
        match self.typ {
            Type::Pawn => self.show_moves_pawn(chess_board,last_move),
            Type::Knight => self.show_moves_knight(&chess_board),
            Type::Bishop => self.show_moves_bishop(&chess_board),
            Type::Rook => self.show_moves_rook(&chess_board),
            Type::Queen => self.show_moves_queen(&chess_board),
            Type::King => self.show_moves_king(&chess_board,last_move),
        }
    }

    fn show_moves_pawn(&self,chess_board:&[[Option<Meeple>;8];8],last_move: &((usize,usize),(usize,usize))) -> Vec<(usize,usize)> {
        let mut possible_moves:Vec<(usize,usize)> = Vec::new();
        let pawn_move_add_vec = self.get_pawn_vec();
        let mut check_pos = self.pos_add(pawn_move_add_vec[0]);
        //no hit moves
        if self.pos_is_valid(check_pos) && self.pos_is_none(check_pos, chess_board){
            possible_moves.push(check_pos);
            check_pos = self.pos_add(pawn_move_add_vec[1]);
            if self.move_counter == 0 && self.pos_is_valid(check_pos) && self.pos_is_none(check_pos, chess_board){
                possible_moves.push(check_pos);
            }
        }
        //hit moves
        check_pos = self.pos_add(pawn_move_add_vec[2]);
        if self.pos_is_valid(check_pos) && self.pos_is_opposite_color(check_pos, chess_board) {
            possible_moves.push(check_pos);
        }

        check_pos = self.pos_add(pawn_move_add_vec[3]);
        if self.pos_is_valid(check_pos) && self.pos_is_opposite_color(check_pos, chess_board) {
            possible_moves.push(check_pos);
        }

        //en passent
        for add_vec in [(-1,0),(1,0)] {
            check_pos = self.pos_add(add_vec);
            if self.pos.1 == pawn_move_add_vec[4].0 as usize && self.pos_is_valid(check_pos) && self.pos_is_opposite_color(check_pos, &chess_board) {
                if let Some(meep) = get_meeple_at(chess_board, check_pos) {
                    if meep.typ == Type::Pawn && meep.move_counter == 1  && last_move.1 == check_pos{
                        possible_moves.push(self.pos_add((add_vec.0,pawn_move_add_vec[0].1)));
                    }
                }
            }
        }
        possible_moves
    }

    fn get_pawn_vec(&self) -> [(i8,i8);5]{
        match self.color {
            Color::White => [(0,-1),(0,-2),(1,-1),(-1,-1),(3,42)],
            Color::Black => [(0,1),(0,2),(1,1),(-1,1),(4,42)],
        }
    }

    fn show_moves_knight(&self,chess_board:&[[Option<Meeple>;8];8]) -> Vec<(usize,usize)> {
        let mut possible_moves:Vec<(usize,usize)> = Vec::new();
        let check_add_vec:[(i8,i8);8] = [(2,1),(2,-1),(-2,1),(-2,-1),(1,2),(-1,2),(1,-2),(-1,-2)];
    
        for add_pos in check_add_vec {
            let added_pos = self.pos_add(add_pos);
            if self.pos_is_valid(added_pos) && self.pos_is_opposite_color_or_none(added_pos, &chess_board) {
                possible_moves.push(added_pos);
            }
        }
        possible_moves
    }

    fn show_moves_bishop(&self,chess_board:&[[Option<Meeple>;8];8]) -> Vec<(usize,usize)> {
        let mut possible_moves:Vec<(usize,usize)> = Vec::new();
        let check_add_mult_vec:[(i8,i8);4] = [(1,1),(-1,-1),(1,-1),(-1,1)];

        for add_pos in check_add_mult_vec {
            let mut run_var:i8 = 0;
            loop{
                run_var += 1;
                let added_pos = self.pos_add((add_pos.0*run_var,add_pos.1*run_var));
                if self.pos_is_valid(added_pos) && !self.pos_is_same_color(added_pos, chess_board) {
                    possible_moves.push(added_pos);
                    if self.pos_is_opposite_color(added_pos, chess_board){    
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        possible_moves
    }

    fn show_moves_rook(&self,chess_board:&[[Option<Meeple>;8];8]) -> Vec<(usize,usize)> {
        let mut possible_moves:Vec<(usize,usize)> = Vec::new();
        let check_add_mult_vec:[(i8,i8);4] = [(1,0),(-1,0),(0,1),(0,-1)];

        for add_pos in check_add_mult_vec {
            let mut run_var = 0;
            loop {
                run_var += 1;
                let added_pos = self.pos_add((add_pos.0*run_var,add_pos.1*run_var));
                if self.pos_is_valid(added_pos) && !self.pos_is_same_color(added_pos, chess_board){
                    possible_moves.push(added_pos);
                    if self.pos_is_opposite_color(added_pos, chess_board){     
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        possible_moves
    }

    fn show_moves_queen(&self,chess_board:&[[Option<Meeple>;8];8]) -> Vec<(usize,usize)> {
        let mut possible_moves:Vec<(usize,usize)> = self.show_moves_rook(chess_board);
        possible_moves.append(&mut self.show_moves_bishop(chess_board));
        possible_moves
    }

    fn show_moves_king(&self,chess_board:&[[Option<Meeple>;8];8],last_move: &((usize,usize),(usize,usize))) -> Vec<(usize,usize)> {
        let mut possible_moves:Vec<(usize,usize)> = Vec::new();
        let check_add_vec:[(i8,i8);8] = [(-1,-1),(0,-1),(1,-1),(-1,0),(1,0),(-1,1),(0,1),(1,1)];
    
        for add_pos in check_add_vec {
            let added_pos = self.pos_add(add_pos);
            if self.pos_is_valid(added_pos) && self.pos_is_opposite_color_or_none(added_pos, chess_board) {
                possible_moves.push(added_pos);
            }
        }

        if self.move_counter == 0 && last_move != &((42,42),(42,42)) {
            possible_moves.append(&mut self.check_casteling_king_and_queen(chess_board));
        }

        possible_moves
    }
    fn check_casteling_king_and_queen(&self,chess_board:&[[Option<Meeple>;8];8]) -> Vec<(usize,usize)>{
        let mut possible_moves:Vec<(usize,usize)> = Vec::new();
        let y = self.pos.1;
    
        let mut check_vec_right:Option<Vec<(usize,usize)>> = None;
        if self.pos_is_none((5,y), chess_board) && self.pos_is_none((6,y), chess_board) {
            if let Some(rook) = get_meeple_at(chess_board, (7,y)) {
                if rook.typ == Type::Rook && rook.color == self.color && rook.move_counter == 0 {
                    check_vec_right = Some(vec![(4,y),(5,y),(6,y),(7,y)]);
                }
            }
        }

        if let Some(right_meeples) = check_vec_right {
            if self.check_casteling_chess(right_meeples, chess_board) {
                possible_moves.push((6,y));
            }
        } 

        let mut check_vec_left:Option<Vec<(usize,usize)>> = None;
        if self.pos_is_none((3,y), chess_board) && self.pos_is_none((2,y), chess_board) && self.pos_is_none((1,y), chess_board){
            if let Some(rook) = get_meeple_at(chess_board, (0,y)) {
                if rook.typ == Type::Rook && rook.color == self.color && rook.move_counter == 0{
                    check_vec_left = Some(vec![(4,y),(3,y),(2,y),(1,y),(0,y)]);
                }
            }
        }

        if let Some(left_meeples) = check_vec_left {
            if self.check_casteling_chess(left_meeples, chess_board) {
                possible_moves.push((2,y));
            }
        } 
        possible_moves
    }

    fn check_casteling_chess(&self,check_meeples: Vec<(usize,usize)>,chess_board:&[[Option<Meeple>;8];8]) -> bool {
        for row in chess_board {
            for check_option_meeple in row {
                if let Some(check_meeple) = check_option_meeple {
                    if check_meeple.color != self.color  {
                        if !self.check_casteling_chess_contains(check_meeples.clone(), check_meeple, chess_board) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    fn check_casteling_chess_contains(&self,casteling_check_list: Vec<(usize,usize)>, check_meeple: &Meeple, chess_board:&[[Option<Meeple>;8];8]) -> bool {
        !check_meeple.show_moves(chess_board,&((42,42),(42,42))).iter().any(|&item| casteling_check_list.contains(&item))
    }

    ///checks if the position is valid ((42,42) is unvalid)
    fn pos_is_valid(&self,check_pos: (usize,usize)) -> bool {
        if check_pos.0 == 42 {
            return false;
        }
        true 
    }

    ///adds the position to the postion of the meeple and returns it  
    fn pos_add(&self,(x_add,y_add):(i8,i8)) -> (usize,usize) {
        let x_sum = self.pos.0 as i8 + x_add;
        let y_sum = self.pos.1 as i8 + y_add;

        if !(0..=7).contains(&x_sum) || !(0..=7).contains(&y_sum) {
            return (42, 42);
        }
        (x_sum as usize, y_sum as usize)
    }

    ///returns if the meeple at check_pos is None
    fn pos_is_none(&self,check_pos: (usize,usize), chess_board:&[[Option<Meeple>;8];8]) -> bool {
        match get_meeple_at(chess_board,check_pos) {
            None => true,
            _ => false,
        }
    }

    ///returns if check_pos is the opposite color of the meeple
    fn pos_is_opposite_color(&self,check_pos: (usize,usize), chess_board:&[[Option<Meeple>;8];8]) -> bool {
        match get_meeple_at(chess_board,check_pos) {
            Some(meeple) => if meeple.color != self.color {true} else {false},
            _ => false,
        }
    }

    ///returns if check_pos is the color of the meeple
    fn pos_is_same_color(&self,check_pos: (usize,usize), chess_board:&[[Option<Meeple>;8];8]) -> bool {
        match get_meeple_at(chess_board,check_pos) {
            Some(meeple) => if meeple.color == self.color {true} else {false},
            _ => false,
        }
    }

    ///returns if the position is None or the opposite color of the meeple
    fn pos_is_opposite_color_or_none(&self,check_pos: (usize,usize), chess_board:&[[Option<Meeple>;8];8]) -> bool {
        match get_meeple_at(chess_board,check_pos) {
            Some(meeple) => if meeple.color != self.color {true} else {false},
            None => true,
        }
    }
}

///only to look up what meeple is where (not mutable)
pub fn get_meeple_at(chess_board:&[[Option<Meeple>;8];8],(x,y): (usize,usize)) -> Option<Meeple>{
    chess_board[x] [y]
}