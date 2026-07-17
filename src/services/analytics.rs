use std::collections::HashMap;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::{info, instrument, warn};

use crate::db::analytics::{fetch_wants_for_scoring, upsert_scores, WantScore};
use crate::errors::AppResult;

#[instrument(skip(pool))]
pub async fn calculate_zscores(pool: &PgPool) -> AppResult<()> {
    let wants = fetch_wants_for_scoring(pool).await?;

    let mut by_category: HashMap<i32, Vec<usize>> = HashMap::new();
    for (idx, want) in wants.iter().enumerate() {
        if let Some(cat_id) = want.category_id {
            by_category.entry(cat_id).or_default().push(idx);
        }
    }

    let mut all_scores: Vec<WantScore> = Vec::new();
    let mut categories_processed: usize = 0;

    for (cat_id, indices) in &by_category {
        if indices.len() < 2 {
            warn!(category_id = cat_id, "skipping category with < 2 wants");
            continue;
        }

        let prices: Vec<f64> = indices
            .iter()
            .map(|&i| wants[i].price_limit.to_f64().unwrap_or(0.0))
            .collect();

        let activities: Vec<f64> = indices
            .iter()
            .map(|&i| {
                f64::from(wants[i].views.unwrap_or(0))
                    + f64::from(wants[i].kwork_count.unwrap_or(0))
            })
            .collect();

        let mean_p = mean(&prices);
        let std_p = stddev(&prices, mean_p);

        let mean_a = mean(&activities);
        let std_a = stddev(&activities, mean_a);

        for ((&idx, &p), &a) in indices.iter().zip(prices.iter()).zip(activities.iter()) {
            all_scores.push(WantScore {
                want_id: wants[idx].id,
                zscore_price: zscore(p, mean_p, std_p).and_then(Decimal::from_f64),
                zscore_activity: zscore(a, mean_a, std_a).and_then(Decimal::from_f64),
            });
        }

        categories_processed += 1;
    }

    let upserted = upsert_scores(pool, &all_scores).await?;

    info!(
        categories = categories_processed,
        scores_upserted = upserted,
        "z-score batch complete"
    );

    Ok(())
}

fn mean(values: &[f64]) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let n = values.len() as f64;
    values.iter().sum::<f64>() / n
}

fn stddev(values: &[f64], mean: f64) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let n = values.len() as f64;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    variance.sqrt()
}

fn zscore(x: f64, mean: f64, std: f64) -> Option<f64> {
    if std == 0.0 {
        None
    } else {
        Some((x - mean) / std)
    }
}
