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
    drinking_seconds: u16,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            clients: HashMap::new(),
            current_player: None,
            current_player_index: 0,
            started: false,
            deck: Deck::new_shuffled(),
            drinking_seconds: 0,
        }
    }

    pub fn increment_drinking_seconds(&mut self) {
        self.drinking_seconds += 5;
    }

    pub fn reset_drinking_seconds(&mut self) {
        self.drinking_seconds = 0;
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

    // Remove a client from the clients map
    // If it's the clients go, then progress the turn to the next player.
    // If the player who is leaving is the only player, then stop the game
    // This function returns true if the 'current_player' has changed. This is so that the calling
    // function can decide whether or not to broadcast a message to the clients indicating that the
    // current_player has changed.
    pub fn remove_client(&mut self, t: Token) -> bool {
        if self.clients.get(&t) == self.current_player.as_ref() {
            self.clients.remove(&t);
            if self.clients.len() > 1 {
                debug!(
                    "beforing changing to next player: {:?}",
                    self.current_player
                );
                self.next_player();
                debug!("Changing to next player: {:?}", self.current_player);
                true
            } else {
                debug!("The only player has left the game.");
                self.started = false;
                false
            }
        } else {
            debug!("The player who's leaving isn't the current player");
            self.clients.remove(&t);
            false
        }
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

        let player = players[self.current_player_index];
        self.current_player_index += 1;
        self.current_player = Some(player.clone());
        player
    }
}
