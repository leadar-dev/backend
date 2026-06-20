use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::{debug, info, instrument};

use crate::errors::AppResult;
use crate::models::want::KworkWantPayload;

pub struct UpsertResult {
    pub id: i64,
    pub is_insert: bool,
}

#[derive(sqlx::FromRow)]
struct UpsertRow {
    id: i64,
    is_insert: Option<bool>,
}

#[instrument(skip(pool), fields(source = %payload.source, external_id = payload.want_id))]
pub async fn upsert(pool: &PgPool, payload: &KworkWantPayload) -> AppResult<UpsertResult> {
    debug!("upserting want");

    let price_limit = Decimal::from_f64_retain(payload.price_limit)
        .unwrap_or_else(|| Decimal::from(0i64));
    let possible_price_limit = Decimal::from_f64_retain(payload.possible_price_limit)
        .unwrap_or_else(|| Decimal::from(0i64));
    let hired_percent = payload
        .hired_percent
        .and_then(Decimal::from_f64_retain);

    let row: UpsertRow = sqlx::query_as::<_, UpsertRow>(
        r#"
        INSERT INTO wants (
            source, external_id, name, description,
            price_limit, possible_price_limit,
            category_id, max_days, status,
            kwork_count, views, hired_percent,
            url, date_create, date_expire, parsed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, now())
        ON CONFLICT (source, external_id) DO UPDATE SET
            name                 = EXCLUDED.name,
            description          = EXCLUDED.description,
            price_limit          = EXCLUDED.price_limit,
            possible_price_limit = EXCLUDED.possible_price_limit,
            category_id          = EXCLUDED.category_id,
            max_days             = EXCLUDED.max_days,
            status               = EXCLUDED.status,
            kwork_count          = EXCLUDED.kwork_count,
            views                = EXCLUDED.views,
            hired_percent        = EXCLUDED.hired_percent,
            url                  = EXCLUDED.url,
            date_create          = EXCLUDED.date_create,
            date_expire          = EXCLUDED.date_expire,
            parsed_at            = now(),
            updated_at           = now()
        RETURNING id, (xmax = 0) AS is_insert
        "#,
    )
    .bind(&payload.source)
    .bind(payload.want_id)
    .bind(&payload.name)
    .bind(payload.description.as_deref())
    .bind(price_limit)
    .bind(possible_price_limit)
    .bind(payload.category_id)
    .bind(payload.max_days)
    .bind(payload.status.to_lowercase())
    .bind(payload.kwork_count)
    .bind(payload.views)
    .bind(hired_percent)
    .bind(&payload.url)
    .bind(payload.date_create)
    .bind(payload.date_expire)
    .fetch_one(pool)
    .await?;

    let id = row.id;
    let is_insert = row.is_insert.unwrap_or(false);

    if is_insert {
        info!(want_id = id, source = %payload.source, "want inserted");
    } else {
        debug!(want_id = id, source = %payload.source, "want updated");
    }

    Ok(UpsertResult { id, is_insert })
}
