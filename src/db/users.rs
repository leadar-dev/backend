use sqlx::PgPool;

use crate::errors::AppResult;
use crate::models::user::UserRow;

pub async fn upsert_user(
    pool: &PgPool,
    telegram_id: i64,
    first_name: Option<&str>,
    username: Option<&str>,
    role: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO users (telegram_id, role, is_active, first_name, username, last_login)
         VALUES ($1, $2, true, $3, $4, now())
         ON CONFLICT (telegram_id) DO UPDATE SET
             role       = $2,
             is_active  = true,
             first_name = COALESCE($3, users.first_name),
             username   = COALESCE($4, users.username),
             last_login = now(),
             updated_at = now()",
    )
    .bind(telegram_id)
    .bind(role)
    .bind(first_name)
    .bind(username)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_active_user(pool: &PgPool, telegram_id: i64) -> AppResult<Option<UserRow>> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT telegram_id, first_name, username, role, is_active, created_at, last_login
         FROM users
         WHERE telegram_id = $1 AND is_active = true",
    )
    .bind(telegram_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn fetch_user(pool: &PgPool, telegram_id: i64) -> AppResult<Option<UserRow>> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT telegram_id, first_name, username, role, is_active, created_at, last_login
         FROM users
         WHERE telegram_id = $1",
    )
    .bind(telegram_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
