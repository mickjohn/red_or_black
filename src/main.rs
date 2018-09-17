extern crate rand;
extern crate ws;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod messages;
mod deck;
mod game;

use messages::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use ws::util::Token;
use ws::Message::*;
use ws::{listen, CloseCode, Handler, Message, Result as WsResult, Sender};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Client {
    pub username: String,
    pub token: usize,
}

fn validate_username(username: &str) -> Result<(), String> {
    if username.len() > 39 {
        Err("username too long".to_string())
    } else {
        Ok(())
    }
}

#[derive(Clone)]
struct Server {
    out: Sender,
    clients: Rc<RefCell<HashMap<Token, Client>>>,
    current_player: Rc<RefCell<Option<Client>>>,
    current_player_index: Rc<Cell<usize>>,
    started: Rc<Cell<bool>>,
}

impl Server {
    fn unrecognised_msg() -> String {
        let resp = SendableMessage::Error {
            error: "Unrecognised message".to_string(),
        };
        serde_json::to_string(&resp).unwrap()
    }

    fn broadcast_players(&mut self) -> WsResult<()> {
        let clients_map = self.clients.borrow();
        let usernames: Vec<Client> = clients_map.values().cloned().collect();
        let resp = serde_json::to_string(&SendableMessage::Players { players: usernames }).unwrap();
        self.out.broadcast(resp)
    }

    fn choose_player(&self) -> Client {
        if !self.started.get() {
            panic!("The game is not started, cannot choose player yet!!!");
        } else if self.clients.borrow().len() == 0 {
            panic!("Can not choose player as there are not clients connected");
        }

        // Select a name
        let clients = self.clients.borrow();
        let players: Vec<&Client> = clients.values().collect();
        let player = players[self.current_player_index.get()].clone();

        // Increment current player index.
        self.current_player_index
            .set(self.current_player_index.get() + 1);
        if self.current_player_index.get() >= self.clients.borrow().len() {
            self.current_player_index.set(0);
        }

        player
    }

    fn start_game(&mut self) -> WsResult<()> {
        self.started.set(true);
        let player = self.choose_player();
        self.current_player.replace(Some(player.clone()));
        self.out.broadcast(SendableMessage::Turn {
            username: player.username,
        })
    }

    fn handle_message(&mut self, msg: &ReceivableMessage) -> WsResult<()> {
        match msg {
            ReceivableMessage::Login { username: ref u } => {
                if let Err(e) = validate_username(&u) {
                    let err_msg =
                        serde_json::to_string(&SendableMessage::Error { error: e }).unwrap();
                    self.out.send(err_msg)
                } else if !self.clients.borrow().contains_key(&self.out.token()) {
                    self.clients.borrow_mut().insert(
                        self.out.token(),
                        Client {
                            username: u.clone(),
                            token: self.out.token().0,
                        },
                    );
                    self.broadcast_players()?;
                    self.out
                        .send(serde_json::to_string(&SendableMessage::LoggedIn).unwrap())?;
                    if !self.started.get() {
                        self.start_game()?;
                    } else {
                        if let Some(ref p) = *self.current_player.borrow() {
                            self.out.send(SendableMessage::Turn {
                                username: p.username.clone(),
                            })?;
                        }
                    }
                    Ok(())
                } else {
                    self.out.send(SendableMessage::Error {
                        error: "User is already registerred".to_string(),
                    })
                }
            }
            _ => self.out.send(Server::unrecognised_msg()),
        }
    }
}

impl Handler for Server {
    // Read the message and parse it to one of the ReceivableMessage types.
    // Then examine the message and respond with the Approperate SendableMessage type.
    fn on_message(&mut self, msg: Message) -> WsResult<()> {
        println!("Received message: {}", msg);
        println!("{:#?}", self.clients.borrow());

        match self.clients.borrow().get(&self.out.token()) {
            Some(s) => println!("Have gotten messages from {} before", s.username),
            _ => println!("This is a new user"),
        };

        match msg {
            Text(s) => match serde_json::from_str::<ReceivableMessage>(&s) {
                Ok(rmsg) => self.handle_message(&rmsg),
                _ => self.out.send(Server::unrecognised_msg()),
            },
            _ => self.out.send(Server::unrecognised_msg()),
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        // The WebSocket protocol allows for a utf8 reason for the closing state after the
        // close code. WS-RS will attempt to interpret this data as a utf8 description of the
        // reason for closing the connection. I many cases, `reason` will be an empty string.
        // So, you may not normally want to display `reason` to the user,
        // but let's assume that we know that `reason` is human-readable.
        println!("Removing player...");
        self.clients.borrow_mut().remove(&self.out.token());
        self.broadcast_players().unwrap();
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        }
    }
}

fn main() {
    let clients = Rc::new(RefCell::new(HashMap::new()));
    let current_player = Rc::new(RefCell::new(None));
    let current_player_index = Rc::new(Cell::new(0));
    let started = Rc::new(Cell::new(false));
    listen("127.0.0.1:8080", |out| Server {
        out,
        clients: clients.clone(),
        current_player: current_player.clone(),
        current_player_index: current_player_index.clone(),
        started: started.clone(),
    }).unwrap()
}
