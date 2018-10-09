// use super::Client;
use game2::RedOrBlack;

use serde_json;
use std::collections::HashMap;
use ws::util::Token;

use messages::*;
use std::cell::RefCell;
use std::rc::Rc;
use ws::Message::*;
use ws::{CloseCode, Handler, Message, Result as WsResult, Sender};

#[derive(Clone)]
pub struct Server {
    pub out: Sender,
    // The clients need to be modified by multiple connections
    pub clients: Rc<RefCell<HashMap<Token, Client>>>,
    // The game state also needs to be modified by multiple connections
    pub game: Rc<RefCell<RedOrBlack>>,
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
            if clients.get(&self.out.token()).is_some() {
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
            game.add_player(username);
            game.get_current_player().unwrap().clone()
        };

        self.broadcast_players().unwrap();
        self.out.send(SendableMessage::LoggedIn).unwrap();
        self.out
            .send(SendableMessage::Turn {
                username: current_player,
            }).unwrap();
    }

    fn check_is_players_go(&mut self) -> bool {
        let clients = self.clients.borrow();
        let mut game = self.game.borrow_mut();
        if let (Some(client), Some(username)) =
            (clients.get(&self.out.token()), game.get_current_player())
        {
            if &client.username == username {
                return true;
            }
        }
        false
    }

    fn recieved_guess(&mut self, card_colour: &CardColour) {
        if !self.check_is_players_go() {
            // It's not this players go, do nothing.
            return;
        }
        let mut game = self.game.borrow_mut();
        let current_player = game.get_current_player().unwrap().clone();
        let (correct, penalty, next_player, card) = game.play_turn(card_colour);
        let message = SendableMessage::GuessResult {
            correct,
            card,
            penalty,
            username: current_player,
        };
        // let message = if correct {
        //     debug!("correct guess for {}", current_player);
        //     SendableMessage::CorrectGuess {
        //         drinking_seconds: penalty,
        //         username: current_player,
        //     }
        // } else {
        //     debug!("incorrect guess for {}", current_player);
        //     SendableMessage::WrongGuess {
        //         drinking_seconds: penalty,
        //         username: current_player,
        //     }
        // };
        self.out.send(message).unwrap();
        self.out
            .send(SendableMessage::Turn {
                username: next_player.unwrap().clone(),
            }).unwrap();
    }

    fn remove_client(&mut self) {
        debug!("Removing client");
        let mut clients = self.clients.borrow_mut();
        let mut game = self.game.borrow_mut();

        if let Some(client) = clients.get(&self.out.token()) {
            if game.remove_player(&client.username) {
                let player = game.get_current_player();
                if let Some(p) = player {
                    self.out.broadcast(SendableMessage::PlayerHasLeft {
                        username: p.clone(),
                    }).unwrap();
                    self.out.broadcast(SendableMessage::Turn {
                        username: p.clone(),
                    }).unwrap();
                }
            }
        }
        clients.remove(&self.out.token());
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

#[cfg(test)]
mod integration {
    // extern crate lazy_static;
    use messages::*;

    // DOESN'T WORK, the client is on a listen loop,  the test willl never finish...
    // mod login {
    //     use ws::{listen, connect, CloseCode};
    //     use std::cell::RefCell;
    //     use std::collections::HashMap;
    //     use std::rc::Rc;
    //     use game;
    //     use game2;
    //     use messages::*;

    //     #[test]
    //     fn can_login() {
    //         let game = Rc::new(RefCell::new(game2::RedOrBlack::new(Vec::new())));
    //         let clients = Rc::new(RefCell::new(HashMap::new()));
    //         // Start server
    //         listen("127.0.0.1:8000", |out| game::Server {
    //             out,
    //             game: game.clone(),
    //             clients: clients.clone(),
    //         }).unwrap();

    //         assert_eq!(clients.borrow().len(), 0);

    //         connect("ws://127.0.0.1:8000", |out| {
    //             out.send(ReceivableMessage::Login { username: "mickjohn".to_string() });

    //             move |msg| {
    //                 println!("Got message: {}", msg);
    //                 out.close(CloseCode::Normal)
    //             }
    //         }).unwrap();
    //     }
    // }
}
