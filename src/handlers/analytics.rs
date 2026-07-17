use axum::{
    extract::{Extension, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use tracing::instrument;

use crate::errors::AppResult;
use crate::models::analytics::{HeatmapPoint, ZscoreResponse};
use crate::models::auth::AuthUser;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ZscoreQuery {
    pub category_id: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[instrument(skip(state, _user), fields(category_id = ?query.category_id, limit = ?query.limit))]
pub async fn get_zscore(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Query(query): Query<ZscoreQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = query.limit.unwrap_or(50).min(200).max(1);
    let offset = query.offset.unwrap_or(0).max(0);

    let rows = crate::db::analytics::list_zscore(&state.pool, query.category_id, limit, offset).await?;
    let data: Vec<ZscoreResponse> = rows.into_iter().map(ZscoreResponse::from).collect();

    Ok(Json(json!({ "ok": true, "data": data })))
}

#[instrument(skip(state, _user))]
pub async fn get_heatmap(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = crate::db::analytics::list_heatmap(&state.pool).await?;
    let data: Vec<HeatmapPoint> = rows.into_iter().map(HeatmapPoint::from).collect();

    Ok(Json(json!({ "ok": true, "data": data })))
}
