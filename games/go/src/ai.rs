use crate::game::{Game, Stone};
use rand::prelude::{IndexedRandom, SliceRandom};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct MCTSStats {
    pub iterations: usize,
    #[allow(dead_code)]
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

pub fn get_best_move(game: &Game, _iterations: usize) -> (Option<(usize, usize)>, MCTSStats) {
    let time_budget = Duration::from_millis(1500);
    let start_time = Instant::now();

    let root_player = game.current_turn;
    let root = Rc::new(RefCell::new(MCTSNode::new(game.clone(), None, None)));

    let mut loops = 0;

    while start_time.elapsed() < time_budget {
        loops += 1;
        let mut node = root.clone();

        // 1. selection
        loop {
            let borrowed = node.borrow();
            if borrowed.is_terminal() || !borrowed.is_fully_expanded() {
                break;
            }

            if !borrowed.children.is_empty() {
                let parent_visits = borrowed.visits;
                let is_root_player = borrowed.state.current_turn == root_player;

                let next_node = borrowed
                    .children
                    .iter()
                    .max_by(|a, b| {
                        let a_val = a.borrow().uct_value(parent_visits, is_root_player);
                        let b_val = b.borrow().uct_value(parent_visits, is_root_player);
                        a_val.partial_cmp(&b_val).unwrap()
                    })
                    .cloned();

                drop(borrowed);
                if let Some(n) = next_node {
                    node = n;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // 2. EXPANSION
        if !node.borrow().is_terminal() {
            while !node.borrow().is_fully_expanded() {
                let move_option = node.borrow_mut().unexpanded_moves.pop();
                if let Some((mx, my)) = move_option {
                    let mut new_state = node.borrow().state.clone();
                    if new_state.place_stone(mx, my).is_ok() {
                        let new_node = Rc::new(RefCell::new(MCTSNode::new(
                            new_state,
                            Some((mx, my)),
                            Some(node.clone()),
                        )));
                        node.borrow_mut().children.push(new_node.clone());
                        node = new_node;
                        break;
                    }
                }
            }
        }

        // 3. ( Fast Atari)
        let mut sim_state = node.borrow().state.clone();

        sim_state.previous_states.clear();
        sim_state.history.clear();

        let mut rng = rand::rng();
        let mut moves_count = 0;

        while !sim_state.is_terminal() && moves_count < 60 {
            let mut moved = false;
            let c = sim_state.current_turn;
            let opp = c.other();

            //  1: Fast Kill
            let kill_moves = get_fast_atari_moves(&sim_state, opp);
            if let Some(&(kx, ky)) = kill_moves.first() {
                if sim_state.place_stone(kx, ky).is_ok() {
                    moved = true;
                }
            }

            //  2: Fast Save
            if !moved {
                let save_moves = get_fast_atari_moves(&sim_state, c);
                if let Some(&(sx, sy)) = save_moves.first() {
                    if sim_state.place_stone(sx, sy).is_ok() {
                        moved = true;
                    }
                }
            }

            //  3: Random
            if !moved {
                let empty = sim_state.get_empty_points();
                for _ in 0..5 {
                    if let Some(&(mx, my)) = empty.choose(&mut rng) {
                        if sim_state.place_stone(mx, my).is_ok() {
                            moved = true;
                            break;
                        }
                    }
                }
                if !moved {
                    sim_state.pass();
                }
            }
            moves_count += 1;
        }

        // 4. scoring
        let (b_score, w_score) = if sim_state.is_terminal() {
            sim_state.calculate_score()
        } else {
            sim_state.calculate_material_score()
        };

        let diff = if root_player == Stone::Black {
            b_score - w_score
        } else {
            w_score - b_score
        };
        let reward = score_sigmoid(diff);

        // 5. backprop.
        let mut backprop = Some(node);
        while let Some(n) = backprop {
            let mut b = n.borrow_mut();
            b.visits += 1;
            b.total_score += reward;
            backprop = b.parent.clone();
        }
    }

    let root_borrowed = root.borrow();
    let mut stats: Vec<_> = root_borrowed
        .children
        .iter()
        .map(|c| {
            let b = c.borrow();
            (
                b.move_from_parent.unwrap(),
                b.visits,
                b.total_score / b.visits as f32,
            )
        })
        .collect();

    stats.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n=== MCTS (Root {:?}) ===", root_player);
    println!("Time: 1.5s | Iterations: {}", loops);
    println!("Top 5 Moves:");
    println!("Move\t\tVisits\tScore");
    for (m, v, s) in stats.iter().take(5) {
        println!("{:?}\t{}\t{:.3}", m, v, s);
    }

    let best_move = stats.first().map(|s| s.0);

    let mcts_stats = MCTSStats {
        iterations: loops,
        root_player,
        top_moves: stats.into_iter().take(5).collect(),
    };

    (best_move, mcts_stats)
}
