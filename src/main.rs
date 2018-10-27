extern crate rand;
extern crate ws;
#[macro_use]
extern crate serde_derive;
extern crate env_logger;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate openssl;

mod deck;
mod game;
mod messages;
mod red_or_black;

use openssl::pkey::PKey;
use openssl::ssl::{SslAcceptor, SslMethod, SslStream};
use openssl::x509::X509;
use ws::listen;

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

static ADDRESS: &'static str = "127.0.0.1";
static PORT: &'static str = "9000";
static SSL_CERT_PATH: &'static str = "ssl/cert.pem";
static SSL_KEY_PATH: &'static str = "ssl/key.pem";

fn main() {
    let game = Rc::new(RefCell::new(red_or_black::RedOrBlack::new(Vec::new())));
    let clients = Rc::new(RefCell::new(HashMap::new()));
    let ip_and_port = format!("{}:{}", ADDRESS, PORT);

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "websocket_red_or_black=debug");
    }
    env_logger::init();

    info!("Loading cert from {}", SSL_CERT_PATH);
    let cert = {
        let data = read_file(SSL_CERT_PATH).unwrap();
        X509::from_pem(data.as_ref()).unwrap()
    };

    info!("Loading key from {}", SSL_KEY_PATH);
    let pkey = {
        let data = read_file(SSL_KEY_PATH).unwrap();
        PKey::private_key_from_pem(data.as_ref()).unwrap()
    };

    let acceptor = Rc::new({
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder.set_private_key(&pkey).unwrap();
        builder.set_certificate(&cert).unwrap();
        builder.build()
    });

    info!("Starting up on {}", ip_and_port);
    ws::Builder::new()
        .with_settings(ws::Settings {
            encrypt_server: true,
            ..ws::Settings::default()
        }).build(|out: ws::Sender| game::Server {
            out: out,
            game: game.clone(),
            clients: clients.clone(),
            ssl: acceptor.clone(),
        }).unwrap()
        .listen(ip_and_port)
        .unwrap();
}

fn read_file(name: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(name)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}
