use serde_json::Value;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct IncomingMessage {
    pub user_id: String,
    pub game_id: String,
    pub message: Value,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct IncomingAdminMessage {
    pub user_id: String,
    pub game_id: String,
    pub auth_key: String,
    pub message: Value,
}