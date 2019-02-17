mod game;
mod history;
pub mod messages;
mod rules;

use self::game::Server;
use self::rules::RedOrBlack;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ws::listen;

pub fn start_server(ip_and_port: &str) {
    let game = Rc::new(RefCell::new(RedOrBlack::new(Vec::new())));
    let clients = Rc::new(RefCell::new(HashMap::new()));
    info!("Starting up on {}", ip_and_port);
    listen(ip_and_port, |out| Server {
        out,
        game: game.clone(),
        clients: clients.clone(),
    }).unwrap()
}
