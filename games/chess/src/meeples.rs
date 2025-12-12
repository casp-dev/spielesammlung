#[derive(Copy, Clone, PartialEq,Debug)]
pub enum Color {
    White,Black,
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
    move_counter: u128,
}

impl Meeple {
    pub fn new(point: (usize,usize),meeple_type: Type, meeple_color: Color) -> Meeple {
        Meeple { pos: point, typ: meeple_type, color: meeple_color, move_counter: 0}
    }
}

pub fn get_meeple_at(chess_board:[[Option<Meeple>;8];8],(x,y): (usize,usize)) -> Option<Meeple>{
    chess_board[x] [y]
}