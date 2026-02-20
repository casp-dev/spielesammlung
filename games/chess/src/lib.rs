use std::{collections::HashMap, usize};

use egui::{Color32, RichText, Ui};
use game_core::{Game, MultiplayerGame};

use serde_json::Value;
use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

mod engine;
mod meeples;
use crate::{
    draw::draw_board,
    engine::{calculate_board, Engine},
    meeples::{opposite_color, Color, Meeple, Type},
};
mod draw;

pub struct ChessGame {
    state: String,
    pub game_board: [[Option<Meeple>; 8]; 8],
    possible_moves: [[Option<Vec<(usize, usize)>>; 8]; 8],
    pub shown_moves: Option<Vec<(usize, usize)>>,
    casteling_rights: ((bool, bool), (bool, bool)),
    en_passant_pos: Option<(usize, usize)>,
    repeat: HashMap<String, u8>,
    logs: Vec<((usize, usize), (usize, usize))>,
    clicked_meeple: (usize, usize),
    turn: Color,
    pawn_mutate: bool,
    engine: Option<Engine>,
    possible_bot_level: u16,
    client: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    multiplayer: Option<Color>,
    room_key: String,
}

impl ChessGame {
    ///this functions creates a new Chessboard with the state "initial" and no Engine
    pub fn new() -> Self {
        let state_ = "initial".to_string();
        let mut chess_board: [[Option<Meeple>; 8]; 8] = Default::default();
        let logs_ = vec![((42, 42), (42, 42))];
        let turn_ = Color::White;
        for x in 0..=7 {
            for y in 0..=7 {
                match y.clone() {
                    0 => {
                        chess_board[x][y] = Some(ChessGame::create_special_line(
                            (x.try_into().unwrap(), y.try_into().unwrap()),
                            Color::Black,
                        ))
                    }
                    1 => {
                        chess_board[x][y] = Some(Meeple::new(
                            (x.try_into().unwrap(), y.try_into().unwrap()),
                            Type::Pawn,
                            Color::Black,
                            1.0,
                        ))
                    }
                    6 => {
                        chess_board[x][y] = Some(Meeple::new(
                            (x.try_into().unwrap(), y.try_into().unwrap()),
                            Type::Pawn,
                            Color::White,
                            1.0,
                        ))
                    }
                    7 => {
                        chess_board[x][y] = Some(ChessGame::create_special_line(
                            (x.try_into().unwrap(), y.try_into().unwrap()),
                            Color::White,
                        ))
                    }
                    _ => continue,
                }
            }
        }
        let possible_moves_ = create_basic_possible_moves();
        Self {
            state: state_,
            game_board: chess_board,
            possible_moves: possible_moves_,
            shown_moves: None,
            logs: logs_,
            casteling_rights: ((false, false), (false, false)),
            en_passant_pos: Default::default(),
            repeat: HashMap::new(),
            clicked_meeple: (42, 42),
            turn: turn_,
            pawn_mutate: false,
            engine: None,
            possible_bot_level: 3,
            client: None,
            multiplayer: None,
            room_key: String::new(),
        }
    }

    ///this function creates one meeple for
    fn create_special_line(cords: (usize, usize), color: Color) -> Meeple {
        match cords.0 {
            0 | 7 => Meeple::new(cords, Type::Rook, color, 5.0),
            1 | 6 => Meeple::new(cords, Type::Knight, color, 2.7),
            2 | 5 => Meeple::new(cords, Type::Bishop, color, 3.0),
            3 => Meeple::new(cords, Type::Queen, color, 9.0),
            4 => Meeple::new(cords, Type::King, color, 0.0),
            _ => panic!("Something went wrong while creating a special row"),
        }
    }

    pub fn show_moves(&mut self, (x, y): (usize, usize)) {
        if self.state == "Tie because of triple repetition"
            || self.state == "White has won"
            || self.state == "Black has won"
        {
            return;
        }

        if let Some(color) = self.multiplayer {
            if color != self.turn {
                return;
            }
        }
        self.shown_moves = self.possible_moves[x][y].clone();
        self.clicked_meeple = (x, y);
    }

    pub fn move_meeple(&mut self, scnd: (usize, usize)) {
        if self.state == "White has won"
            || self.state == "Black has won"
            || self.state == "Tie because of triple repetition"
        {
            return;
        }

        let frst = self.clicked_meeple.clone();

        if self.check_casteling(frst, scnd) {
            self.casteling_meeple(frst, scnd);
        } else if self.check_en_passant(frst, scnd) {
            self.game_board[scnd.0][frst.1] = None;
        }
        walk_and_replace(frst, scnd, &mut self.game_board);

        self.game_board[scnd.0][scnd.1]
            .as_mut()
            .unwrap()
            .move_counter += 1;
        self.logs.push((self.clicked_meeple, scnd));
        self.shown_moves = Default::default();
        self.check_pawn_mutate(scnd);
        self.turn = opposite_color(self.turn);
        self.state = calculate_board(self.game_board).to_string();
        self.get_all_possible_moves();
        self.triple_repetition();
        self.move_engine();
        self.move_multiplayer();
    }

    fn check_casteling(&self, frst_pos: (usize, usize), scnd_pos: (usize, usize)) -> bool {
        let frst = self.game_board[frst_pos.0][frst_pos.1].unwrap();
        let cmp_value = frst.pos.0 as i8 - scnd_pos.0 as i8;
        if frst.typ == Type::King && (cmp_value == 2 || cmp_value == -2) {
            return true;
        }
        false
    }

    fn casteling_meeple(&mut self, frst: (usize, usize), scnd: (usize, usize)) {
        let cmp_value = frst.0 as i8 - scnd.0 as i8;
        if cmp_value < 0 {
            let new_rook_pos: (usize, usize) = (5, scnd.1);
            walk_and_replace((7, scnd.1), new_rook_pos, &mut self.game_board);
        } else {
            let new_rook_pos: (usize, usize) = (3, scnd.1);
            walk_and_replace((0, scnd.1), new_rook_pos, &mut self.game_board);
        }
    }

    fn check_en_passant(&self, frst_pos: (usize, usize), scnd_pos: (usize, usize)) -> bool {
        let frst = self.game_board[frst_pos.0][frst_pos.1].unwrap();
        if frst.typ == Type::Pawn && self.game_board[scnd_pos.0][scnd_pos.1] == None {
            if let Some(opposite_color_pawn) = self.game_board[scnd_pos.0][frst_pos.1] {
                if opposite_color_pawn.typ == Type::Pawn && opposite_color_pawn.color != frst.color
                {
                    return true;
                }
            }
        }
        false
    }

    fn check_pawn_mutate(&mut self, scnd_pos: (usize, usize)) {
        if let Some(pawn) = self.game_board[scnd_pos.0][scnd_pos.1].as_mut() {
            if pawn.typ == Type::Pawn
                && ((pawn.color == Color::White && pawn.pos.1 == 0)
                    || (pawn.color == Color::Black && pawn.pos.1 == 7))
            {
                if let Some(bot) = self.engine {
                    if bot.color == pawn.color {
                        self.mutate_pawn(Type::Queen);
                        return;
                    }
                    self.pawn_mutate = true;
                } else {
                    self.pawn_mutate = true;
                }
            }
        }
    }

    fn mutate_pawn(&mut self, mutate_into: Type) {
        let pawn_pos = self.logs.last().unwrap().1;
        if let Some(pawn) = self.game_board[pawn_pos.0][pawn_pos.1].as_mut() {
            pawn.typ = mutate_into;
            pawn.value = match pawn.typ {
                Type::Queen => 9.0,
                Type::Rook => 5.0,
                Type::Bishop => 3.0,
                Type::Knight => 3.0,
                _ => panic!("This should not happen"),
            };
        }
        self.pawn_mutate = false;
        if !self.engine.is_none() {
            self.move_engine();
        }
    }

    fn move_multiplayer(&mut self) {
        if let Some(multiplayer_color) = self.multiplayer {
            if multiplayer_color != self.turn {
                let x = format!(
                    "[{},{}]",
                    self.logs.last().unwrap().0 .0,
                    self.logs.last().unwrap().0 .1
                );
                let y = format!(
                    "[{},{}]",
                    self.logs.last().unwrap().1 .0,
                    self.logs.last().unwrap().1 .1
                );
                let move_msg =
                    format!(r#"{{ "type": "GameMove", "data": {{ "from" : {x}, "to": {y} }} }}"#,);
                self.send(&move_msg).unwrap();
                println!("send move message");
                self.wait_one_reply_game();
            }
        }
    }

    fn move_engine(&mut self) {
        if let Some(bot) = self.engine {
            if bot.color == self.turn && self.pawn_mutate == false {
                let bot_move =
                    bot.move_move(&mut self.game_board, &self.logs.last().unwrap(), self.turn);
                if bot_move == ((42, 42), (42, 42)) {
                    self.state = format!("{:?} has won", opposite_color(self.turn));
                } else {
                    self.show_moves(bot_move.0);
                    self.move_meeple(bot_move.1);
                }
            }
        }
    }

    fn get_all_possible_moves(&mut self) {
        let mut ret_vec: [[Option<Vec<(usize, usize)>>; 8]; 8] = Default::default();
        let colores = get_meeples_from_color(&self.game_board, self.turn);
        let mut can_move = false;
        for colored_meeple in colores.0 {
            let meeples = get_meeples_from_color(&self.game_board, Color::White);
            let can_hit = colored_meeple.show_legal_moves(
                &self.game_board,
                &self.logs.last().unwrap(),
                &meeples.0,
                &meeples.1,
            );
            if !can_hit.0.is_empty() {
                can_move = true;
            }

            self.en_passant_pos = can_hit.1;
            self.casteling_rights = can_hit.2;
            ret_vec[colored_meeple.pos.0][colored_meeple.pos.1] = Some(can_hit.0);
        }

        if !can_move {
            //patt
            self.state = format!("{:?} has won", opposite_color(self.turn));
        }
        self.possible_moves = ret_vec.clone();
    }

    ///this fct makes a key out of the current board which is saved like FEN (Forsyth–Edwards Notation) (the only
    ///difference is that the en passant is different) and looks if
    ///the key is already put into the hashmap
    fn triple_repetition(&mut self) {
        let mut hash_key = String::new();
        let mut empty_counter = 0;
        for y in 0..8 {
            for x in 0..8 {
                if let Some(meeple) = self.game_board[x][y] {
                    if empty_counter != 0 {
                        hash_key.push_str(&empty_counter.to_string());
                        empty_counter = 0;
                    }
                    hash_key.push(meeple.get_char());
                } else {
                    empty_counter += 1;
                }
            }
            if empty_counter != 0 {
                hash_key.push_str(&empty_counter.to_string());
                empty_counter = 0;
            }
            if y == 7 {
                hash_key.push(' ');
            } else {
                hash_key.push('/');
            }
        }
        hash_key.push_str(if self.turn == Color::White {
            "w "
        } else {
            "b "
        });
        if self.casteling_rights.0 .0 {
            hash_key.push('K');
        } else {
            hash_key.push('-');
        }
        if self.casteling_rights.0 .1 {
            hash_key.push('Q');
        } else {
            hash_key.push('-');
        }
        if self.casteling_rights.1 .0 {
            hash_key.push('k');
        } else {
            hash_key.push('-');
        }
        if self.casteling_rights.1 .1 {
            hash_key.push('q');
        } else {
            hash_key.push('-');
        }

        hash_key.push(' ');

        if let Some(ed_pos) = self.en_passant_pos {
            hash_key.push_str(ed_pos.0.to_string().as_str());
            hash_key.push_str(ed_pos.1.to_string().as_str());
        } else {
            hash_key.push('-');
        }

        self.repeat
            .entry(hash_key.clone())
            .and_modify(|e| *e += 1)
            .or_insert(1);
        if let Some(count) = self.repeat.get(&hash_key) {
            if *count >= 3 {
                self.state = "Tie because of triple repetition".to_string();
            }
        }
    }
}

fn walk_and_replace(
    frst: (usize, usize),
    scnd: (usize, usize),
    chess_board: &mut [[Option<Meeple>; 8]; 8],
) {
    chess_board[scnd.0][scnd.1] = chess_board[frst.0][frst.1].take();
    chess_board[scnd.0][scnd.1].as_mut().unwrap().pos = scnd;
}

fn create_basic_possible_moves() -> [[Option<Vec<(usize, usize)>>; 8]; 8] {
    let mut ret_vec: [[Option<Vec<(usize, usize)>>; 8]; 8] = Default::default();
    for index in 0..8 {
        ret_vec[index][6] = Some(vec![(index, 5), (index, 4)]);
    }
    ret_vec[1][7] = Some(vec![(0, 5), (2, 5)]);
    ret_vec[6][7] = Some(vec![(7, 5), (5, 5)]);
    ret_vec
}

pub fn get_meeples_from_color(
    chess_board: &[[Option<Meeple>; 8]; 8],
    color_at_0: Color,
) -> (Vec<Meeple>, Vec<Meeple>) {
    let mut ret_vec: (Vec<Meeple>, Vec<Meeple>) = (Vec::new(), Vec::new());
    let mut kings: (Vec<Meeple>, Vec<Meeple>) = (Vec::new(), Vec::new());
    for y in 0..8 {
        for x in 0..8 {
            if let Some(meeple) = chess_board[x][y] {
                if meeple.color == color_at_0 {
                    if meeple.typ == Type::King {
                        kings.0.push(meeple);
                    } else {
                        ret_vec.0.push(meeple);
                    }
                } else {
                    if meeple.typ == Type::King {
                        kings.1.push(meeple);
                    } else {
                        ret_vec.1.push(meeple);
                    }
                }
            }
        }
    }
    ret_vec.0.append(&mut kings.0);
    ret_vec.1.append(&mut kings.1);
    ret_vec
}

impl Game for ChessGame {
    fn name(&self) -> &str {
        "Chess"
    }

    fn ui(&mut self, ui: &mut Ui) {
        if self.state == "waiting for opponent" {
            ui.heading("Rust Go - Multiplayer");
            ui.label(format!("Room ID: {}", self.room_key));
            ui.label("Warte auf Gegner...");

            // Non-blocking: auf PlayerJoined msg warten
            if self.client.is_some() {
                ui.ctx().request_repaint();

                let received = match self.client.as_mut().unwrap().read() {
                    Ok(tungstenite::Message::Text(txt)) => Some(txt),
                    Err(tungstenite::Error::Io(ref e))
                        if e.kind() == std::io::ErrorKind::WouldBlock =>
                    {
                        None
                    }
                    _ => None,
                };

                if let Some(txt) = received {
                    if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                        if v.get("type").and_then(|t| t.as_str()) == Some("PlayerJoined") {
                            self.start_multiplayer_game();
                        }
                    }
                }
            }

            if ui.button("Spiel starten").clicked() {}
        } else if self.state != "initial" {
            if self.state == "Tie because of triple repetition"
                || self.state == "White has won"
                || self.state == "Black has won"
            {
                ui.heading(RichText::new(&self.state).strong().color(Color32::RED));
            } else {
                ui.heading(format!("score: {}", self.state));
            }
            if self.multiplayer.is_some() {
                self.wait_one_reply_game();
            } else {
                let reset_btn = egui::Button::new("Reset Game");
                if ui.add(reset_btn).clicked() {
                    let bot = self.engine.clone();
                    *self = ChessGame::new();
                    self.engine = bot;
                    self.state = "0.0".to_string();
                }
            }
            draw_board(ui, self);
        } else {
            self.multipalyer_ui(ui, true, false);
        }
    }
}

impl MultiplayerGame for ChessGame {
    fn on_text(&mut self, str: String) {
        println!("Received: {}", str);

        let v: Value = serde_json::from_str(&str).unwrap();

        let from = &v["data"]["from"];
        let to = &v["data"]["to"];

        let x1 = from[0].as_u64().unwrap() as usize;
        let y1 = from[1].as_u64().unwrap() as usize;

        let x2 = to[0].as_u64().unwrap() as usize;
        let y2 = to[1].as_u64().unwrap() as usize;

        if self.turn == self.multiplayer.unwrap() {
            return;
        }
        self.clicked_meeple = (x1, y1);
        self.move_meeple((x2, y2));
    }

    fn local_button_clicked(&mut self, player_counter: Option<u16>) -> Option<u16> {
        self.state = String::from("Play Local");
        player_counter
    }

    fn bot_button_clicked(&mut self, bot_level: Option<u16>) -> Option<u16> {
        self.state = String::from("Play vs Bot");
        self.engine = Some(Engine::new(bot_level.unwrap(), Color::Black));
        bot_level
    }

    fn set_client(&mut self, client: WebSocket<MaybeTlsStream<TcpStream>>) {
        self.client = Some(client);
    }

    fn get_client(&mut self) -> &mut WebSocket<MaybeTlsStream<TcpStream>> {
        self.client.as_mut().unwrap()
    }

    fn get_room_key_text(&mut self) -> &mut String {
        &mut self.room_key
    }

    fn set_room_key_text(&mut self, text: String) {
        self.room_key = text;
    }

    fn player_count_slider(&mut self, ui: &mut Ui) -> u16 {
        self.ui(ui);
        0
    }

    fn bot_level_slider(&mut self, ui: &mut Ui) -> u16 {
        ui.add(
            egui::Slider::new(&mut self.possible_bot_level, 1..=7).text("What level for the bot?"),
        );
        self.possible_bot_level
    }

    fn start_multiplayer_game(&mut self) {
        // non-blocking für spiel
        if let Some(ref client) = self.client {
            if let tungstenite::stream::MaybeTlsStream::Plain(ref tcp) = *client.get_ref() {
                let _ = tcp.set_nonblocking(true);
            }
        }

        if self.state != "waiting for opponent" {
            self.multiplayer = Some(Color::White);
            println!("i am white");
            self.state = String::from("Multiplayer as White");
        } else {
            self.multiplayer = Some(Color::Black);
            println!("i am black");
            self.state = String::from("Multiplayer as Black");
            self.wait_one_reply_game();
        }
    }

    fn create_host_button_clicked(&mut self) {
        // Verbindet und erstellt Raum
        if self
            .connect(String::from("ws://localhost:9000"), None)
            .is_err()
        {
            self.set_room_key_text(String::from("Connection failed"));
            return;
        }
        if self.send(r#"{ "type": "CreateRoom" }"#).is_err() {
            self.set_room_key_text(String::from("Communication error"));
            return;
        }
        let json_str = self.wait_one_reply();
        let v: Value = match serde_json::from_str(&json_str) {
            Ok(val) => val,
            Err(_) => {
                self.set_room_key_text(String::from("json parse failed"));
                return;
            }
        };
        let room_id = match v.get("room_id").and_then(|id| id.as_str()) {
            Some(id) => id.to_string(),
            None => {
                self.set_room_key_text(String::from("bad server response"));
                return;
            }
        };
        self.set_room_key_text(room_id);

        self.multiplayer = Some(Color::White);
        self.state = String::from("waiting for opponent");

        // non-blocking für spiel
        if let Some(ref client) = self.client {
            if let tungstenite::stream::MaybeTlsStream::Plain(ref tcp) = *client.get_ref() {
                let _ = tcp.set_nonblocking(true);
            }
        }
    }
}
