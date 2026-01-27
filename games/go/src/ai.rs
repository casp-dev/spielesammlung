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
        let mut unexpanded_moves = get_sensible_moves(&state);
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

    pub fn uct_value(&self, parent_visits: u32, is_root_player: bool) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let exploration_constant = 0.8;

        let mean_score = self.total_score / self.visits as f32;
        let exploitation = if is_root_player {
            mean_score
        } else {
            1.0 - mean_score
        };
        let exploration =
            exploration_constant * ((parent_visits as f32).ln() / self.visits as f32).sqrt();
        exploitation + exploration
    }
}

fn score_sigmoid(diff: f32) -> f32 {
    1.0 / (1.0 + (-diff * 0.2).exp())
}

fn get_fast_atari_moves(game: &Game, target_color: Stone) -> Vec<(usize, usize)> {
    let size = game.board.size;
    let mut visited = vec![false; size * size];
    let mut moves = Vec::new();
    let mut stack = Vec::with_capacity(size * size);

    for y in 0..size {
        for x in 0..size {
            let idx = y * size + x;

            if !visited[idx] {
                if let Some(s) = game.board.grid[idx] {
                    if s == target_color {
                        stack.clear();
                        stack.push((x, y));
                        visited[idx] = true;

                        let mut liberties = Vec::new();
                        let mut group_has_liberties = false;

                        let mut head = 0;
                        while head < stack.len() {
                            let (cx, cy) = stack[head];
                            head += 1;

                            let neighbors = [
                                if cx > 0 { Some((cx - 1, cy)) } else { None },
                                if cx < size - 1 {
                                    Some((cx + 1, cy))
                                } else {
                                    None
                                },
                                if cy > 0 { Some((cx, cy - 1)) } else { None },
                                if cy < size - 1 {
                                    Some((cx, cy + 1))
                                } else {
                                    None
                                },
                            ];

                            for n in neighbors.iter().flatten() {
                                let n_idx = n.1 * size + n.0;
                                match game.board.grid[n_idx] {
                                    None => {
                                        if !liberties.contains(n) {
                                            liberties.push(*n);
                                        }
                                        if liberties.len() > 1 {
                                            group_has_liberties = true;
                                        }
                                    }
                                    Some(stone) if stone == target_color => {
                                        if !visited[n_idx] {
                                            visited[n_idx] = true;
                                            stack.push(*n);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if !group_has_liberties && liberties.len() == 1 {
                            moves.push(liberties[0]);
                        }
                    }
                }
            }
        }
    }
    moves
}

fn get_sensible_moves(game: &Game) -> Vec<(usize, usize)> {
    let size = game.board.size;
    let current_turn = game.current_turn;
    let opponent = current_turn.other();
    let mut moves = Vec::new();

    let capture_moves = get_fast_atari_moves(game, opponent);
    let save_moves = get_fast_atari_moves(game, current_turn);

    for m in capture_moves {
        if !moves.contains(&m) {
            moves.push(m);
        }
    }
    for m in save_moves {
        if !moves.contains(&m) {
            moves.push(m);
        }
    }

    let stone_count = game.board.grid.iter().filter(|s| s.is_some()).count();
    if stone_count < 8 {
        let stars = vec![
            (2, 2),
            (6, 2),
            (2, 6),
            (6, 6),
            (4, 4),
            (3, 3),
            (15, 3),
            (3, 15),
            (15, 15),
        ];
        for m in stars {
            if m.0 < size && m.1 < size && !moves.contains(&m) && game.board.get(m.0, m.1).is_none()
            {
                moves.push(m);
            }
        }
        if !moves.is_empty() {
            return moves;
        }
    }

    let empty_points = game.get_empty_points();
    let mut candidates = Vec::new();

    for (x, y) in empty_points {
        if moves.contains(&(x, y)) {
            continue;
        }

        let mut score = 0;

        for (nx, ny) in game.board.get_neighbors(x, y) {
            if let Some(s) = game.board.get(nx, ny) {
                if s == opponent {
                    score += 50;
                } else {
                    score += 20;
                }
            }
        }

        if let Some((lx, ly)) = game.last_move {
            let dist = (x as i32 - lx as i32).abs() + (y as i32 - ly as i32).abs();
            if dist <= 2 {
                score += 30;
            }
        }

        if score > 0 {
            candidates.push(((x, y), score));
        }
    }

    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    for (m, _) in candidates.into_iter().take(12) {
        moves.push(m);
    }

    if moves.is_empty() {
        let all = game.get_empty_points();
        return all.into_iter().take(5).collect();
    }

    moves
}
