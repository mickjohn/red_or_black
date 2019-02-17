#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct IncomingMessage<T> {
    pub user_id: String,
    pub game_id: String,
    pub message: T
}