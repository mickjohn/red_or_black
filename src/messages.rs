use super::Client;
use serde_json;
use ws::Message;

#[derive(Deserialize, Serialize)]
pub enum ReceivableMessage {
    Login { username: String },
    GameMove { id: u64, choice: u8 },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "msg_type")]
pub enum SendableMessage {
    Ok { msg: String },
    Players { players: Vec<Client> },
    Turn { username: String },
    Error { error: String },
    LoggedIn,
}

impl From<SendableMessage> for Message {
    fn from(s: SendableMessage) -> Message {
        Message::text(serde_json::to_string(&s).unwrap())
    }
}
