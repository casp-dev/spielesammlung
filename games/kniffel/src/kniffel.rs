use crate::bot::*;
use rand::Rng;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dice {
    pub eyes: u8,
    pub is_locked: bool,
}

impl Ord for Dice {
    fn cmp(&self, other: &Self) -> Ordering {
        let dice1 = self.eyes;
        let dice2 = other.eyes;
        if dice1 > dice2 {
            return Ordering::Greater;
        } else if dice2 > dice1 {
            return Ordering::Less;
        }
        Ordering::Equal
    }
}

impl PartialOrd for Dice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Dice {
    fn new(eyes: u8) -> Self {
        Dice {
            eyes,
            is_locked: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointTable {
    pub points_thrown: [Option<u8>; 13], //also die Punkte bei einsern, zweiern etc
    pub total_points: [u8; 4],           //Zwischensumme oben, oben mit bonus, unten, gesamt
}

pub type DiceThrow = [Dice; 5];

#[derive(Clone, PartialEq, Debug)]
pub enum PlayerKind {
    Human,
    Bot,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub point_table: PointTable,
    pub number_of_throws: u8,
    pub dice_throw: DiceThrow,
    pub kind: PlayerKind,
}

impl PointTable {
    fn new() -> Self {
        PointTable {
            points_thrown: [None; 13],
            total_points: [0; 4],
        }
    }
}

impl Player {
    pub fn human() -> Self {
        Player {
            point_table: PointTable::new(),
            number_of_throws: 0,
            dice_throw: [Dice::new(0); 5],
            kind: PlayerKind::Human,
        }
    }

    pub fn bot() -> Self {
        Player {
            kind: PlayerKind::Bot,
            ..Player::human()
        }
    }

    pub fn is_bot(&self) -> bool {
        self.kind == PlayerKind::Bot
    }

    pub fn calculate_empty_categories(&self) -> Vec<usize> {
        let mut empty_categories = Vec::new();
        for category in 0..13 {
            if self.point_table.points_thrown[category].is_none() {
                empty_categories.push(category);
            }
        }
        empty_categories
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Game {
    pub all_players: Vec<Player>,
    pub current_player: Player,
    pub current_player_index: usize,
    pub number_of_players: usize,
}

pub trait YahtzeeGame {
    // new Game with a certain number of Players
    #[allow(clippy::new_ret_no_self)]
    fn new(players_human: usize, players_bot: usize) -> Result<Game, &'static str>;

    fn winner(&self) -> Result<(Vec<usize>, usize), &'static str>;
}

impl YahtzeeGame for Game {
    fn new(players_human: usize, players_bot: usize) -> Result<Game, &'static str> {
        let number = players_human + players_bot;
        if !(2..=4).contains(&number) {
            return Err("Spieleranzahl wird nicht unterstützt. Nur 2-4 Spieler möglich");
        }

        let mut all_players = Vec::with_capacity(number);
        for _ in 0..players_human {
            all_players.push(Player::human());
        }
        for _ in 0..players_bot {
            all_players.push(Player::bot());
        }

        let current_player = all_players[0].clone();
        Ok(Self {
            all_players,
            current_player,
            current_player_index: 0,
            number_of_players: number,
        })
    }

    // return: (player_index, total_points)
    fn winner(&self) -> Result<(Vec<usize>, usize), &'static str> {
        for _ in 0..self.number_of_players {
            if !point_table_full(self) {
                return Err("Noch kein Gewinner");
            }
        }
        let mut max_points = 0;
        let mut winner_index = Vec::new();
        for index_player in 0..self.number_of_players {
            if self.all_players[index_player].point_table.total_points[3] > max_points {
                max_points = self.all_players[index_player].point_table.total_points[3];
                winner_index.push(index_player);
            }
        }
        if winner_index[0] < self.number_of_players - 1 {
            for index_player in winner_index[0]..self.number_of_players {
                if self.all_players[index_player].point_table.total_points[3] == max_points {
                    winner_index.push(index_player);
                }
            }
        }

        Ok((winner_index, max_points.into()))
    }
}

pub fn next_player(game: &mut Game) {
    game.current_player_index = (game.current_player_index + 1) % game.number_of_players;
    game.current_player = game.all_players[game.current_player_index].clone();
    game.current_player.number_of_throws = 0;
}

pub fn throw_dice(game: &mut Game) {
    if game.current_player.number_of_throws > 2 {
        game.current_player.number_of_throws = 0;
    } else {
        game.current_player.number_of_throws += 1;
        for current_dice in 0..5 {
            if !game.current_player.dice_throw[current_dice].is_locked {
                game.current_player.dice_throw[current_dice].eyes =
                    rand::thread_rng().gen_range(1..=6);
            }
        }
        // Synchronisiere current_player mit all_players
        game.all_players[game.current_player_index] = game.current_player.clone();
    }
}

pub fn change_blocked_status_dice(game: &mut Game, dice_index: usize) -> &Game {
    if game.current_player.dice_throw[dice_index].is_locked {
        game.current_player.dice_throw[dice_index].is_locked = false;
        return game;
    }
    game.current_player.dice_throw[dice_index].is_locked = true;
    game
}

pub fn add_dice_point_table(game: &mut Game, category: usize) -> &Game {
    let selected_category = game.current_player.point_table.points_thrown[category];
    if selected_category.is_none() {
        game.current_player.point_table.points_thrown[category] =
            Some(calculate_points(category, game.current_player.dice_throw));
        game.current_player.dice_throw = [Dice::new(0); 5];
        game.current_player.point_table =
            update_totals_point_table(game.current_player.point_table);
        game.all_players[game.current_player_index] = game.current_player.clone();
        game
    } else {
        game
    }
}

pub fn calculate_points(category: usize, dice_throw: DiceThrow) -> u8 {
    let mut sum = 0;
    let u8_category = category as u8;
    match u8_category {
        0..=5 => {
            for dice in dice_throw.iter() {
                if dice.eyes == (u8_category + 1) {
                    sum += u8_category + 1;
                }
            }
            sum
        }
        6 => {
            if three_same(dice_throw) {
                for dice in dice_throw {
                    sum += dice.eyes;
                }
            }
            sum
        }
        7 => {
            if four_same(dice_throw) {
                for dice in dice_throw {
                    sum += dice.eyes;
                }
            }
            sum
        }
        8 => {
            if full_house(dice_throw) {
                sum += 25;
            }
            sum
        }
        9 => {
            if small_straight(dice_throw) {
                //kleine Straße
                sum += 30;
            }
            sum
        }
        10 => {
            if large_straight(dice_throw) {
                //große Straße
                sum += 40;
            }
            sum
        }
        11 => {
            if kniffel(dice_throw) {
                sum += 50;
            }
            sum
        }
        12 => {
            for dice in dice_throw {
                sum += dice.eyes;
            }
            sum
        }
        _ => 255,
    }
}

fn three_same(dice_throw: DiceThrow) -> bool {
    let mut dice_roll = dice_throw;
    dice_roll.sort();

    for dice_index in 0..=2 {
        if dice_roll[dice_index].eyes == dice_roll[dice_index + 1].eyes
            && dice_roll[dice_index].eyes == dice_roll[dice_index + 2].eyes
        {
            return true;
        }
    }
    false
}

fn four_same(dice_throw: DiceThrow) -> bool {
    let mut dice_roll = dice_throw;
    dice_roll.sort();

    for dice_index in 0..=1 {
        if dice_roll[dice_index].eyes == dice_roll[dice_index + 1].eyes
            && dice_roll[dice_index].eyes == dice_roll[dice_index + 2].eyes
            && dice_roll[dice_index].eyes == dice_roll[dice_index + 3].eyes
        {
            return true;
        }
    }
    false
}

fn full_house(dice_throw: DiceThrow) -> bool {
    //bei
    if !three_same(dice_throw) {
        false
    } else {
        let mut dice_roll = dice_throw;
        dice_roll.sort();

        // also erste 3 Würfel sind gleich
        if dice_roll[0].eyes == dice_roll[2].eyes {
            //letzte 2 gleich aber anders als 1,2,3
            dice_roll[3].eyes == dice_roll[4].eyes && dice_roll[2].eyes != dice_roll[3].eyes
        } else {
            dice_roll[0].eyes == dice_roll[1].eyes && dice_roll[1].eyes != dice_roll[2].eyes
        }
    }
}

fn small_straight(dice_throw: DiceThrow) -> bool {
    let mut dice_roll = dice_throw;
    dice_roll.sort();
    let mut counter = 0;

    for index in 0..=3 {
        if dice_roll[index].eyes + 1 == dice_roll[index + 1].eyes {
            counter += 1;
        }
    }
    if counter > 2 {
        return true;
    }
    false
}

fn large_straight(dice_throw: DiceThrow) -> bool {
    let mut dice_roll = dice_throw;
    dice_roll.sort();
    let mut counter = 0;

    for index in 0..=3 {
        if dice_roll[index].eyes + 1 == dice_roll[index + 1].eyes {
            counter += 1;
        }
    }
    if counter > 3 {
        return true;
    }
    false
}

fn kniffel(dice_throw: DiceThrow) -> bool {
    for dice in 0..4 {
        if dice_throw[dice].eyes != dice_throw[dice + 1].eyes {
            return false;
        }
    }
    true
}

fn update_totals_point_table(mut point_table: PointTable) -> PointTable {
    let mut sum_top = 0;
    for point_index in 0..=5 {
        if point_table.points_thrown[point_index].is_some() {
            sum_top += point_table.points_thrown[point_index].unwrap();
        }
    }
    let mut sum_top_with_bonus = sum_top;
    if sum_top > 62 {
        sum_top_with_bonus += 35;
    }
    let mut sum_bottom = 0;
    for point_index in 6..=12 {
        if point_table.points_thrown[point_index].is_some() {
            sum_bottom += point_table.points_thrown[point_index].unwrap();
        }
    }
    let sum_total = sum_top_with_bonus + sum_bottom;
    point_table.total_points = [sum_top, sum_top_with_bonus, sum_bottom, sum_total];
    point_table
}

pub fn point_table_full(game: &Game) -> bool {
    for player_counter in 0..game.number_of_players {
        for counter in 0..13 {
            if game.all_players[player_counter].point_table.points_thrown[counter].is_none() {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_0playersequal_error() {
        let game = Game::new(0);
        assert!(game.is_err());
    }

    #[test]
    fn test_new_2playersworks() {
        let game = Game::new(2);

        let game2 = {
            Game {
                all_players: vec![Player::new(), Player::new()],
                current_player: Player {
                    point_table: PointTable::new(),
                    number_of_throws: 0,
                    dice_throw: [Dice::new(0); 5],
                },
                current_player_index: 0,
                number_of_players: 2,
            }
        };
        assert_eq!(game, Ok(game2));
    }

    #[test]
    fn test_four_same_not() {
        let dice_throw = [
            Dice::new(2),
            Dice::new(2),
            Dice::new(6),
            Dice::new(1),
            Dice::new(1),
        ];

        assert_eq!(four_same(dice_throw), false);
    }

    #[test]
    fn test_four_same_yes1() {
        let dice_throw = [
            Dice::new(2),
            Dice::new(2),
            Dice::new(6),
            Dice::new(2),
            Dice::new(2),
        ];
        assert_eq!(four_same(dice_throw), true);
    }

    #[test]
    fn test_four_same_yes2() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(1),
            Dice::new(1),
            Dice::new(1),
            Dice::new(2),
        ];
        assert_eq!(four_same(dice_throw), true);
    }

    #[test]
    fn test_three_same_not() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(2),
            Dice::new(6),
            Dice::new(3),
            Dice::new(2),
        ];
        assert_eq!(three_same(dice_throw), false);
    }

    #[test]
    fn test_three_same_yes() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(2),
            Dice::new(6),
            Dice::new(2),
            Dice::new(2),
        ];
        assert_eq!(three_same(dice_throw), true);
    }

    #[test]
    fn test_kniffel() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(1),
            Dice::new(1),
            Dice::new(1),
            Dice::new(1),
        ];
        assert_eq!(kniffel(dice_throw), true);
    }

    #[test]
    fn test_kniffel_not() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(1),
            Dice::new(6),
            Dice::new(1),
            Dice::new(1),
        ];
        assert_eq!(kniffel(dice_throw), false);
    }

    #[test]
    fn test_small_straight_not() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(2),
            Dice::new(6),
            Dice::new(3),
            Dice::new(2),
        ];
        assert_eq!(small_straight(dice_throw), false);
    }

    #[test]
    fn test_small_straight_yes() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(2),
            Dice::new(2),
            Dice::new(3),
            Dice::new(4),
        ];
        assert_eq!(small_straight(dice_throw), true);
    }

    #[test]
    fn test_large_straight_not() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(2),
            Dice::new(2),
            Dice::new(3),
            Dice::new(4),
        ];
        assert_eq!(large_straight(dice_throw), false);
    }

    #[test]
    fn test_large_straight_yes() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(5),
            Dice::new(2),
            Dice::new(3),
            Dice::new(4),
        ];
        assert_eq!(large_straight(dice_throw), true);
    }

    #[test]
    fn test_full_house_not() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(5),
            Dice::new(2),
            Dice::new(3),
            Dice::new(4),
        ];
        assert_eq!(full_house(dice_throw), false);
    }

    #[test]
    fn test_full_house_yes() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(5),
            Dice::new(1),
            Dice::new(5),
            Dice::new(1),
        ];
        assert_eq!(full_house(dice_throw), true);
    }

    #[test]
    fn test_calculate_points_3x1() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(5),
            Dice::new(1),
            Dice::new(1),
            Dice::new(4),
        ];

        assert_eq!(calculate_points(6, dice_throw), 12);
    }

    #[test]
    fn test_calculate_points_is_0() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(5),
            Dice::new(1),
            Dice::new(2),
            Dice::new(4),
        ];

        assert_eq!(calculate_points(6, dice_throw), 0);
    }

    #[test]
    fn test_calculate_points_is_255() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(5),
            Dice::new(1),
            Dice::new(2),
            Dice::new(4),
        ];

        assert_eq!(calculate_points(20, dice_throw), 255);
    }

    #[test]
    fn test_calculate_points_3x6() {
        let dice_throw = [
            Dice::new(6),
            Dice::new(6),
            Dice::new(6),
            Dice::new(6),
            Dice::new(4),
        ];

        assert_eq!(calculate_points(6, dice_throw), 28);
    }

    #[test]
    fn test_calculate_points_chance() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(5),
            Dice::new(1),
            Dice::new(1),
            Dice::new(4),
        ];

        assert_eq!(calculate_points(12, dice_throw), 12);
    }

    #[test]
    fn test_calculate_points_0x3() {
        let dice_throw = [
            Dice::new(1),
            Dice::new(5),
            Dice::new(1),
            Dice::new(1),
            Dice::new(4),
        ];

        assert_eq!(calculate_points(2, dice_throw), 0);
    }

    #[test]
    fn test_dice_throw() {
        let mut game = Game::new(2).unwrap();
        throw_dice(&mut game);

        let mut dice_roll = 0;

        for index in 0..5 {
            if game.current_player.dice_throw[index].eyes == 0 {
                dice_roll += 1;
            }
        }
        assert_eq!(dice_roll, 0);
    }

    #[test]
    fn test_dice_throw_switch_player() {
        let mut game = Game::new(2).unwrap();
        let cur_pl = game.current_player;
        throw_dice(&mut game);
        throw_dice(&mut game);
        throw_dice(&mut game);
        throw_dice(&mut game);
        let new_pl = game.current_player;

        let test = cur_pl == new_pl;

        assert!(!test);
    }

    #[test]
    fn test_change_blocked_stat_block() {
        let mut game = Game::new(2).unwrap();
        throw_dice(&mut game);
        change_blocked_status_dice(&mut game, 1);
        throw_dice(&mut game);
        let test = game.current_player.dice_throw[1].is_locked;

        assert!(test);
    }
}
