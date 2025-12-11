// contains the  and game logic for minesweeper

/*
Rules: (via: https://en.wikipedia.org/wiki/Minesweeper_(video_game))

Minesweeper is a puzzle video game. In the game, mines (that resemble naval mines in the classic theme) are scattered throughout a board, which is divided into cells. Cells have three states: unopened, opened and flagged. An unopened cell is blank and clickable, while an opened cell is exposed. Flagged cells are those marked by the player to indicate a potential mine location.

A player selects a cell to open it. If a player opens a mined cell, the game ends. Otherwise, the opened cell displays either a number, indicating the number of mines vertically, horizontally or diagonally adjacent to it, or a blank tile (or "0"), and all adjacent non-mined cells will automatically be opened. Players can also flag a cell, visualized by a flag being put on the location, to denote that they believe a mine to be in that place. Flagged cells are still considered unopened, and a player can click on them to open them.

In some versions of the game when the number of adjacent mines is equal to the number of adjacent flagged cells, all adjacent non-flagged unopened cells will be opened, a process known as chording.
*/

#![allow(unused_variables)]
#![allow(dead_code)]

use rand::Rng; // later for mine placement

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

#[derive(Clone, PartialEq, Debug, Eq, Copy)] // For Future: x,y instead of CellNumber
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
    cell_state: CellState,
    cell_content: CellContent,
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Game {
    board: Vec<Vec<Cell>>,
    action_counter: usize, // future win condition
    mine_count: usize,
    flag_count: usize, // TODO: if flag_count reaches 0 all unflagged Cells will be revealed. You have as many flags as there are mines on the board
    game_over: bool,
    game_won: bool, // unused
}

pub trait Minesweeper {
    fn new_game(difficulty: Difficulty) -> Self;
    fn apply_action(&mut self, action_kind: ActionKind) -> Result<(), &'static str>;
    fn winner(&self) -> bool;
    // TODO: game lock in case of boom!....
}