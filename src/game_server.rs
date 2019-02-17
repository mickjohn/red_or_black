use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use ws::Message::*;
use ws::{CloseCode, Handler, Message, Result as WsResult, Sender};

pub struct DummyGame;

impl Game for DummyGame {
    fn handle_message(&mut self) {
        println!("Handling message");
    }
}

pub trait Game {
    fn handle_message(&mut self);
}

// #[derive(Debug)]
pub struct GameServer<'a> {
    port: u16,
    address: &'a str,
    games: HashMap<String, Box<Game>>,
    max_games: u64,
}

impl<'a> GameServer <'a> {
    pub fn new(port: u16, address: &'a str, max_games: u64) -> Self {
        GameServer {
            port,
            address,
            max_games,
            games: HashMap::new(),
        }
    }

    pub fn add_game(&'a mut self) -> String {
        let game_id: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .collect();
        info!("Adding new dummy game with ID: {}", game_id);
        self.games.insert(game_id.clone(), Box::new(DummyGame));
        game_id.clone()
    }
}

// impl<'a> Handler for GameServer<'a> {
//     fn on_message(&mut self, msg: Message) -> WsResult<()> {
//         debug!("Received message: {}", msg);
//         match msg {
//             Text(s) => match serde_json::from_str::<ReceivableMessage>(&s) {
//                 Ok(rmsg) => self.handle_message(&rmsg),
//                 _ => self.out.send(Server::unrecognised_msg()).unwrap(),
//             },
//             _ => self.out.send(Server::unrecognised_msg()).unwrap(),
//         };
//         Ok(())
//     }

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
// }


#[cfg(test)]
mod test {

}