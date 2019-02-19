use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use ws::Message::*;
use ws::{Sender, Handler, Message, Result as WsResult, listen};
use serde_json::Value;

use crate::message::IncomingMessage;

pub struct DummyGame {
    id: String,
    name: String,
}

impl Game for DummyGame {
    fn handle_message(&mut self, message: Value) {
        println!("Handling message");
        println!("This is {} with ID {}. Message is {}", self.get_name(), self.get_id(), message);
    }

    fn get_name(&self) -> &str {
        &self.name
    }
     
    fn get_id(&self) -> &str {
        &self.id
    }
}

pub trait Game {
    fn handle_message(&mut self, message: Value);
    fn get_name(&self) -> &str;
    fn get_id(&self) -> &str;
}

pub struct GameServer {
    // port: u16,
    // address: String,
    games: HashMap<String, Box<Game>>,
    max_games: u64,
    pub out: Sender,
}

impl GameServer {
    // pub fn new(port: u16, address: String, max_games: u64) -> Self {
    //     GameServer {
    //         port,
    //         address,
    //         max_games,
    //         games: HashMap::new(),
    //     }
    // }

    pub fn start() {
        listen("127.0.0.1:8080", |out| {
            let mut gs = GameServer {
                out,
                max_games: 10,
                games: HashMap::new(),
            };
            println!("game id 1 {}", gs.add_game());
            println!("game id 2 {}", gs.add_game());
            gs
        }).unwrap()
    }

    pub fn add_game(&mut self) -> String {
        let game_id: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .collect();
        info!("Adding new dummy game with ID: {}", game_id);
        let g = DummyGame {
            id: game_id.clone(),
            name: "DummyGame".to_string(),
        };

        self.games.insert(game_id.clone(), Box::new(g));
        game_id
    }

    pub fn forward_message_to_game(&mut self, in_msg: IncomingMessage) {
        if let Some(game_t) = self.games.get_mut(&in_msg.game_id) {
            game_t.handle_message(in_msg.message);
        } else {
            warn!("No game with id {}", in_msg.game_id)
        }
    }
}

impl Handler for GameServer {
    fn on_message(&mut self, msg: Message) -> WsResult<()> {
        debug!("Received message: {}", msg);
        match msg {
            Text(s) => match serde_json::from_str::<IncomingMessage>(&s) {
                Ok(in_msg) => {
                    self.forward_message_to_game(in_msg);
                },
                _ => {
                    warn!("Failed to deserialize incoming message. Message was {}", &s);
                }
            },
            _ => ()
        };
        Ok(())
    }

//     fn on_close(&mut self, code: CloseCode, reason: &str) {
//         // The WebSocket protocol allows for a utf8 reason for the closing state after the
//         // close code. WS-RS will attempt to interpret this data as a utf8 description of the
//         // reason for closing the connection. I many cases, `reason` will be an empty string.
//         // So, you may not normally want to display `reason` to the user,
//         // but let's assume that we know that `reason` is human-readable.

//         self.remove_client();
//         match code {
//             CloseCode::Normal => info!("The client is done with the connection."),
//             CloseCode::Away => info!("The client is leaving the site."),
//             _ => error!("Close code: {:?}, reason: {}", code, reason),
//         }
//     }
}


#[cfg(test)]
mod test {

}