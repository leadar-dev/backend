use axum::{extract::Request, middleware::Next, response::Response};

use crate::errors::AppError;
use crate::models::auth::AuthUser;

pub async fn require_admin(req: Request, next: Next) -> Result<Response, AppError> {
    let user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(AppError::Unauthorized)?;

    if user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    Ok(next.run(req).await)
}
