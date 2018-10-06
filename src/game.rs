// use super::Client;
use deck::{Card, Deck};
use game2::RedOrBlack;

use serde_json;
use std::collections::HashMap;
use std::sync::Mutex;
use ws;
use ws::util::Token;

use messages::*;
use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use ws::Message::*;
use ws::{listen, CloseCode, Handler, Message, Result as WsResult, Sender};

#[derive(Clone)]
pub struct Server {
    pub out: Sender,
    // The clients need to be modified by multiple connections
    pub clients: Rc<RefCell<HashMap<Token, Client>>>,
    // The game state also needs to be modified by multiple connections
    pub game: Rc<RefCell<RedOrBlack>>,
    // A mutex to allow only one 'thread' to send messages at a time.
    // This is to ensure messages are sent in the order I want them to.
    // pub write_lock: Rc<Mutex<()>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Client {
    pub username: String,
    pub token: usize,
}

impl Server {
    // Helper functions
    fn unrecognised_msg() -> String {
        let resp = SendableMessage::Error {
            error: "Unrecognised message".to_string(),
        };
        serde_json::to_string(&resp).unwrap()
    }

    fn broadcast_players(&mut self) -> WsResult<()> {
        let clients_map = self.clients.borrow();
        self.out.broadcast(SendableMessage::Players {
            players: clients_map.values().cloned().collect(),
        })
    }
    // end helpers

    fn handle_message(&mut self, msg: &ReceivableMessage) {
        match msg {
            ReceivableMessage::Login { username: ref u } => {
                self.add_client(u.to_string());
            }
            ReceivableMessage::Guess { ref card_colour } => {
                self.recieved_guess(card_colour);
            }
        }
    }

    fn add_client(&mut self, username: String) {
        // scope for clients mutable borrow
        info!("Adding client {}", username);
        {
            let mut clients = self.clients.borrow_mut();
            if let Some(_) = clients.get(&self.out.token()) {
                // Client already exists.. do nothing
                info!("{} is already logged in... doing nothing.", username);
                return;
            }

            clients.insert(
                self.out.token(),
                Client {
                    username: username.clone(),
                    token: self.out.token().0,
                },
            );
        }

        // scope for game mutable borrow
        let current_player = {
            let mut game = self.game.borrow_mut();
            game.add_player(username.clone());
            game.get_current_player().unwrap().clone()
        };

        self.broadcast_players();
        self.out.send(SendableMessage::LoggedIn);
        self.out.send(SendableMessage::Turn { username: current_player });
    }

    fn check_is_players_go(&mut self) -> bool {
        let clients = self.clients.borrow();
        let mut game = self.game.borrow_mut();
        if let (Some(client), Some(username)) =
            (clients.get(&self.out.token()), game.get_current_player())
        {
            if &client.username == username {
                return true
            }
        }
        false
    }

    fn recieved_guess(&mut self, card_colour: &CardColour) {
        if !self.check_is_players_go() {
            return;
        }
        let mut game = self.game.borrow_mut();
        let current_player = game.get_current_player().unwrap().clone();
        let (correct, penalty, next_player) = game.play_turn(card_colour);
        let message = if correct {
            debug!("correct guess for {}", current_player);
            SendableMessage::CorrectGuess {
                drinking_seconds: penalty,
                username: current_player,
            }
        } else {
            debug!("incorrect guess for {}", current_player);
            SendableMessage::WrongGuess {
                drinking_seconds: penalty,
                username: current_player,
            }
        };
        self.out.send(message);
        self.out.send(SendableMessage::Turn {
            username: next_player.unwrap().clone(),
        });
    }

    fn write_messages(&mut self, messages: Vec<SendableMessage>) -> WsResult<()> {
        for message in messages {
            self.out.send(message)?;
        }
        Ok(())
    }

    fn broadcast_messages(&self, messages: Vec<SendableMessage>) -> WsResult<()> {
        for message in messages {
            self.out.broadcast(message)?;
        }
        Ok(())
    }

    fn remove_client(&mut self) {
        debug!("Removing client");
        let mut broadcast_messages = Vec::new();
        let mut clients = self.clients.borrow_mut();
        let mut game = self.game.borrow_mut();

        if let Some(client) = clients.get(&self.out.token()) {
            if game.remove_player(&client.username) {
                let player = game.get_current_player();
                if let Some(p) = player {
                    broadcast_messages.push(SendableMessage::PlayerHasLeft {
                        username: p.clone(),
                    });
                    broadcast_messages.push(SendableMessage::Turn {
                        username: p.clone(),
                    });
                }
            }
        }
        clients.remove(&self.out.token());
        self.broadcast_messages(broadcast_messages);
    }
}

impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> WsResult<()> {
        debug!("Received message: {}", msg);
        match msg {
            Text(s) => match serde_json::from_str::<ReceivableMessage>(&s) {
                Ok(rmsg) => self.handle_message(&rmsg),
                _ => self.out.send(Server::unrecognised_msg()).unwrap(),
            },
            _ => self.out.send(Server::unrecognised_msg()).unwrap(),
        };
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        // The WebSocket protocol allows for a utf8 reason for the closing state after the
        // close code. WS-RS will attempt to interpret this data as a utf8 description of the
        // reason for closing the connection. I many cases, `reason` will be an empty string.
        // So, you may not normally want to display `reason` to the user,
        // but let's assume that we know that `reason` is human-readable.

        self.remove_client();
        match code {
            CloseCode::Normal => info!("The client is done with the connection."),
            CloseCode::Away => info!("The client is leaving the site."),
            _ => error!("Close code: {:?}, reason: {}", code, reason),
        }
    }
}
