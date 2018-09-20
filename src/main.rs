extern crate rand;
extern crate ws;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate env_logger;
#[macro_use]
extern crate log;

mod deck;
mod game;
mod messages;

use std::cell::RefCell;
use std::rc::Rc;
use std::env;
use ws::Message::*;
use ws::{listen, CloseCode, Handler, Message, Result as WsResult, Sender};

use game::GameState;
use messages::*;

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
    game_state: Rc<RefCell<GameState>>,
}

impl Server {
    fn unrecognised_msg() -> String {
        let resp = SendableMessage::Error {
            error: "Unrecognised message".to_string(),
        };
        serde_json::to_string(&resp).unwrap()
    }

    fn broadcast_players(&mut self) -> WsResult<()> {
        let game_state = self.game_state.borrow();
        let usernames: Vec<Client> = game_state.get_clients().values().cloned().collect();
        self.out
            .broadcast(SendableMessage::Players { players: usernames })
    }

    fn start_game(&mut self) -> WsResult<()> {
        let mut game_state = self.game_state.borrow_mut();
        game_state.started = true;
        let player = game_state.next_player();
        self.out.broadcast(SendableMessage::Turn {
            username: player.username.clone(),
        })
    }

    fn validate_guess(guess: &CardColour, card: &Card) -> bool {
        if card_colour == &CardColour::Black
            && (card.suit == Suit::Spade || card.suit == Suit::Club)
            {
                info!("user guessed correctly");
                true
            }
        else if card_colour == &CardColour::Red
            && (card.suit == Suit::Heart || card.suit == Suit::Diamond)
            {
                info!("user guessed correctly");
                true
            } else {
                info!("incorrect guess");
                false
            }
    }

    fn handle_message(&mut self, msg: &ReceivableMessage) -> WsResult<()> {
        match msg {
            ReceivableMessage::Login { username: ref u } => {
                if let Err(e) = validate_username(&u) {
                    self.out.send(SendableMessage::Error { error: e })
                } else if !self
                    .game_state
                    .borrow()
                    .get_clients()
                    .contains_key(&self.out.token())
                {
                    self.game_state.borrow_mut().add_client(
                        self.out.token(),
                        Client {
                            username: u.clone(),
                            token: self.out.token().0,
                        },
                    );
                    self.broadcast_players()?;
                    self.out.send(SendableMessage::LoggedIn)?;
                    if !self.game_state.borrow().started {
                        self.start_game()?;
                    } else if let Some(ref p) = *self.game_state.borrow().get_current_player() {
                        self.out.send(SendableMessage::Turn {
                            username: p.username.clone(),
                        })?;
                    }
                    Ok(())
                } else {
                    self.out.send(SendableMessage::Error {
                        error: "User is already registerred".to_string(),
                    })
                }
            }
            ReceivableMessage::Guess { card_colour } => {
                use deck::Suit;
                let mut game_state = self.game_state.borrow_mut();
                let card = game_state.get_card();
                debug!("Card drawn from deck = {:?}", card);
                debug!("Guess from user = {:?}", card_colour);
                if card_colour == &CardColour::Black
                    && (card.suit == Suit::Spade || card.suit == Suit::Club)
                {
                    info!("user guessed correctly");
                    self.out.send(SendableMessage::CorrectGuess)
                }
                else if card_colour == &CardColour::Red
                    && (card.suit == Suit::Heart || card.suit == Suit::Diamond)
                {
                    info!("user guessed correctly");
                    self.out.send(SendableMessage::CorrectGuess)
                } else {
                    info!("incorrect guess");
                    self.out.send(SendableMessage::WrongGuess)
                }
            }
            // _ => self.out.send(Server::unrecognised_msg()),
        }
    }
}

impl Handler for Server {
    // Read the message and parse it to one of the ReceivableMessage types.
    // Then examine the message and respond with the Approperate SendableMessage type.
    fn on_message(&mut self, msg: Message) -> WsResult<()> {
        debug!("Received message: {}", msg);
        // println!("{:#?}", self.clients.borrow());

        match self
            .game_state
            .borrow()
            .get_clients()
            .get(&self.out.token())
        {
            Some(s) => debug!("Have gotten messages from {} before", s.username),
            _ => debug!("This is a new user"),
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
        self.game_state.borrow_mut().remove_client(self.out.token());
        self.broadcast_players().unwrap();
        match code {
            CloseCode::Normal => info!("The client is done with the connection."),
            CloseCode::Away => info!("The client is leaving the site."),
            _ => error!("The client encountered an error: {}", reason),
        }
    }
}

fn main() {
    env::set_var("RUST_LOG", "websocket_red_or_black=debug");
    env_logger::init();
    info!("Starting up!");
    let game_state = Rc::new(RefCell::new(GameState::new()));
    listen("127.0.0.1:8080", |out| Server {
        out,
        game_state: game_state.clone(),
    }).unwrap()
}
