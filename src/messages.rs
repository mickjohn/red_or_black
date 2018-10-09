use game::Client;
use serde_json;
use ws::Message;
use deck;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum CardColour {
    Red,
    Black,
}

#[derive(Deserialize, Serialize)]
pub enum ReceivableMessage {
    Login { username: String },
    Guess { card_colour: CardColour },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "msg_type")]
pub enum SendableMessage {
    Ok {
        msg: String,
    },
    Players {
        players: Vec<Client>,
    },
    Turn {
        username: String,
    },
    Error {
        error: String,
    },
    LoggedIn,
    GuessResult {
        correct: bool,
        card: deck::Card,
        penalty: u16,
        username: String,
    },
    CorrectGuess {
        drinking_seconds: u16,
        username: String,
    },
    WrongGuess {
        drinking_seconds: u16,
        username: String,
    },
    PlayerHasLeft {
        username: String,
    },
}

impl From<SendableMessage> for Message {
    fn from(s: SendableMessage) -> Message {
        Message::text(serde_json::to_string(&s).unwrap())
    }
}

impl<'a> From<&'a SendableMessage> for Message {
    fn from(s: &'a SendableMessage) -> Message {
        Message::text(serde_json::to_string(s).unwrap())
    }
}

#[cfg(test)]
impl From<ReceivableMessage> for Message {
    fn from(s: ReceivableMessage) -> Message {
        Message::text(serde_json::to_string(&s).unwrap())
    }
}

#[cfg(test)]
impl<'a> From<&'a ReceivableMessage> for Message {
    fn from(s: &'a ReceivableMessage) -> Message {
        Message::text(serde_json::to_string(s).unwrap())
    }
}
