use deck;
use deck::Card;
use game::Client;
use red_or_black::HistoryItem;
use serde_json;
use std::collections::VecDeque;
use ws::Message;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum CardColour {
    Red,
    Black,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ReceivableMessage {
    Login { username: String },
    Guess { card_colour: CardColour },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
        guess: CardColour,
    },
    Penalty {
        penalty: u16,
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
    RequestHistory {
        history: VecDeque<Option<Card>>,
    },
    GameHistory {
        history: Vec<HistoryItem>,
    },
    CardsLeft {
        cards_left: usize,
    },
}

impl From<SendableMessage> for Message {
    fn from(s: SendableMessage) -> Message {
        Message::text(serde_json::to_string(&s).unwrap())
    }
}

impl<'a> From<&'a SendableMessage> for Message {
    fn from(s: &SendableMessage) -> Message {
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
