use sqlx::PgPool;

use crate::errors::AppResult;
use crate::models::feature_flag::FeatureFlagRow;

pub async fn list_flags(pool: &PgPool) -> AppResult<Vec<FeatureFlagRow>> {
    let rows = sqlx::query_as::<_, FeatureFlagRow>(
        "SELECT name, enabled FROM feature_flags ORDER BY name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn update_flag(pool: &PgPool, name: &str, enabled: bool) -> AppResult<bool> {
    let result = sqlx::query(
        "UPDATE feature_flags SET enabled = $1, updated_at = now() WHERE name = $2",
    )
    .bind(enabled)
    .bind(name)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
