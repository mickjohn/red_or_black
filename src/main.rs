extern crate ws;
extern crate rand;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

use ws::{listen, Handler, Sender, Result as WsResult, Message, CloseCode};
use ws::Message::*;
use ws::util::Token;
use std::collections::HashMap;
use std::cell::{Cell, RefCell};
use std::rc::Rc;


#[derive(Debug, Clone, PartialEq)]
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
    clients: Rc<HashMap<Token, Client>>,
}

#[derive(Deserialize, Serialize)]
enum ReceivableMessage {
    RequestId{ username: String },
    GameMove{ id: u64, choice: u8 },
}

#[derive(Deserialize, Serialize)]
enum SendableMessage {
    Ok{ msg: String },
    Id{ id: u64 },
    Players{ players: Vec<String> },
    Turn { username: String },
    Error{ error: String },
}

impl Server {
    fn unrecognised_msg() -> String {
        let resp = SendableMessage::Error{ error: "Unrecognised message".to_string() };
        serde_json::to_string(&resp).unwrap()
    }

    fn handle_message(&mut self, msg: &ReceivableMessage) -> WsResult<()> {
        let clients = &self.clients;
        match &msg {
            &ReceivableMessage::RequestId{ username: ref u } => {
                clients.insert(self.out.token(), Client{ username: u.clone()});
                let sble = SendableMessage::Ok{ msg: "Got username!".to_string() };
                let resp = serde_json::to_string(&sble).unwrap();
                println!("Responding with {}", resp);
                self.out.send(resp)
            },
            _ => self.out.send(Server::unrecognised_msg()),
        }
    }
}

impl Handler for Server {

    // Read the message and parse it to one of the ReceivableMessage types.
    // Then examine the message and respond with the Approperate SendableMessage type.
    fn on_message(&mut self, msg: Message) -> WsResult<()> {
        println!("Received message: {}", msg);
        println!("Token: {:?}", self.out.token());
        // {
        //     let clients = self.clients.lock().unwrap();
        //     println!("{:#?}", *clients);
        //     match clients.get(&self.out.token()) {
        //         Some(s) => println!("Have gotten messages from {} before", s.username),
        //         _ => println!("This is a new user"),
        //     };
        // }
        // let mut clients = self.clients.get_mut();
        match self.clients.get(&self.out.token()) {
            Some(s) => println!("Have gotten messages from {} before", s.username),
            _ => println!("This is a new user"),
        };

        match msg {
            Text(s) => {
                match serde_json::from_str::<ReceivableMessage>(&s) {
                    Ok(rmsg) => {
                        self.handle_message(&rmsg)
                    },
                    _ => self.out.send(Server::unrecognised_msg()),
                }
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
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away   => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        }
    }
}

fn main() {
    let clients = Rc::new(HashMap::new());
    // listen("127.0.0.1:8080", |out| Server { out: out, clients: Mutex::new(HashMap::new()) } ).unwrap()
    listen("127.0.0.1:8080", |out| Server { out: out, clients: clients.clone() } ).unwrap()
} 

