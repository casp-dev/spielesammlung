use rand::Rng;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq)]
pub struct Dice {
    pub eyes: u8,
    pub is_locked: bool,
}

impl Ord for Dice {
    fn cmp(&self, other:&Self) -> Ordering {
        let dice1 = self.eyes;
        let dice2 = self.eyes;
        if dice1 > dice2 {
            return Ordering::Greater;
        }
        else if dice2 > dice1 {
            return Ordering::Less;
        } 
        return Ordering::Equal;
    }
}

impl Dice {
    fn new(eyes: u8) -> Self {
        Dice {
            eyes: eyes,
            is_locked: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointTable {
    pub points_thrown: [Option<u8>; 13], //also die Punkte bei einsern, zweiern etc
    pub total_points: [u8; 4], //Zwischensumme oben, oben mit bonus, unten, gesamt
}

pub type NumberOfThrows = u8;
pub type DiceThrow = [Dice; 5];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Player {
    pub point_table: PointTable,
    pub number_of_throws: u8,
    pub dice_throw: DiceThrow,
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
    fn new() -> Self {
        Player {
            point_table: PointTable::new(),
            number_of_throws: 1,
            dice_throw: [Dice::new(0); 5],
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Game {
    pub all_players: Vec<Player>,
    pub current_player: Player,
    pub number_of_players: usize,
}

pub trait YahtzeeGame {
    // new Game with a certain number of Players
    fn new(number: usize) -> Result<Game, &'static str>;
    // throw dice max 3 times, latest after 3rd throw change the point_table & current_player
    fn game_turn(&self) -> Self;
    
    fn winner(&self) -> Option<Player>;
}

impl YahtzeeGame for Game {
    fn new(players: usize) -> Result<Game, &'static str> {
        if players <2 || players > 4 {
            return Err("Spieleranzahl wird nicht unterstützt. Nur 2-4 Spieler möglich")
        }

        else {
            let all_players = vec![Player::new(); players];
            Ok(Self {
                all_players: all_players.clone(),
                current_player: all_players[0],
                number_of_players: players,
            })
        }

    }

    fn game_turn(&self) -> Self {
        unimplemented!()
    }

    fn winner(&self) -> Option<Player> {
        unimplemented!()
    }

}

/*
TODO: Hilfsfunktionen 
        - fn throw_dice(&self) -> Self; DONE - GETESTET, allen Würfeln, wird ein Wert zuegordnet
        - fn next_player; DONE - GETESTET
        - fn change_blocked_status_dice; DONE - blockieren getestet
        - fn add_dice_point_table; DONE - ungetstet
        - fn update_totals_point_table; DONE - ungetestet
        - fn calculate_points; DONE - GETESTET vmtl korrekt
        - fn potential_points supposed to (in gui) show you all the possible points across the table before selecting
*/

fn next_player(game: &mut Game) -> Option<&Game> {
    for current_player_index in 0..=(game.number_of_players-1) {
        if game.current_player == game.all_players[current_player_index] {
            game.current_player = game.all_players[(current_player_index+1) % game.number_of_players];
            return Some(game);
        }
    }
    None
}

pub fn throw_dice(game: &mut Game) {
    if game.current_player.number_of_throws > 3 {
        game.current_player.number_of_throws = 1;
    }

    else {
        game.current_player.number_of_throws += 1;
        for current_dice in 0..5 {
            if !game.current_player.dice_throw[current_dice].is_locked {
                game.current_player.dice_throw[current_dice].eyes = rand::thread_rng().gen_range(1..=6);
            }
        }
    }
}

fn change_blocked_status_dice(game: &mut Game, dice_index: usize) -> &Game {
    if game.current_player.dice_throw[dice_index].is_locked {
        game.current_player.dice_throw[dice_index].is_locked = false;
        return game;
    }
    game.current_player.dice_throw[dice_index].is_locked = true;
    return game;
}

fn add_dice_point_table(game: &mut Game, category: usize) -> Result<&Game, &'static str> {
    if category < 0 || category > 12 {
        return Err("Kategorie existiert nicht");
    }

    let selected_category = game.current_player.point_table.points_thrown[category];
    if selected_category.is_none() {
        game.current_player.point_table.points_thrown[category] = Some(calculate_points(category, game.current_player.dice_throw));
        update_totals_point_table(game.current_player.point_table);
        return Ok(game); 
    }

    else {
        return Err("Kategorie bereits belegt");
    }
    
}

fn calculate_points(category: usize, dice_throw: DiceThrow) -> u8 {
    let mut sum = 0;
    let u8_category = category as u8;
    match u8_category {
        0 | 1 | 2 | 3 | 4 | 5 => {
            for dice in 0..5 {
                if dice_throw[dice].eyes == (u8_category + 1) {
                    sum += u8_category + 1;
                }
            }
            return sum;
        }
        6 => {
            if three_same(dice_throw) {
                for dice in dice_throw {
                    sum += dice.eyes;
                }
            }
            return sum;
        } 
        7 => {
            if four_same(dice_throw) {
                for dice in dice_throw {
                    sum += dice.eyes;
                }
            }
            return sum;
        }
        8 => {
            if full_house(dice_throw) {
                sum += 25;
            }
            return sum;
        }
        9 => {
            if small_straight(dice_throw) { //kleine Straße
                sum += 30;
            }
            return sum;
        }
        10 => {
            if large_straight(dice_throw) { //große Straße
                sum += 40;
            }
            return sum;
        }
        11 => {
            if kniffel(dice_throw) {
                sum += 5*dice_throw[0].eyes;
            }
            return sum;
        }
        12 => {
            for dice in dice_throw {
                    sum += dice.eyes;
                }
            return sum;
        }
        _ => {
            return 255;
        }
    }
}

fn three_same(dice_throw: DiceThrow) -> bool {
    let mut dice_roll = dice_throw;
    dice_roll.sort();

    for dice_index in 0..=2 {
        if dice_roll[dice_index].eyes == dice_roll[dice_index+1].eyes && dice_roll[dice_index].eyes == dice_roll[dice_index+2].eyes {
            return true;
        }
    }
    return false;
}

fn four_same(dice_throw: DiceThrow) -> bool {
    let mut dice_roll = dice_throw;
    dice_roll.sort();

    for dice_index in 0..=1 {
        if dice_roll[dice_index].eyes == dice_roll[dice_index+1].eyes && 
           dice_roll[dice_index].eyes == dice_roll[dice_index+2].eyes &&
           dice_roll[dice_index].eyes == dice_roll[dice_index+3].eyes {
            return true;
           }
    }
    return false;
}

fn full_house(dice_throw: DiceThrow) -> bool {
    if !three_same(dice_throw) {
        return false;
    }
    else {
        let mut dice_roll = dice_throw;
        dice_roll.sort();

        if dice_roll[0].eyes == dice_roll[2].eyes { // also erste 3 Würfel sind gleich
            return dice_roll[3].eyes == dice_roll[4].eyes 
        }
        else {
            return dice_roll[0].eyes == dice_roll[1].eyes
        }
    }
}

fn small_straight(dice_throw: DiceThrow) -> bool {
    let mut dice_roll = dice_throw;
    dice_roll.sort();
    let mut counter = 0;

    for index in 0..=3 {
        if dice_roll[index].eyes+1 == dice_roll[index+1].eyes {
            counter += 1;
        }
    }
    if counter > 2 {
        return true;
    }
    return false;
}

fn large_straight(dice_throw: DiceThrow) -> bool {
   let mut dice_roll = dice_throw;
    dice_roll.sort();
    let mut counter = 0;

    for index in 0..=3 {
        if dice_roll[index].eyes+1 == dice_roll[index+1].eyes {
            counter += 1;
        }
    }
    if counter > 3 {
        return true;
    }
    return false;
}

fn kniffel(dice_throw: DiceThrow) -> bool {
    for dice in 0..4 {
        if dice_throw[dice].eyes != dice_throw[dice+1].eyes {
            return false;
        }
    }
    return true;
}

fn update_totals_point_table(mut point_table: PointTable) -> PointTable  {
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
    for point_index in 6..= 12 {
        if point_table.points_thrown[point_index].is_some() {
            sum_bottom += point_table.points_thrown[point_index].unwrap();
        }
    }
    let sum_total = sum_top_with_bonus + sum_bottom;
    point_table.total_points = [sum_top, sum_top_with_bonus, sum_bottom, sum_total];
    return point_table;
}


fn main() {
    println!("Hallo");
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

        let game2 = { Game 
            {all_players: vec![Player::new(), Player::new()],
            current_player: Player {
                point_table: PointTable::new(),
                number_of_throws: 1,
                dice_throw: [Dice::new(0); 5],
            },
            number_of_players: 2,}
        };
        assert_eq!(game, Ok(game2));
    }    

    #[test]
    fn test_four_same_not() {
        let dice_throw = [Dice::new(2), Dice::new(2), Dice::new(6), Dice::new(1), Dice::new(1)];

        assert_eq!(four_same(dice_throw), false);
    }

        #[test]
    fn test_four_same_yes1() {
        let dice_throw = [Dice::new(2), Dice::new(2), Dice::new(6), Dice::new(2), Dice::new(2)];
        assert_eq!(four_same(dice_throw), true);
    }

    #[test]
    fn test_four_same_yes2() {
        let dice_throw = [Dice::new(1), Dice::new(1), Dice::new(1), Dice::new(1), Dice::new(2)];
        assert_eq!(four_same(dice_throw), true);
    }

    #[test]
    fn test_three_same_not() {
        let dice_throw = [Dice::new(1), Dice::new(2), Dice::new(6), Dice::new(3), Dice::new(2)];
        assert_eq!(three_same(dice_throw), false);
    }

    #[test]
    fn test_three_same_yes() {
        let dice_throw = [Dice::new(1), Dice::new(2), Dice::new(6), Dice::new(2), Dice::new(2)];
        assert_eq!(three_same(dice_throw), true);
    }

    #[test]
    fn test_kniffel() {
        let dice_throw = [Dice::new(1), Dice::new(1), Dice::new(1), Dice::new(1), Dice::new(1)];
        assert_eq!(kniffel(dice_throw), true);
    }

    #[test]
    fn test_kniffel_not() {
        let dice_throw = [Dice::new(1), Dice::new(1), Dice::new(6), Dice::new(1), Dice::new(1)];
        assert_eq!(kniffel(dice_throw), false);
    }

    #[test]
    fn test_small_straight_not() {
        let dice_throw = [Dice::new(1), Dice::new(2), Dice::new(6), Dice::new(3), Dice::new(2)];
        assert_eq!(small_straight(dice_throw), false);
    }
    
    #[test]
    fn test_small_straight_yes() {
        let dice_throw = [Dice::new(1), Dice::new(2), Dice::new(2), Dice::new(3), Dice::new(4)];
        assert_eq!(small_straight(dice_throw), true);
    }

    #[test]
    fn test_large_straight_not() {
        let dice_throw = [Dice::new(1), Dice::new(2), Dice::new(2), Dice::new(3), Dice::new(4)];
        assert_eq!(large_straight(dice_throw), false);
    }

    #[test]
    fn test_large_straight_yes() {
        let dice_throw = [Dice::new(1), Dice::new(5), Dice::new(2), Dice::new(3), Dice::new(4)];
        assert_eq!(large_straight(dice_throw), true);
    }

    #[test]
    fn test_full_house_not() {
        let dice_throw = [Dice::new(1), Dice::new(5), Dice::new(2), Dice::new(3), Dice::new(4)];
        assert_eq!(full_house(dice_throw), false);
    }

    #[test]
    fn test_full_house_yes() {
        let dice_throw = [Dice::new(1), Dice::new(5), Dice::new(1), Dice::new(5), Dice::new(1)];
        assert_eq!(full_house(dice_throw), true);
    }

    #[test]
    fn test_calculate_points_3x1() {
        let dice_throw = [Dice::new(1), Dice::new(5), Dice::new(1), Dice::new(1), Dice::new(4)];

        assert_eq!(calculate_points(6, dice_throw), 12);
    }

    #[test]
    fn test_calculate_points_is_0() {
       let dice_throw = [Dice::new(1), Dice::new(5), Dice::new(1), Dice::new(2), Dice::new(4)];

        assert_eq!(calculate_points(6, dice_throw), 0);
    }

    #[test]
    fn test_calculate_points_is_255() { 
        let dice_throw = [Dice::new(1), Dice::new(5), Dice::new(1), Dice::new(2), Dice::new(4)];

        assert_eq!(calculate_points(20, dice_throw), 255);
    }

    #[test]
    fn test_calculate_points_3x6() {
        let dice_throw = [Dice::new(6), Dice::new(6), Dice::new(6), Dice::new(6), Dice::new(4)];

        assert_eq!(calculate_points(6, dice_throw), 28);
    }

    #[test]
    fn test_calculate_points_chance() {
        let dice_throw = [Dice::new(1), Dice::new(5), Dice::new(1), Dice::new(1), Dice::new(4)];

        assert_eq!(calculate_points(12, dice_throw), 12);
    }

    #[test]
    fn test_calculate_points_0x3() {
        let dice_throw = [Dice::new(1), Dice::new(5), Dice::new(1), Dice::new(1), Dice::new(4)];

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