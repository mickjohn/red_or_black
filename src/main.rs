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

fn main() {
    let game = Rc::new(RefCell::new(red_or_black::RedOrBlack::new(Vec::new())));
    let clients = Rc::new(RefCell::new(HashMap::new()));

    // Set rustlog to debug if it's not set
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "websocket_red_or_black=debug");
    }

    // Read config from env vars
    let address = env::var("RED_OR_BLACK_WEBSERVER_ADDRESS").unwrap_or("127.0.0.1".to_string());
    let port = env::var("RED_OR_BLACK_WEBSERVER_PORT").unwrap_or("9000".to_string());
    let ip_and_port = format!("{}:{}", address, port);

    env_logger::init();
    info!("Starting up on {}", ip_and_port);
    listen(ip_and_port, |out| game::Server {
        out,
        game: game.clone(),
        clients: clients.clone(),
    }).unwrap()
}
