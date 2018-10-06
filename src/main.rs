extern crate rand;
extern crate ws;
#[macro_use]
extern crate serde_derive;
extern crate env_logger;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;

mod deck;
mod game;
mod game2;
mod messages;

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::sync::Mutex;
use ws::Message::*;
use ws::{listen, CloseCode, Handler, Message, Result as WsResult, Sender};

use deck::Card;
use messages::*;

// impl Handler for Server {
//     // Read the message and parse it to one of the ReceivableMessage types.
//     // Then examine the message and respond with the Approperate SendableMessage type.
//     fn on_message(&mut self, msg: Message) -> WsResult<()> {
//         debug!("Received message: {}", msg);

//         match self
//             .game_state
//             .borrow()
//             .get_clients()
//             .get(&self.out.token())
//         {
//             Some(s) => debug!("Have gotten messages from {} before", s.username),
//             _ => debug!("This is a new user"),
//         };

//         match msg {
//             Text(s) => match serde_json::from_str::<ReceivableMessage>(&s) {
//                 Ok(rmsg) => self.handle_message(&rmsg),
//                 _ => self.out.send(Server::unrecognised_msg()),
//             },
//             _ => self.out.send(Server::unrecognised_msg()),
//         }
//     }

//     fn on_close(&mut self, code: CloseCode, reason: &str) {
//         // The WebSocket protocol allows for a utf8 reason for the closing state after the
//         // close code. WS-RS will attempt to interpret this data as a utf8 description of the
//         // reason for closing the connection. I many cases, `reason` will be an empty string.
//         // So, you may not normally want to display `reason` to the user,
//         // but let's assume that we know that `reason` is human-readable.

//         // Broadcast the name of the player who is leaving
//         if let Some(c) = self
//             .game_state
//             .borrow()
//             .get_clients()
//             .get(&self.out.token())
//         {
//             info!("Removing player {}", c.username);
//             self.out
//                 .broadcast(SendableMessage::PlayerHasLeft {
//                     username: c.username.clone(),
//                 }).unwrap();
//         }

//         // remove the player, and if it's their go, then move to next player broadcast that the current player has
//         // changed.
//         {
//             let mut game_state = self.game_state.borrow_mut();
//             debug!("Removing player");
//             if game_state.remove_client(self.out.token()) {
//                 let player = game_state.get_current_player();
//                 if let Some(p) = player {
//                     self.out
//                         .broadcast(SendableMessage::Turn {
//                             username: p.username.clone(),
//                         }).unwrap();
//                 }
//             }
//         }

//         self.broadcast_players().unwrap();

//         match code {
//             CloseCode::Normal => info!("The client is done with the connection."),
//             CloseCode::Away => info!("The client is leaving the site."),
//             _ => error!("Close code: {:?}, reason: {}", code, reason),
//         }
//     }
// }

fn main() {
    let game = Rc::new(RefCell::new(game2::RedOrBlack::new(Vec::new())));
    let clients = Rc::new(RefCell::new(HashMap::new()));

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "websocket_red_or_black=debug");
    }
    env_logger::init();
    info!("Starting up!");
    // listen("127.0.0.1:8000", |out| Server {
    listen("0.0.0.0:8000", |out| game::Server {
        out,
        game: game.clone(),
        clients: clients.clone(),
    }).unwrap()
}
