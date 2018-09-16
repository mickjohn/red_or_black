extern crate rand;
extern crate ws;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ws::util::Token;
use ws::Message::*;
use ws::{listen, CloseCode, Handler, Message, Result as WsResult, Sender};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Client {
    pub username: String,
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
}

#[derive(Deserialize, Serialize)]
enum ReceivableMessage {
    RequestId { username: String },
    GameMove { id: u64, choice: u8 },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "msg_type")]
enum SendableMessage {
    Ok { msg: String },
    Players { players: Vec<Client> },
    Turn { username: String },
    Error { error: String },
    LoggedIn,
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

    fn handle_message(&mut self, msg: &ReceivableMessage) -> WsResult<()> {
        match &msg {
            &ReceivableMessage::RequestId { username: ref u } => {
                if let Err(e) = validate_username(&u) {
                    let err_msg =
                        serde_json::to_string(&SendableMessage::Error { error: e }).unwrap();
                    self.out.send(err_msg)
                } else {
                    if !self.clients.borrow().contains_key(&self.out.token()) {
                        self.clients.borrow_mut().insert(
                            self.out.token(),
                            Client {
                                username: u.clone(),
                            },
                        );
                        let resp = serde_json::to_string(&SendableMessage::LoggedIn).unwrap();
                        println!("Responding with {}", resp);
                        self.broadcast_players()?;
                        self.out.send(resp)
                    } else {
                        self.out.send(
                            serde_json::to_string(&SendableMessage::Error {
                                error: "User is already registerred".to_string(),
                            }).unwrap(),
                        )
                    }
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
    listen("127.0.0.1:8080", |out| Server {
        out: out,
        clients: clients.clone(),
    }).unwrap()
}
