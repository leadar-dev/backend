use axum::{
    extract::{Extension, State},
    Json,
};
use serde_json::json;
use tracing::instrument;

use crate::errors::AppResult;
use crate::models::auth::AuthUser;
use crate::AppState;

#[instrument(skip(state, _user))]
pub async fn get_categories(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = crate::db::categories::list_categories(&state.pool).await?;
    Ok(Json(json!({ "ok": true, "data": rows })))
}
