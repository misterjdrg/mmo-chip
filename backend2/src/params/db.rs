use anyhow::Context;
use sqlx::types::Json;
use uuid::Uuid;

use crate::{DB, params::domain::IsParamKind};

pub async fn list<P: IsParamKind>(db: &DB, die_id: Uuid) -> anyhow::Result<Vec<P>> {
    Ok(sqlx::query_scalar::<_, Json<P>>(
        "SELECT content FROM die_params WHERE die_id = $1 AND kind = $2",
    )
    .bind(die_id)
    .bind(P::KIND.as_ref())
    .fetch_all(db)
    .await
    .with_context(|| format!("failed to list of {}", P::KIND.as_ref()))?
    .into_iter()
    .map(|p| p.0)
    .collect())
}
pub async fn delete_one<P: IsParamKind>(
    db: &DB,
    die_id: Uuid,
    param_id: Uuid,
) -> anyhow::Result<bool> {
    sqlx::query("DELETE FROM die_params WHERE id = $1 AND die_id = $2 AND kind = $3")
        .bind(param_id)
        .bind(die_id)
        .bind(P::KIND.as_ref())
        .execute(db)
        .await
        .with_context(|| format!("failed to delete die param {}", P::KIND.as_ref()))
        .map(|r| r.rows_affected() > 0)
}

pub async fn insert_or_update<P: IsParamKind>(
    db: &DB,
    die_id: Uuid,
    param: &P,
) -> anyhow::Result<bool> {
    sqlx::query("INSERT INTO die_params(id, die_id, kind, content) VALUES ($1, $2, $3, $4) ON CONFLICT(id, die_id, kind) DO UPDATE SET content = $4")
          .bind(param.get_id())
          .bind(die_id)
          .bind(P::KIND.as_ref())
          .bind(Json(param))
          .execute(db)
          .await
          .with_context(|| format!("failed to insert_or_update die param {}", P::KIND.as_ref()))
          .map(|r| r.rows_affected() > 0)
}
