use std::collections::HashSet;

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

    pub fn set(&mut self, x: usize, y: usize, stone: Option<Stone>) {
        if x < self.size && y < self.size {
            self.grid[y * self.size + x] = stone;
        }
    }

    pub fn get_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        if x > 0 {
            neighbors.push((x - 1, y));
        }
        if x < self.size - 1 {
            neighbors.push((x + 1, y));
        }
        if y > 0 {
            neighbors.push((x, y - 1));
        }
        if y < self.size - 1 {
            neighbors.push((x, y + 1));
        }
        neighbors
    }

    pub fn get_group(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let color = match self.get(x, y) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut group = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = vec![(x, y)];

        while let Some((cx, cy)) = stack.pop() {
            if visited.contains(&(cx, cy)) {
                continue;
            }
            visited.insert((cx, cy));
            group.push((cx, cy));

            for (nx, ny) in self.get_neighbors(cx, cy) {
                if let Some(n_color) = self.get(nx, ny) {
                    if n_color == color && !visited.contains(&(nx, ny)) {
                        stack.push((nx, ny));
                    }
                }
            }
        }
        group
    }

    pub fn count_liberties(&self, x: usize, y: usize) -> usize {
        let group = self.get_group(x, y);
        let mut liberties = HashSet::new();

        for (gx, gy) in group {
            for (nx, ny) in self.get_neighbors(gx, gy) {
                if self.get(nx, ny).is_none() {
                    liberties.insert((nx, ny));
                }
            }
        }
        liberties.len()
    }
}

#[derive(Clone)]
pub struct Game {
    pub board: Board,
    pub current_turn: Stone,
    pub previous_states: HashSet<Vec<Option<Stone>>>,
    pub consecutive_passes: usize,
    pub last_move: Option<(usize, usize)>,
    pub captured_black: usize,
    pub captured_white: usize,
    pub game_over: bool,
    pub history: Vec<(usize, usize)>,
}

impl Game {
    pub fn new(size: usize) -> Self {
        Self {
            board: Board::new(size),
            current_turn: Stone::Black,
            previous_states: HashSet::new(),
            consecutive_passes: 0,
            last_move: None,
            captured_black: 0,
            captured_white: 0,
            game_over: false,
            history: Vec::new(),
        }
    }
    pub fn place_stone(&mut self, x: usize, y: usize) -> Result<(), String> {
        if self.game_over {
            return Err("Spiel ist zu Ende".to_string());
        }
        if self.board.get(x, y).is_some() {
            return Err("Punkt ist nicht frei".to_string());
        }

        let mut next_board = self.board.clone();
        next_board.set(x, y, Some(self.current_turn));

        // Check captures
        let neighbors = next_board.get_neighbors(x, y);
        let mut captured_stones = Vec::new();

        for (nx, ny) in neighbors {
            if let Some(s) = next_board.get(nx, ny) {
                if s == self.current_turn.other() && next_board.count_liberties(nx, ny) == 0 {
                    let group = next_board.get_group(nx, ny);
                    captured_stones.extend(group);
                }
            }
        }

        // Remove captured stones
        for &(cx, cy) in &captured_stones {
            next_board.set(cx, cy, None);
        }

        // Check suicide
        if next_board.count_liberties(x, y) == 0 {
            return Err("Selbstmordzug".to_string());
        }

        // Check Ko (simple repetition)
        if self.previous_states.contains(&next_board.grid) {
            return Err("Ko Regelverstoß".to_string());
        }

        // Apply move
        self.board = next_board;
        if self.current_turn == Stone::Black {
            self.captured_white += captured_stones.len();
        } else {
            self.captured_black += captured_stones.len();
        }

        self.previous_states.insert(self.board.grid.clone());
        self.current_turn = self.current_turn.other();
        self.consecutive_passes = 0;
        self.last_move = Some((x, y));
        self.history.push((x, y));

        Ok(())
    }

    pub fn pass(&mut self) {
        self.previous_states.insert(self.board.grid.clone());
        self.consecutive_passes += 1;
        self.current_turn = self.current_turn.other();
        self.last_move = None;

        if self.consecutive_passes >= 2 {
            self.game_over = true;
        }
    }

    pub fn get_empty_points(&self) -> Vec<(usize, usize)> {
        let mut points = Vec::new();
        for y in 0..self.board.size {
            for x in 0..self.board.size {
                if self.board.get(x, y).is_none() {
                    points.push((x, y));
                }
            }
        }
        points
    }

    pub fn calculate_score(&self) -> (f32, f32) {
        let size = self.board.size;
        let mut black_score = self.captured_black as f32;
        let mut white_score = self.captured_white as f32 + 6.5; // Komi

        for stone in &self.board.grid {
            match stone {
                Some(Stone::Black) => black_score += 1.0,
                Some(Stone::White) => white_score += 1.0,
                None => {}
            }
        }

        // gebiet
        let mut visited = vec![false; size * size];

        for y in 0..size {
            for x in 0..size {
                let idx = y * size + x;
                if visited[idx] || self.board.get(x, y).is_some() {
                    continue;
                }

                // flood-fill
                let mut region = Vec::new();
                let mut stack = vec![(x, y)];
                let mut borders_black = false;
                let mut borders_white = false;

                while let Some((cx, cy)) = stack.pop() {
                    let ci = cy * size + cx;
                    if visited[ci] {
                        continue;
                    }
                    visited[ci] = true;
                    region.push((cx, cy));

                    for (nx, ny) in self.board.get_neighbors(cx, cy) {
                        let ni = ny * size + nx;
                        match self.board.get(nx, ny) {
                            None => {
                                if !visited[ni] {
                                    stack.push((nx, ny));
                                }
                            }
                            Some(Stone::Black) => borders_black = true,
                            Some(Stone::White) => borders_white = true,
                        }
                    }
                }

                if borders_black && !borders_white {
                    black_score += region.len() as f32;
                } else if borders_white && !borders_black {
                    white_score += region.len() as f32;
                }
            }
        }

        (black_score, white_score)
    }

    pub fn is_terminal(&self) -> bool {
        self.game_over || self.consecutive_passes >= 2
    }

    pub fn calculate_material_score(&self) -> (f32, f32) {
        let mut black_score = 0.0;
        let mut white_score = 6.5; // Komi

        for stone in &self.board.grid {
            match stone {
                Some(Stone::Black) => black_score += 1.0,
                Some(Stone::White) => white_score += 1.0,
                None => {}
            }
        }

        black_score += self.captured_white as f32;
        white_score += self.captured_black as f32;

        (black_score, white_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture() {
        let mut game = Game::new(9);
        // B
        game.place_stone(0, 1).unwrap();
        // W
        game.place_stone(0, 0).unwrap();
        // B
        game.place_stone(1, 0).unwrap();

        // W bei 0,0
        assert_eq!(game.board.get(0, 0), None);
        assert_eq!(game.captured_white, 1);
    }

    #[test]
    fn test_suicide_rule() {
        let mut game = Game::new(9);

        // B
        game.place_stone(0, 1).unwrap();
        // W
        game.place_stone(1, 1).unwrap();
        // B
        game.place_stone(1, 0).unwrap();

        let result = game.place_stone(0, 0);
        assert!(result.is_err());
        assert_eq!(result.err(), Some("Selbstmordzug".to_string()));
    }

    #[test]
    fn test_ko_rule() {
        // .  . B .
        // B W . B
        // . . B .

        let mut game = Game::new(5);
        let params = [
            (2, 1), // B
            (3, 1), // W
            (1, 2), // B
            (4, 2), // W
            (2, 3), // B
            (3, 3), // W
        ];

        for (x, y) in params {
            game.place_stone(x, y).unwrap();
        }
    }

    #[test]
    fn test_game_over() {
        let mut game = Game::new(9);
        assert!(!game.game_over);
        game.pass();
        assert!(!game.game_over);
        game.pass();
        assert!(game.game_over);
    }
}
