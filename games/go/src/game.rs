#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stone {
    Black,
    White,
}

impl Stone {
    pub fn other(&self) -> Stone {
        match self {
            Stone::Black => Stone::White,
            Stone::White => Stone::Black,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    pub size: usize,
    pub grid: Vec<Option<Stone>>,
}

impl Board {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            grid: vec![None; size * size],
        }
    }
}

pub struct Game {
    pub board: Board,
    pub current_turn: Stone,
}

impl Game {
    pub fn new(size: usize) -> Self {
        Self {
            board: Board::new(size),
            current_turn: Stone::Black,
        }
    }
    pub fn place_stone() {
        //todo
    }
    pub fn pass() {
        //todo
    }
    pub fn get_valid_moves() {
        //todo
    }
    pub fn get_winner() {
        //todo
    }
}
