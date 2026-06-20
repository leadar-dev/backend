use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct CategoryRow {
    pub id: i32,
    pub source: String,
    pub external_id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub created_at: DateTime<Utc>,
}
