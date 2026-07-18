use axum::{extract::State, Json};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use serde_json::json;
use time::Duration;
use tracing::{info, instrument, warn};

use crate::db::users as users_db;
use crate::errors::{AppError, AppResult};
use crate::models::auth::TelegramAuthData;
use crate::services::auth as auth_service;
use crate::AppState;

#[instrument(skip(state, jar, body), fields(telegram_id = body.id))]
pub async fn post_auth_telegram(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<TelegramAuthData>,
) -> AppResult<(CookieJar, Json<serde_json::Value>)> {
    let now = Utc::now().timestamp();
    if now - body.auth_date > 86400 {
        warn!(telegram_id = body.id, "auth_date too old");
        return Err(AppError::InvalidRequest("auth_date is too old".into()));
    }

    if !auth_service::verify_telegram_auth(&body, &state.config.auth.bot_token) {
        warn!(telegram_id = body.id, "telegram auth verification failed");
        return Err(AppError::Unauthorized);
    }

    let allowed = state.config.allowed_telegram_ids();
    let role = if allowed.contains(&body.id) {
        "admin"
    } else {
        let existing = users_db::fetch_active_user(&state.pool, body.id).await?;
        if existing.is_none() {
            warn!(telegram_id = body.id, "user not in DB or not active");
            return Err(AppError::Unauthorized);
        }
        "user"
    };

    users_db::upsert_user(
        &state.pool,
        body.id,
        Some(&body.first_name),
        body.username.as_deref(),
        role,
    )
    .await?;

    let token = auth_service::create_jwt(
        body.id,
        &state.config.auth.jwt_secret,
        state.config.auth.jwt_expiry_hours,
    )?;

    let cookie = Cookie::build(("token", token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::hours(state.config.auth.jwt_expiry_hours))
        .path("/")
        .build();

    info!(telegram_id = body.id, role, "user authenticated");

    Ok((
        jar.add(cookie),
        Json(json!({ "ok": true, "data": { "telegram_id": body.id } })),
    ))
}

pub async fn post_auth_logout(jar: CookieJar) -> (CookieJar, Json<serde_json::Value>) {
    let cookie = Cookie::build(("token", ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::seconds(0))
        .path("/")
        .build();

    (jar.remove(cookie), Json(json!({ "ok": true, "data": null })))
}
