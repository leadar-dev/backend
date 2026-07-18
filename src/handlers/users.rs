use axum::{extract::State, Extension, Json};
use serde_json::json;
use tracing::instrument;

use crate::db::users as users_db;
use crate::errors::{AppError, AppResult};
use crate::models::auth::AuthUser;
use crate::models::user::UserMeResponse;
use crate::AppState;

#[instrument(skip(state), fields(telegram_id = user.telegram_id))]
pub async fn get_users_me(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let row = users_db::fetch_user(&state.pool, user.telegram_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let response = UserMeResponse::from(row);
    Ok(Json(json!({ "ok": true, "data": response })))
}
