use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dice {
    pub eyes: u8,
    pub is_locked: bool,
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

    fn next_player(self) -> Self {
    unimplemented!();
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
        - fn throw_dice(&self) -> Self; DONE
        - fn next_player
        - fn block_dice
        - fn add_dice_point_table
        - fn update_totals_point_table
        - fn calculate_points
        - fn potential_points supposed to (in gui) show you all the possible points across the table before selecting
*/

fn throw_dice(mut game: Game) -> Game {
    if game.current_player.number_of_throws > 3 {
        game.current_player.number_of_throws = 1;
        game.current_player = game.current_player.next_player(); //TODO next_player()
        return game;
    }

    else {
        game.current_player.number_of_throws += 1;
        for current_dice in 0..4 {
            if !game.current_player.dice_throw[current_dice].is_locked {
                game.current_player.dice_throw[current_dice].eyes = rand::thread_rng().gen_range(1..=6);
            }
        }
        return game;

    }
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
}