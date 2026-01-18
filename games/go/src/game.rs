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

    pub fn get(&self, x: usize, y: usize) -> Option<Stone> {
        if x < self.size && y < self.size {
            self.grid[y * self.size + x]
        } else {
            None
        }
    }
}

pub struct Game {
    pub board: Board,
    pub current_turn: Stone,
    pub captured_black: usize,
    pub captured_white: usize,
    pub game_over: bool,
}

impl Game {
    pub fn new(size: usize) -> Self {
        Self {
            board: Board::new(size),
            current_turn: Stone::Black,
            captured_black: 0,
            captured_white: 0,
            game_over: false,
        }
    }
    pub fn place_stone(&mut self, _x: usize, _y: usize) -> Result<(), String> {
        //todo: implement logic
        // For now just toggle turn
        self.current_turn = self.current_turn.other();
        Ok(())
    }
    pub fn pass(&mut self) {
        self.current_turn = self.current_turn.other();
    }
    pub fn get_valid_moves(&self) {
        //todo
    }
    pub fn calculate_score(&self) -> (f32, f32) {
        //todo
        (0.0, 0.0)
    }
}
