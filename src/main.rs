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
mod red_or_black;
mod message;

use std::env;

fn main() {
    // Set rustlog to debug if it's not set
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "websocket_red_or_black=debug");
    }

    env_logger::init();

    // Read config from env vars
    let address =
        env::var("RED_OR_BLACK_WEBSERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("RED_OR_BLACK_WEBSERVER_PORT").unwrap_or_else(|_| "9000".to_string());
    let ip_and_port = format!("{}:{}", address, port);
    env_logger::init();
    red_or_black::start_server(&ip_and_port);
}
