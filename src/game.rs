use super::Client;
use deck::Deck;

use ws::util::Token;
// use std::cell::{Cell, RefCell};
// use std::rc::Rc;
use std::collections::HashMap;

// pub struct GameState {
//     clients: Rc<RefCell<HashMap<Token, Client>>>,
//     current_player: Rc<RefCell<Option<Client>>>,
//     current_player_index: Rc<Cell<usize>>,
//     started: Rc<Cell<bool>>,
// }

pub struct GameState {
    clients: HashMap<Token, Client>,
    current_player: Option<Client>,
    current_player_index: usize,
    started: bool,
    deck: Deck,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            clients: HashMap::new(),
            current_player: None,
            current_player_index: 0,
            started: false,
            deck: Deck::new_shuffled(),
        }
    }

    pub fn add_client(&mut self, t: Token, c: Client) {
        self.clients.insert(t, c);
    }

    pub fn remove_client(&mut self, t: &Token) {
        self.clients.remove(t);
    }

    // pub fn set_current_player(&mut self, c: Client) {
    //     self.current_player = Some(c);
    // }

    // pub fn get_current_player(&self) -> &Option<Client> {
    //     &self.current_player
    // }

    pub fn next_player(&mut self) -> &Client {
        let players: Vec<&Client> = self.clients.values().collect();

        // Check bounds incase len has shrunk from players leaving
        if self.current_player_index >= self.clients.len() {
            self.current_player_index = 0;
        }

        let p = players[self.current_player_index];

        // Increment index after selecting player
        if self.current_player_index + 1 >= self.clients.len() {
            self.current_player_index = 0;
        } else {
            self.current_player_index += 1;
        }

        p
    }
}
