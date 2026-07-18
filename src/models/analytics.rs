use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, sqlx::FromRow)]
pub struct ZscoreRow {
    pub want_id: i64,
    pub name: String,
    pub url: String,
    pub category_id: Option<i32>,
    pub price_limit: Decimal,
    pub zscore_price: Option<Decimal>,
    pub zscore_activity: Option<Decimal>,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ZscoreResponse {
    pub want_id: i64,
    pub name: String,
    pub url: String,
    pub category_id: Option<i32>,
    pub price_limit: String,
    pub zscore_price: Option<f64>,
    pub zscore_activity: Option<f64>,
    pub calculated_at: DateTime<Utc>,
}

impl From<ZscoreRow> for ZscoreResponse {
    fn from(row: ZscoreRow) -> Self {
        Self {
            want_id: row.want_id,
            name: row.name,
            url: row.url,
            category_id: row.category_id,
            price_limit: row.price_limit.to_string(),
            zscore_price: row.zscore_price.and_then(|d| d.to_f64()),
            zscore_activity: row.zscore_activity.and_then(|d| d.to_f64()),
            calculated_at: row.calculated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct HeatmapRow {
    pub date: DateTime<Utc>,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct HeatmapPoint {
    pub date: NaiveDate,
    pub count: i64,
}

impl From<HeatmapRow> for HeatmapPoint {
    fn from(row: HeatmapRow) -> Self {
        Self {
            date: row.date.date_naive(),
            count: row.count,
        }
    }
}
