use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use tracing::instrument;

use crate::errors::{AppError, AppResult};
use crate::models::auth::AuthUser;
use crate::models::want::WantResponse;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct WantsQuery {
    pub source: Option<String>,
    pub category_id: Option<i32>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[instrument(skip(state, _user), fields(source = ?query.source, category_id = ?query.category_id))]
pub async fn get_wants(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Query(query): Query<WantsQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    let rows = crate::db::wants::list_wants(&state.pool, &query, limit, offset).await?;
    let data: Vec<WantResponse> = rows.into_iter().map(WantResponse::from).collect();

    Ok(Json(json!({ "ok": true, "data": data })))
}

#[instrument(skip(state, _user))]
pub async fn get_want_by_id(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    let row = crate::db::wants::get_want_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::WantNotFound(id))?;

    Ok(Json(json!({ "ok": true, "data": WantResponse::from(row) })))
}
