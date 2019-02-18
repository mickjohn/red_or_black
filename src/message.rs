use serde_json::Value;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct IncomingMessage {
    pub user_id: String,
    pub game_id: String,
    pub message: Value,
}