use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramAuthData {
    pub id: i64,
    pub first_name: String,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    pub auth_date: i64,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    #[allow(dead_code)]
    pub telegram_id: i64,
}
