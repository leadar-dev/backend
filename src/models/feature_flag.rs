use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct FeatureFlagRow {
    pub name: String,
    pub enabled: bool,
}
