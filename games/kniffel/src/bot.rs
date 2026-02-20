use crate::calculate_points;
use crate::change_blocked_status_dice;
use crate::kniffel;
use crate::kniffel::add_dice_point_table;
use crate::kniffel::throw_dice;
use crate::DiceThrow;
use crate::Rating::*;

#[derive(PartialEq)]
pub enum Rating {
    Straight,
    Double,
    FullHouse,
    Sacrifice,
}

pub fn achieve_best_category(game: &mut kniffel::Game) {
    let empty_categories = game.current_player.calculate_empty_categories();
    //alle Kategorien die >= 20 Punkte bringen (außer Chance)
    let mut important = Vec::new();
    let mut sacrifices = Vec::new();

    if empty_categories.is_empty() {
        return;
    }

    //vll bug? ansonsten halt empty_categories[category]
    for category in &empty_categories {
        if category < &3 || category == &12 {
            sacrifices.push(category);
        } else {
            important.push(category);
        }
    }

    if !important.is_empty() {
        //wenn shcon 3 Würfe höchste Punltzahl nehmen, ansonsten wenn nicht erfüllt / 0,
        //dann schauen ob sonst was leer, wenn nicht dann 0 punkte bei minimum
        if game.current_player.number_of_throws == 3 {
            //Kategorie mit höchster Punktzahl auffüllen
            if let Some(category) =
                find_max_points(game.current_player.dice_throw, empty_categories)
            {
                *game = add_dice_point_table(game, category).clone();
            }
        } else {
            //aktuell würfelt er auf Pasch, selbst wenn FullHouse geworfen wurde; TODO?
            let rating = rate_throw(empty_categories.clone(), game.current_player.dice_throw);
            match rating {
                Rating::Double => {
                    for double in [11, 7, 6] {
                        if empty_categories.contains(&double)
                            && calculate_points(double, game.current_player.dice_throw) != 0
                        {
                            *game = add_dice_point_table(game, double).clone();
                            return;
                        }
                    }

                    let dice_equal_at = two_or_more_equal_at(game.current_player.dice_throw);

                    if dice_equal_at[0].len() > dice_equal_at[1].len() {
                        block_dice(game, dice_equal_at[0].clone());
                    } else if !dice_equal_at[1].is_empty() {
                        block_dice(game, dice_equal_at[1].clone());
                    }
                }
                Rating::Straight => {
                    for straight in [10, 9] {
                        if empty_categories.contains(&straight)
                            && calculate_points(straight, game.current_player.dice_throw) != 0
                        {
                            *game = add_dice_point_table(game, straight).clone();
                            return;
                        }
                    }
                    block_straight(game);
                }
                Rating::FullHouse => {
                    if calculate_points(8, game.current_player.dice_throw) != 0 {
                        *game = add_dice_point_table(game, 8).clone();
                    } else {
                        block_full_house(game);
                    }
                }
                Sacrifice => {
                    //finde höchste punktzahl in leeren Kategorien
                    if let Some(category) =
                        find_max_points(game.current_player.dice_throw, empty_categories)
                    {
                        *game = add_dice_point_table(game, category).clone();
                    }
                }
            }
        }
    } else {
        //wenn alle Felder >=20 Punkte belegt sind
        if let Some(category) = find_max_points(game.current_player.dice_throw, empty_categories) {
            *game = add_dice_point_table(game, category).clone();
        }
    }
}

pub fn rate_throw(empty_cat: Vec<usize>, dice_throw: DiceThrow) -> Rating {
    let mut free_cat = Vec::new();
    for double in [6, 7, 11] {
        if empty_cat.contains(&double) {
            free_cat.push(double);
            //wir haben bereits mindestens 3er Pasch || bereits min. 2 gleiche Würfel, also versuchen wir größeren Pasch
            if calculate_points(double, dice_throw) != 0 || two_same(dice_throw) {
                return Double;
            }
        }
    }

    //wenn FullHouse frei ist; wird über Straße priorisiert
    if empty_cat.contains(&8) && two_same(dice_throw) {
        return FullHouse;
    }

    for straight in [9, 10] {
        if empty_cat.contains(&straight) {
            return Straight;
        }
    }
    Sacrifice
}

pub fn bot_game_turn(game: &mut kniffel::Game) {
    //merke die Anzahl gefüllter Kategorien vor dem Zug
    let filled_before = game
        .current_player
        .point_table
        .points_thrown
        .iter()
        .filter(|p| p.is_some())
        .count();

    loop {
        throw_dice(game);
        achieve_best_category(game);

        //ob Kat gefüllt ist
        let filled_after = game
            .current_player
            .point_table
            .points_thrown
            .iter()
            .filter(|p| p.is_some())
            .count();
        if filled_after > filled_before {
            break;
        }

        if game.current_player.number_of_throws >= 3 {
            achieve_best_category(game);
            break;
        }
    }
}

pub fn two_same(dice_throw: DiceThrow) -> bool {
    for index in 0..4 {
        for index2 in 0..5 {
            if dice_throw[index].eyes == dice_throw[index2].eyes {
                return true;
            }
        }
    }
    false
}

pub fn two_or_more_equal_at(dice_throw: DiceThrow) -> Vec<Vec<usize>> {
    let mut equal_dice = Vec::new();
    let mut equal_dice_2 = Vec::new();
    let mut equal_dice_eyes = 0;
    let mut equal_dice_eyes2 = 0;
    for dice in 0..4 {
        for dice_2 in dice + 1..5 {
            //die matchenden Augen entsprechen dem ersten Match
            if dice_throw[dice].eyes == equal_dice_eyes && dice_throw[dice] == dice_throw[dice_2] {
                equal_dice.push(dice);
                equal_dice.push(dice_2);
            }
            //falls es noch kein Match gab wird die Augenzahl des 1. Match gemerkt
            else if equal_dice_eyes == 0 && dice_throw[dice] == dice_throw[dice_2] {
                equal_dice_eyes = dice_throw[dice].eyes;
                equal_dice.push(dice);
                equal_dice.push(dice_2);
            }
            //wenn es ein zweites Paar/Pasch gibt; analog zum 1. Match
            else if dice_throw[dice].eyes == equal_dice_eyes2
                && dice_throw[dice] == dice_throw[dice_2]
            {
                equal_dice_2.push(dice);
                equal_dice_2.push(dice_2);
            } else if equal_dice_eyes2 == 0 && dice_throw[dice] == dice_throw[dice_2] {
                equal_dice_eyes2 = dice_throw[dice].eyes;
                equal_dice_2.push(dice);
                equal_dice_2.push(dice_2);
            }
        }
    }
    vec![equal_dice, equal_dice_2]
}

pub fn block_straight(game: &mut kniffel::Game) {
    let longest_seq = find_longest_sequence(game.current_player.dice_throw);

    block_dice(game, longest_seq);
}

pub fn block_full_house(game: &mut kniffel::Game) {
    let equal_dice = two_or_more_equal_at(game.current_player.dice_throw);
    let mut first_match = equal_dice[0].clone();
    let sec_match = equal_dice[1].clone();

    //wenn viererpash dann werden nur 3 gesichert; potenziell TODO: Fix für Kniffel
    if first_match.len() > 3 {
        first_match.pop();
    }

    block_dice(game, first_match);

    //der 2.Pasch maximal 3 Würfel, d.h. ist safe den immer zu blocken wenn er existiert
    if !sec_match.is_empty() {
        block_dice(game, sec_match);
    }
}

pub fn find_longest_sequence(dice_throw: DiceThrow) -> Vec<usize> {
    let mut longest_seq = Vec::new();
    let mut current_seq = Vec::new();
    //dice_roll ist vec mit augen
    let mut dice_roll = remove_duplicates(dice_throw);
    dice_roll.sort();

    for index in 0..dice_roll.len() {
        if current_seq.is_empty() {
            current_seq.push(index);
        } else {
            let last_index = current_seq.last().unwrap();
            let last_eyes = dice_roll[*last_index];

            if dice_roll[index] == last_eyes + 1 {
                current_seq.push(index);
            } else if dice_roll[index] != last_eyes {
                if current_seq.len() > longest_seq.len() {
                    longest_seq = current_seq.clone();
                }
                current_seq.clear();
                current_seq.push(index);
            }
        }
    }
    longest_seq
}

pub fn remove_duplicates(dice_throw: DiceThrow) -> Vec<u8> {
    let mut unique_eyes = Vec::new();

    for dice in &dice_throw {
        if !unique_eyes.contains(&dice.eyes) {
            unique_eyes.push(dice.eyes);
        }
    }
    unique_eyes
}

pub fn block_dice(game: &mut kniffel::Game, dice: Vec<usize>) {
    for &dice_index in &dice {
        if !game.current_player.dice_throw[dice_index].is_locked {
            change_blocked_status_dice(game, dice_index);
        }
    }
}

pub fn find_max_points(dice_throw: DiceThrow, empty_categories: Vec<usize>) -> Option<usize> {
    if empty_categories.is_empty() {
        return None;
    }

    let mut max_points = 0;
    let mut max_category = 99;
    for category in empty_categories {
        if calculate_points(category, dice_throw) >= max_points {
            max_points = calculate_points(category, dice_throw);
            max_category = category;
        }
    }
    Some(max_category)
}
