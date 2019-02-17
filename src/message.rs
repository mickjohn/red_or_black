use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct IncomingMessage<T: Deserialize + Serialize> {
    pub user_id: String,
    pub game_id: String,
    pub message: T
}
