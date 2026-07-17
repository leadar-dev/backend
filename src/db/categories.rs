use sqlx::PgPool;
use tracing::debug;

use crate::errors::AppResult;
use crate::models::categories_event::KworkCategoriesPayload;
use crate::models::category::CategoryRow;

pub async fn list_categories(pool: &PgPool) -> AppResult<Vec<CategoryRow>> {
    let rows =
        sqlx::query_as::<_, CategoryRow>("SELECT * FROM categories ORDER BY source, name")
            .fetch_all(pool)
            .await?;
    Ok(rows)
}

pub async fn upsert_categories(pool: &PgPool, payload: &KworkCategoriesPayload) -> AppResult<usize> {
    let source = &payload.source;

    for cat in &payload.categories {
        sqlx::query!(
            "INSERT INTO categories (source, external_id, name)
             VALUES ($1, $2, $3)
             ON CONFLICT (source, external_id) DO UPDATE SET name = EXCLUDED.name",
            source,
            cat.external_id,
            cat.name,
        )
        .execute(pool)
        .await?;
    }

    for cat in payload.categories.iter().filter(|c| c.parent_external_id.is_some()) {
        let Some(parent_ext_id) = cat.parent_external_id else { continue };
        let updated = sqlx::query!(
            "UPDATE categories c SET parent_id = p.id
             FROM categories p
             WHERE c.source = $1 AND c.external_id = $2
               AND p.source = $1 AND p.external_id = $3",
            source,
            cat.external_id,
            parent_ext_id,
        )
        .execute(pool)
        .await?;
        debug!(external_id = cat.external_id, parent = parent_ext_id, rows = updated.rows_affected(), "parent linked");
    }

    Ok(payload.categories.len())
}
