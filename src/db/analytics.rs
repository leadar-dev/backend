use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::instrument;

use crate::errors::AppResult;

#[derive(Debug, sqlx::FromRow)]
pub struct WantForScoring {
    pub id: i64,
    pub category_id: Option<i32>,
    pub price_limit: Decimal,
    pub views: Option<i32>,
    pub kwork_count: Option<i32>,
}

pub struct WantScore {
    pub want_id: i64,
    pub zscore_price: Option<Decimal>,
    pub zscore_activity: Option<Decimal>,
}

#[instrument(skip(pool))]
pub async fn fetch_wants_for_scoring(pool: &PgPool) -> AppResult<Vec<WantForScoring>> {
    let rows = sqlx::query_as::<_, WantForScoring>(
        "SELECT id, category_id, price_limit, views, kwork_count FROM wants WHERE status = 'active'",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[instrument(skip(pool, scores), fields(count = scores.len()))]
pub async fn upsert_scores(pool: &PgPool, scores: &[WantScore]) -> AppResult<u64> {
    if scores.is_empty() {
        return Ok(0);
    }

    let want_ids: Vec<i64> = scores.iter().map(|s| s.want_id).collect();
    let zscore_prices: Vec<Option<Decimal>> = scores.iter().map(|s| s.zscore_price).collect();
    let zscore_activities: Vec<Option<Decimal>> =
        scores.iter().map(|s| s.zscore_activity).collect();
    let calculated_at = Utc::now();

    let result = sqlx::query(
        "INSERT INTO want_scores (want_id, zscore_price, zscore_activity, trend_slope, calculated_at)
        SELECT
            unnest($1::bigint[]),
            unnest($2::numeric[]),
            unnest($3::numeric[]),
            NULL::numeric,
            $4::timestamptz
        ON CONFLICT (want_id) DO UPDATE SET
            zscore_price    = EXCLUDED.zscore_price,
            zscore_activity = EXCLUDED.zscore_activity,
            trend_slope     = NULL,
            calculated_at   = EXCLUDED.calculated_at",
    )
    .bind(&want_ids)
    .bind(&zscore_prices)
    .bind(&zscore_activities)
    .bind(calculated_at)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
