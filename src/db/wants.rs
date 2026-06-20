use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::errors::AppResult;
use crate::handlers::wants::WantsQuery;
use crate::models::want::WantRow;

pub async fn list_wants(
    pool: &PgPool,
    query: &WantsQuery,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<WantRow>> {
    let mut qb = sqlx::QueryBuilder::new("SELECT * FROM wants WHERE 1=1");

    if let Some(ref source) = query.source {
        qb.push(" AND source = ");
        qb.push_bind(source.clone());
    }
    if let Some(category_id) = query.category_id {
        qb.push(" AND category_id = ");
        qb.push_bind(category_id);
    }
    if let Some(price_min) = query.price_min {
        qb.push(" AND price_limit >= ");
        qb.push_bind(
            Decimal::from_f64_retain(price_min).unwrap_or_else(|| Decimal::from(0i64)),
        );
    }
    if let Some(price_max) = query.price_max {
        qb.push(" AND price_limit <= ");
        qb.push_bind(
            Decimal::from_f64_retain(price_max)
                .unwrap_or_else(|| Decimal::from(i64::MAX)),
        );
    }
    if let Some(ref status) = query.status {
        qb.push(" AND status = ");
        qb.push_bind(status.clone());
    }

    qb.push(" ORDER BY date_create DESC");
    qb.push(" LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let rows = qb
        .build_query_as::<WantRow>()
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

pub async fn get_want_by_id(pool: &PgPool, id: i64) -> AppResult<Option<WantRow>> {
    let row = sqlx::query_as::<_, WantRow>("SELECT * FROM wants WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}
