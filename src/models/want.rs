use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct WantId(pub i64);

impl fmt::Display for WantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WantStatus {
    Active,
    Archive,
    Closed,
}

impl WantStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WantStatus::Active => "active",
            WantStatus::Archive => "archive",
            WantStatus::Closed => "closed",
        }
    }
}

impl fmt::Display for WantStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for WantStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(WantStatus::Active),
            "archive" => Ok(WantStatus::Archive),
            "closed" => Ok(WantStatus::Closed),
            other => Err(format!("unknown status: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Kwork,
    Fl,
    Upwork,
}

impl Source {
    pub fn as_str(&self) -> &'static str {
        match self {
            Source::Kwork => "kwork",
            Source::Fl => "fl",
            Source::Upwork => "upwork",
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Source {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "kwork" => Ok(Source::Kwork),
            "fl" => Ok(Source::Fl),
            "upwork" => Ok(Source::Upwork),
            other => Err(format!("unknown source: {other}")),
        }
    }
}

/// DB row — maps directly to the `wants` table
#[derive(Debug, sqlx::FromRow)]
pub struct WantRow {
    pub id: i64,
    pub source: String,
    pub external_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub price_limit: Decimal,
    pub possible_price_limit: Decimal,
    pub category_id: Option<i32>,
    pub max_days: Option<i32>,
    pub status: String,
    pub kwork_count: Option<i32>,
    pub views: Option<i32>,
    pub hired_percent: Option<Decimal>,
    pub url: String,
    pub date_create: DateTime<Utc>,
    pub date_expire: Option<DateTime<Utc>>,
    pub parsed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// REST response DTO
#[derive(Debug, Serialize)]
pub struct WantResponse {
    pub id: i64,
    pub source: String,
    pub external_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub price_limit: f64,
    pub possible_price_limit: f64,
    pub category_id: Option<i32>,
    pub max_days: Option<i32>,
    pub status: String,
    pub kwork_count: Option<i32>,
    pub views: Option<i32>,
    pub hired_percent: Option<f64>,
    pub url: String,
    pub date_create: DateTime<Utc>,
    pub date_expire: Option<DateTime<Utc>>,
    pub parsed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WantRow> for WantResponse {
    fn from(row: WantRow) -> Self {
        Self {
            id: row.id,
            source: row.source,
            external_id: row.external_id,
            name: row.name,
            description: row.description,
            price_limit: row.price_limit.to_f64().unwrap_or(0.0),
            possible_price_limit: row.possible_price_limit.to_f64().unwrap_or(0.0),
            category_id: row.category_id,
            max_days: row.max_days,
            status: row.status,
            kwork_count: row.kwork_count,
            views: row.views,
            hired_percent: row.hired_percent.and_then(|d| d.to_f64()),
            url: row.url,
            date_create: row.date_create,
            date_expire: row.date_expire,
            parsed_at: row.parsed_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Message envelope from broker
#[derive(Debug, Deserialize, Serialize)]
pub struct MessageEnvelope {
    pub event: String,
    pub version: i32,
    pub timestamp: String,
    pub payload: Value,
}

/// Payload for want upsert from parser
#[derive(Debug, Deserialize, Serialize)]
pub struct KworkWantPayload {
    pub source: String,
    pub want_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub price_limit: f64,
    pub possible_price_limit: f64,
    pub category_id: i32,
    pub max_days: Option<i32>,
    pub status: String,
    pub kwork_count: Option<i32>,
    pub views: Option<i32>,
    pub hired_percent: Option<f64>,
    pub url: String,
    pub date_create: DateTime<Utc>,
    pub date_expire: Option<DateTime<Utc>>,
}
