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
mod red_or_black;

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use ws::listen;

static ADDRESS: &'static str = "127.0.0.1";
static PORT: &'static str = "9000";

fn main() {
    let game = Rc::new(RefCell::new(red_or_black::RedOrBlack::new(Vec::new())));
    let clients = Rc::new(RefCell::new(HashMap::new()));
    let ip_and_port = format!("{}:{}", ADDRESS, PORT);

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "websocket_red_or_black=debug");
    }
    env_logger::init();
    info!("Starting up on {}", ip_and_port);
    listen(ip_and_port, |out| game::Server {
        out,
        game: game.clone(),
        clients: clients.clone(),
    }).unwrap()
}
