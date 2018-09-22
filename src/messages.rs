use super::Client;
use deck::Card;
use serde_json;
use ws::Message;

#[derive(Debug, PartialEq, Deserialize)]
pub enum CardColour {
    Red,
    Black,
}

#[derive(Deserialize)]
pub enum ReceivableMessage {
    Login { username: String },
    Guess { card_colour: CardColour },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "msg_type")]
pub enum SendableMessage {
    Ok { msg: String },
    Players { players: Vec<Client> },
    Turn { username: String },
    Error { error: String },
    LoggedIn,
    CorrectGuess,
    WrongGuess,
    PlayerHasLeft { username: String },
}

impl From<SendableMessage> for Message {
    fn from(s: SendableMessage) -> Message {
        Message::text(serde_json::to_string(&s).unwrap())
    }
}
