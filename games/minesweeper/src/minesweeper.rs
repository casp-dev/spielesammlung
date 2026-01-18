// contains the and game logic for minesweeper
/*
Rules: (via: https://en.wikipedia.org/wiki/Minesweeper_(video_game))

Minesweeper is a puzzle video game. In the game, mines (that resemble naval mines in the classic theme) are scattered throughout a board, which is divided into cells. Cells have three states: unopened, opened and flagged. An unopened cell is blank and clickable, while an opened cell is exposed. Flagged cells are those marked by the player to indicate a potential mine location.

A player selects a cell to open it. If a player opens a mined cell, the game ends. Otherwise, the opened cell displays either a number, indicating the number of mines vertically, horizontally or diagonally adjacent to it, or a blank tile (or "0"), and all adjacent non-mined cells will automatically be opened. Players can also flag a cell, visualized by a flag being put on the location, to denote that they believe a mine to be in that place. Flagged cells are still considered unopened, and a player can click on them to open them.

In some versions of the game when the number of adjacent mines is equal to the number of adjacent flagged cells, all adjacent non-flagged unopened cells will be opened, a process known as chording.
*/

// TODO: bombe verschieben
// BUG: wenn flage im Bereich ist der mit Flood fill geöffnet wird ist die flagge WEG
//...

#![allow(unused_variables)]
#![allow(dead_code)]

use rand::Rng;

#[derive(Clone, PartialEq, Debug, Eq, Copy)]
pub enum CellState {
    Unopened,
    Opened,
    Flagged,
}

#[derive(Clone, PartialEq, Debug, Eq, Copy)]
pub enum CellContent {
    Mine,
    Blank,
    Number(u8), // For Neighboring mines 0-8
}

pub type X = usize;
pub type Y = usize;

#[derive(Clone, PartialEq, Debug, Eq, Copy)]
pub enum ActionKind {
    Flag(X, Y), // No "unflag" just use Flag as a switch
    Open(X, Y),
}

#[derive(Clone, PartialEq, Debug, Eq, Copy)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

#[derive(Clone, PartialEq, Debug, Eq, Copy)]
pub struct Cell {
    pub cell_state: CellState,
    pub cell_content: CellContent,
}

#[allow(unused_parens)]
pub fn is_on_board(game: &Game, action_kind: &ActionKind) -> bool {
    let (x, y) = match action_kind {
        ActionKind::Open(x, y) => (*x, *y),
        ActionKind::Flag(x, y) => (*x, *y),
    };
    if (x < game.board[0].len() && y < game.board.len()) {
        return true;
    }
    return false;
}

#[allow(unused_parens)]
pub fn flag_allowed(game: &Game, action_kind: &ActionKind) -> bool {
    if !(is_on_board(game, action_kind)) {
        return false;
    }
    if let ActionKind::Flag(x, y) = action_kind {
        if (game.board[*y][*x].cell_state == CellState::Unopened
            || (game.board[*y][*x].cell_state == CellState::Flagged))
        {
            return true;
        }
    }
    return false;
}

#[allow(unused_parens)]
pub fn open_allowed(game: &Game, action_kind: &ActionKind) -> bool {
    if !(is_on_board(game, action_kind)) {
        return false;
    }
    if let ActionKind::Open(x, y) = action_kind {
        if (game.board[*y][*x].cell_state == CellState::Unopened) {
            return true;
        }
    }
    return false;
}

#[allow(unused_parens)]
pub fn is_action_allowed(game: &Game, action_kind: &ActionKind) -> bool {
    if (game.game_over == true) {
        return false;
    }
    if (game.game_won == true) {
        return false;
    }
    match action_kind {
        ActionKind::Open(_, _) => open_allowed(game, action_kind),
        ActionKind::Flag(_, _) => flag_allowed(game, action_kind),
    }
}

#[allow(unused_parens)]
pub fn flag(game: &mut Game, action_kind: &ActionKind) {
    if let ActionKind::Flag(x, y) = action_kind {
        if (game.board[*y][*x].cell_state == CellState::Unopened) {
            if (game.flag_count > 0) {
                game.board[*y][*x].cell_state = CellState::Flagged;
                game.flag_count = game.flag_count - 1;
            }
            return;
        }
        if (game.board[*y][*x].cell_state == CellState::Flagged) {
            game.board[*y][*x].cell_state = CellState::Unopened;
            game.flag_count = game.flag_count + 1;
            return;
        }
    }
}

#[allow(unused_parens)]
pub fn open(game: &mut Game, action_kind: &ActionKind) {
    if let ActionKind::Open(x, y) = action_kind {
        if game.board[*y][*x].cell_state == CellState::Opened {
            return;
        }

        if let CellContent::Number(_) = game.board[*y][*x].cell_content {
            game.board[*y][*x].cell_state = CellState::Opened;
            game.opened_counter += 1;
        }

        if (game.board[*y][*x].cell_content == CellContent::Blank) {
            game.board[*y][*x].cell_state = CellState::Opened;
            game.opened_counter += 1;
            flood_fill(game, &y, &x);
        }

        if (game.board[*y][*x].cell_content == CellContent::Mine) {
            game.board[*y][*x].cell_state = CellState::Opened;
            game.opened_counter += 1;
            boom(game);
        }
    }
}

#[allow(unused_parens)]
pub fn flood_fill(game: &mut Game, y: &usize, x: &usize) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = *x as isize + dx;
            let ny = *y as isize + dy;

            if nx >= 0 && ny >= 0 {
                let nx = nx as usize;
                let ny = ny as usize;

                if ny < game.board.len() && nx < game.board[0].len() {
                    if (game.board[ny][nx].cell_state == CellState::Opened) {
                        continue;
                    }

                    if let CellContent::Number(_) = game.board[ny][nx].cell_content {
                        open(game, &ActionKind::Open((nx), (ny)));
                    }
                    if let CellContent::Blank = game.board[ny][nx].cell_content {
                        open(game, &ActionKind::Open((nx), (ny)));
                    }
                }
            }
        }
    }
}

#[allow(unused_parens)]
pub fn reveal_bombs(game: &mut Game) {
    let height = game.board.len();
    let width = game.board[0].len();

    for y in 0..height {
        for x in 0..width {
            if (game.board[y][x].cell_content == CellContent::Mine) {
                game.board[y][x].cell_state = CellState::Opened;
            }
        }
    }
}

pub fn boom(game: &mut Game) -> bool {
    game.game_over = true;
    game.game_won = false;

    reveal_bombs(game);

    return game.game_over;
}

#[allow(unused_parens)]
pub fn adjacent_mines(game: &Game, y: &usize, x: &usize) -> u8 {
    let mut count = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = *x as isize + dx;
            let ny = *y as isize + dy;

            if nx >= 0 && ny >= 0 {
                let nx = nx as usize;
                let ny = ny as usize;

                if ny < game.board.len() && nx < game.board[0].len() {
                    if let CellContent::Mine = game.board[ny][nx].cell_content {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

#[allow(unused_parens)]
pub fn set_blancs_and_numbers(game: &mut Game) {
    let height = game.board.len();
    let width = game.board[0].len();
    for y in 0..height {
        for x in 0..width {
            if let CellContent::Mine = game.board[y][x].cell_content {
                continue;
            }
            let mines_nearby = adjacent_mines(game, &y, &x);
            if (mines_nearby == 0) {
                game.board[y][x].cell_content = CellContent::Blank;
            } else {
                game.board[y][x].cell_content = CellContent::Number(mines_nearby);
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Game {
    pub board: Vec<Vec<Cell>>,
    pub opened_counter: usize,
    pub mine_count: usize,
    pub flag_count: usize,
    pub game_over: bool,
    pub game_won: bool,
}

pub trait Minesweeper {
    fn new_game(difficulty: Difficulty) -> Self;
    fn apply_action(&mut self, action_kind: ActionKind) -> Result<(), &'static str>;
    fn winner(&mut self) -> bool;
}

#[allow(unused_parens)]
impl Minesweeper for Game {
    fn new_game(difficulty: Difficulty) -> Self {
        let default_cell = Cell {
            cell_content: CellContent::Blank,
            cell_state: CellState::Unopened,
        };
        let mine_cell = Cell {
            cell_content: CellContent::Mine,
            cell_state: CellState::Unopened,
        };
        let (mut board, mine_count) = match difficulty {
            Difficulty::Easy => {
                (vec![vec![default_cell; 9]; 9], 10) // 9x9, 10 Mines
            }
            Difficulty::Medium => {
                (vec![vec![default_cell; 16]; 16], 40) // 16x16, 40 Mines
            }
            Difficulty::Hard => {
                (vec![vec![default_cell; 22]; 22], 80) // 22x22, 80 Mines
            }
            Difficulty::Expert => {
                (vec![vec![default_cell; 30]; 16], 99) // 30x16, 99 Mines
            }
        };
        let column = board[0].len(); // X
        let rows = board.len(); // Y

        let cell_count = column * rows;
        let mut no_dupes: Vec<usize> = Vec::new();
        while (no_dupes.len() < mine_count) {
            let random_number = rand::thread_rng().gen_range(0..cell_count);
            if !(no_dupes.contains(&random_number)) {
                no_dupes.push(random_number);
                let y = random_number / column;
                let x = random_number % column;
                board[y][x].cell_content = CellContent::Mine;
            }
        }
        let mut game = Game {
            board: board,
            opened_counter: 0,
            mine_count: mine_count,
            flag_count: mine_count,
            game_over: false,
            game_won: false,
        };

        set_blancs_and_numbers(&mut game);

        return game;
    }

    fn apply_action(&mut self, action_kind: ActionKind) -> Result<(), &'static str> {
        if !(is_action_allowed(&self, &action_kind)) {
            return Err("Action not Allowed");
        }

        match action_kind {
            ActionKind::Open(_, _) => {
                open(self, &action_kind);
                self.winner();
                if (self.game_won == true) {
                    reveal_bombs(self);
                }
                Ok(())
            }
            ActionKind::Flag(_, _) => {
                flag(self, &action_kind);
                self.winner();
                if (self.game_won == true) {
                    reveal_bombs(self);
                }
                Ok(())
            }
        }
    }

    fn winner(&mut self) -> bool {
        if (self.game_over == true) {
            return false;
        }

        let total_cells = self.board.len() * self.board[0].len();
        let safe_cells = total_cells - self.mine_count;

        if (self.opened_counter == safe_cells) {
            self.game_won = true;
            return true;
        }
        return false;
    }
}
