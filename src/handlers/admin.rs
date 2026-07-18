use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use tracing::instrument;

use crate::db::users as users_db;
use crate::errors::{AppError, AppResult};
use crate::models::auth::AuthUser;
use crate::models::user::UserAdminResponse;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccessBody {
    pub is_active: bool,
}

#[instrument(skip(state, _user), fields(page = ?query.page, limit = ?query.limit))]
pub async fn get_admin_users(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Query(query): Query<PaginationQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let rows = users_db::list_users(&state.pool, limit, offset).await?;
    let data: Vec<UserAdminResponse> = rows.into_iter().map(UserAdminResponse::from).collect();

    Ok(Json(json!({ "ok": true, "data": data })))
}

#[instrument(skip(state, user), fields(target_id = telegram_id))]
pub async fn patch_admin_user_access(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(telegram_id): Path<i64>,
    Json(body): Json<UpdateAccessBody>,
) -> AppResult<Json<serde_json::Value>> {
    if user.telegram_id == telegram_id {
        return Err(AppError::InvalidRequest(
            "cannot change your own access".into(),
        ));
    }

    let updated = users_db::update_user_access(&state.pool, telegram_id, body.is_active).await?;
    if !updated {
        return Err(AppError::InvalidRequest(
            format!("user {telegram_id} not found"),
        ));
    }

    Ok(Json(json!({ "ok": true, "data": null })))
}
