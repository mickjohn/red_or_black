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
mod messages;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use ws::Message::*;
use ws::{listen, CloseCode, Handler, Message, Result as WsResult, Sender};

use deck::Card;
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
        // let usernames: Vec<Client> = game_state.get_clients().values().cloned().collect();
        let usernames: Vec<Client> = game_state.get_clients_vec().iter().cloned().collect();
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

    fn validate_guess(guess: &CardColour, card: Card) -> bool {
        use deck::Suit;
        guess == &CardColour::Black && (card.suit == Suit::Spade || card.suit == Suit::Club)
            || guess == &CardColour::Red && (card.suit == Suit::Heart || card.suit == Suit::Diamond)
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
                // Check that the guess is coming from the 'current_player'
                let (client, current_player) = {
                    let game_state = self.game_state.borrow();
                    let clients = game_state.get_clients();
                    let client = clients.get(&self.out.token());
                    if let (Some(cp), Some(c)) = (game_state.get_current_player(), client) {
                        if cp != c {
                            info!("It's not {}'s turn, ignoring guess.", c.username);
                            return self.out.send(SendableMessage::Error { error: "It's not this players turn".to_string()});
                        }
                    }
                    let username = match client {
                        Some(ref c) => c.username.clone(),
                        None => "<no username>".to_string(),
                    };
                    (username, game_state.get_current_player().clone())
                };

                let mut game_state = self.game_state.borrow_mut();
                let card = game_state.get_card();
                debug!("Card drawn from deck = {:?}", card);
                debug!("Guess from user = {:?}", card_colour);
                if Server::validate_guess(&card_colour, card) {
                    info!("{} guessed correctly", client);
                    let drinking_seconds = game_state.increment_drinking_seconds();
                    self.out.broadcast(SendableMessage::CorrectGuess { drinking_seconds, username: current_player.unwrap().username } )?;
                } else {
                    info!("{} guessed incorrectly", client);
                    self.out.broadcast(SendableMessage::WrongGuess { drinking_seconds: game_state.get_drinking_seconds(), username: current_player.unwrap().username })?;
                    game_state.reset_drinking_seconds();
                }
                let next_player = game_state.next_player();
                info!("It is now {}'s turn", next_player.username);
                self.out.broadcast(SendableMessage::Turn {
                    username: next_player.username.clone(),
                })
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

        // Broadcast the name of the player who is leaving
        if let Some(c) = self
            .game_state
            .borrow()
            .get_clients()
            .get(&self.out.token())
        {
            info!("Removing player {}", c.username);
            self.out
                .broadcast(SendableMessage::PlayerHasLeft {
                    username: c.username.clone(),
                }).unwrap();
        }

        // remove the player, and if it's their go, then move to next player broadcast that the current player has
        // changed.
        {
            let mut game_state = self.game_state.borrow_mut();
            debug!("Removing player");
            if game_state.remove_client(self.out.token()) {
                let player = game_state.get_current_player();
                if let Some(p) = player {
                    self.out
                        .broadcast(SendableMessage::Turn {
                            username: p.username.clone(),
                        }).unwrap();
                }
            }
        }

        self.broadcast_players().unwrap();

        match code {
            CloseCode::Normal => info!("The client is done with the connection."),
            CloseCode::Away => info!("The client is leaving the site."),
            _ => error!("Close code: {:?}, reason: {}", code, reason),
        }
    }
}

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "websocket_red_or_black=debug");
    }
    env_logger::init();
    info!("Starting up!");
    let game_state = Rc::new(RefCell::new(GameState::new()));
    // listen("127.0.0.1:8000", |out| Server {
    listen("0.0.0.0:8000", |out| Server {
        out,
        game_state: game_state.clone(),
    }).unwrap()
}
