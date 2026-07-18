use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub telegram_id: i64,
    pub first_name: Option<String>,
    pub username: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct UserMeResponse {
    pub telegram_id: i64,
    pub first_name: Option<String>,
    pub username: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<UserRow> for UserMeResponse {
    fn from(row: UserRow) -> Self {
        Self {
            telegram_id: row.telegram_id,
            first_name: row.first_name,
            username: row.username,
            role: row.role,
            created_at: row.created_at,
            last_login: row.last_login,
        }
    }
}
