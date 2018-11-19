use red_or_black::RedOrBlack;

use serde_json;
use std::collections::HashMap;
use ws::util::Token;

use super::messages::*;
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
        let usernames: Vec<String> = clients_map.values().map(|c| c.username.clone()).collect();
        self.out
            .broadcast(SendableMessage::Players { players: usernames })
    }
    // end helpers

    fn handle_message(&mut self, msg: &ReceivableMessage) {
        use super::messages::ReceivableMessage::*;
        debug!("{:?}", msg);
        match msg {
            Login { username: ref u } => {
                self.add_client(u.to_string());
            }
            Guess { ref card_colour } => {
                self.recieved_guess(card_colour);
            }
        }
    }

    fn send_card_history(&mut self) {
        let game = self.game.borrow();
        let history = game.get_card_history().clone();
        self.out
            .send(SendableMessage::RequestHistory { history })
            .unwrap();
    }

    fn send_game_history(&mut self) {
        let game = self.game.borrow();
        let history = game.get_game_history().clone();
        self.out
            .send(SendableMessage::GameHistory { history })
            .unwrap();
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

        let penalty = self.game.borrow().get_penalty();

        // Send out updated player list
        self.broadcast_players().unwrap();

        // Tell the new player that they are logged in
        self.out.send(SendableMessage::LoggedIn).unwrap();

        // Tell the new player the penalty
        self.out.send(SendableMessage::Penalty { penalty }).unwrap();

        // Send the new player the last three cards
        self.send_card_history();

        // Send the new player the game history
        self.send_game_history();

        // Send now many cards are left
        self.out
            .send(SendableMessage::CardsLeft {
                cards_left: self.game.borrow().cards_left(),
            }).unwrap();

        // Tell the player whose turn it is
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
        let (correct, penalty, next_player, card, cards_left) = game.play_turn(card_colour);
        let message = SendableMessage::GuessResult {
            correct,
            card,
            penalty,
            username: current_player.clone(),
            guess: card_colour.clone(),
        };
        info!("{} was {}", current_player, correct);
        // Broadcast the result to everyone.
        self.out.broadcast(message).unwrap();
        self.out
            .broadcast(SendableMessage::CardsLeft { cards_left })
            .unwrap();
        self.out
            .broadcast(SendableMessage::Turn {
                username: next_player.unwrap().clone(),
            }).unwrap();
    }

    fn remove_client(&mut self) {
        info!("Removing client...");
        // Scope for clients & game borrow
        {
            let mut clients = self.clients.borrow_mut();
            let mut game = self.game.borrow_mut();

            if let Some(client) = clients.get(&self.out.token()) {
                if game.remove_player(&client.username) {
                    let player = game.get_current_player();
                    if let Some(p) = player {
                        self.out
                            .broadcast(SendableMessage::PlayerHasLeft {
                                username: p.clone(),
                            }).unwrap();
                        self.out
                            .broadcast(SendableMessage::Turn {
                                username: p.clone(),
                            }).unwrap();
                    }
                }
            }
            clients.remove(&self.out.token());
        }
        self.broadcast_players().unwrap();
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

// #[cfg(test)]
// mod integration {
//     // extern crate lazy_static;
//     use messages::*;

//     // DOESN'T WORK, the client is on a listen loop,  the test willl never finish...
//     mod login {
//         use ws::{listen, connect, CloseCode};
//         use std::cell::RefCell;
//         use std::collections::HashMap;
//         use std::rc::Rc;
//         use std::thread;
//         use super::super::{Server, RedOrBlack};
//         use red_or_black;
//         use messages::*;

//         #[test]
//         fn can_login() {
//             let child = thread::spawn(move || {
//                 let game = Rc::new(RefCell::new(RedOrBlack::new(Vec::new())));
//                 let clients = Rc::new(RefCell::new(HashMap::new()));
//                 // Start server
//                 listen("127.0.0.1:8000", |out| Server {
//                     out,
//                     game: game.clone(),
//                     clients: clients.clone(),
//                 }).unwrap();
//                 assert_eq!(clients.borrow().len(), 0);
//             });

//             connect("ws://127.0.0.1:8000", |out| {
//                 out.send(ReceivableMessage::Login { username: "mickjohn".to_string() });

//                 move |msg| {
//                     println!("Got message: {}", msg);
//                     out.close(CloseCode::Normal)
//                 }
//             }).unwrap();
//         }
//     }
// }
