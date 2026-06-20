use chrono::Utc;
use hmac::{Hmac, Mac};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tracing::instrument;

use crate::errors::{AppError, AppResult};
use crate::models::auth::TelegramAuthData;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
}

/// Verify Telegram login widget HMAC-SHA256 signature.
#[instrument(skip(data, bot_token), fields(telegram_id = data.id))]
pub fn verify_telegram_auth(data: &TelegramAuthData, bot_token: &str) -> bool {
    // secret_key = SHA256(bot_token)
    let mut hasher = Sha256::new();
    hasher.update(bot_token.as_bytes());
    let secret_key = hasher.finalize();

    // Build check_string: sorted key=value pairs (excluding "hash"), joined by "\n"
    let mut fields: Vec<String> = Vec::new();
    fields.push(format!("auth_date={}", data.auth_date));
    fields.push(format!("first_name={}", data.first_name));
    fields.push(format!("id={}", data.id));
    if let Some(ref username) = data.username {
        fields.push(format!("username={username}"));
    }
    if let Some(ref photo_url) = data.photo_url {
        fields.push(format!("photo_url={photo_url}"));
    }
    fields.sort();
    let check_string = fields.join("\n");

    // HMAC-SHA256(check_string, secret_key)
    let mut mac = match Hmac::<Sha256>::new_from_slice(&secret_key) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(check_string.as_bytes());
    let result = mac.finalize();
    let computed = hex::encode(result.into_bytes());

    computed == data.hash
}

/// Issue a JWT for a verified Telegram user.
pub fn create_jwt(telegram_id: i64, secret: &str, expiry_hours: i64) -> AppResult<String> {
    let now = Utc::now().timestamp();
    let claims = JwtClaims {
        sub: telegram_id.to_string(),
        iat: now,
        exp: now + expiry_hours * 3600,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT encode error: {}", e)))
}

/// Verify and decode a JWT, returning the claims.
pub fn verify_jwt(token: &str, secret: &str) -> AppResult<JwtClaims> {
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}

/// Insert or update a user record for a Telegram ID.
#[instrument(skip(pool), fields(telegram_id))]
pub async fn upsert_user(pool: &PgPool, telegram_id: i64) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO users (telegram_id, is_active)
        VALUES ($1, true)
        ON CONFLICT (telegram_id) DO UPDATE SET
            is_active  = true,
            updated_at = now()
        "#,
    )
    .bind(telegram_id)
    .execute(pool)
    .await?;
    Ok(())
}
