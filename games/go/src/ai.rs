use crate::game::{Game, Stone};
use rand::prelude::SliceRandom;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct MCTSStats {
    pub iterations: usize,
    pub root_player: Stone,
    pub top_moves: Vec<((usize, usize), u32, f32)>, // (Move, Visits, Score)
}

#[derive(Clone)]
pub struct MCTSNode {
    pub state: Game,
    pub move_from_parent: Option<(usize, usize)>,
    pub children: Vec<Rc<RefCell<MCTSNode>>>,
    pub visits: u32,
    pub total_score: f32,
    pub unexpanded_moves: Vec<(usize, usize)>,
    pub parent: Option<Rc<RefCell<MCTSNode>>>,
}

impl MCTSNode {
    pub fn new(
        state: Game,
        move_from_parent: Option<(usize, usize)>,
        parent: Option<Rc<RefCell<MCTSNode>>>,
    ) -> Self {
        let mut unexpanded_moves = state.get_empty_points();
        let mut rng = rand::rng();
        unexpanded_moves.shuffle(&mut rng);

        Self {
            state,
            move_from_parent,
            children: Vec::new(),
            visits: 0,
            total_score: 0.0,
            unexpanded_moves,
            parent,
        }
    }

    pub fn is_fully_expanded(&self) -> bool {
        self.unexpanded_moves.is_empty()
    }

    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }
}
