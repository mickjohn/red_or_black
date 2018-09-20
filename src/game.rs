use super::Client;
use deck::{Card, Deck};

use std::collections::HashMap;
use ws::util::Token;

pub struct GameState {
    clients: HashMap<Token, Client>,
    current_player: Option<Client>,
    current_player_index: usize,
    pub started: bool,
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

    pub fn get_card(&mut self) -> Card {
        if let Some(card) = self.deck.pop() {
            card
        } else {
            self.deck = Deck::new_shuffled();
            self.deck.pop().unwrap()
        }
    }

    pub fn get_clients(&self) -> &HashMap<Token, Client> {
        &self.clients
    }

    pub fn add_client(&mut self, t: Token, c: Client) {
        self.clients.insert(t, c);
    }

    pub fn remove_client(&mut self, t: Token) {
        self.clients.remove(&t);
    }

    pub fn get_current_player(&self) -> &Option<Client> {
        &self.current_player
    }

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

        self.current_player = Some(p.clone());

        p
    }
}
