use axum::{extract::Request, middleware::Next, response::Response};
use axum_extra::extract::CookieJar;
use sqlx::PgPool;

use crate::db::users as users_db;
use crate::errors::AppError;
use crate::models::auth::AuthUser;
use crate::services::auth as auth_service;

pub async fn require_auth_with_secret(
    jar: CookieJar,
    jwt_secret: String,
    pool: PgPool,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = jar
        .get("token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let claims = auth_service::verify_jwt(&token, &jwt_secret)?;
    let telegram_id: i64 = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

    let user = users_db::fetch_active_user(&pool, telegram_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    req.extensions_mut().insert(AuthUser {
        telegram_id,
        role: user.role,
    });

    Ok(next.run(req).await)
}
