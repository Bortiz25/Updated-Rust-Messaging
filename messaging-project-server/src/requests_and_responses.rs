use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserResponse {
    pub user_id: i32,
    pub username: String,
}

#[derive(Serialize)]
pub struct Chat {
    pub chat_id: i32,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub chat_id: i32,
    pub with: String,
    pub last_message: String,
}

#[derive(Serialize)]
pub struct Message {
    pub message_id: i32,
    pub chat_id: i32,
    pub sent_from: i32,
    pub message: String,
}

pub struct ExistsQuery {
    pub exists: Option<bool>,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct CreateMessageGCRequestBody {
    pub message: String,
    pub buddies: Vec<String>,
}

#[derive(Deserialize)]
pub struct LoginRequestBody {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateChatRequestBody {
    pub buddy_id: String,
}

#[derive(Deserialize)]
pub struct CreateMessageRequestBody {
    pub message: String,
}
