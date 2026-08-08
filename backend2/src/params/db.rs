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
) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM die_params WHERE id = $1 AND die_id = $2 AND kind = $3")
        .bind(param_id)
        .bind(die_id)
        .bind(P::KIND.as_ref())
        .execute(db)
        .await
        .with_context(|| format!("failed to delete die param {}", P::KIND.as_ref()))
        .map(|r| r.rows_affected() > 0)?
        .ok_or_else(|| anyhow::anyhow!("no param deleted"))
}

/// Don't change id, die_id, kind
/// Only content updated
pub async fn update_content<P: IsParamKind>(
    db: &DB,
    die_id: Uuid,
    param: &P,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE die_params SET content = $1 WHERE id = $2 AND die_id = $3 AND kind = $4")
        .bind(Json(param))
        .bind(param.get_id())
        .bind(die_id)
        .bind(P::KIND.as_ref())
        .execute(db)
        .await
        .with_context(|| {
            format!(
                "failed to update content for die param {} with id {}",
                P::KIND.as_ref(),
                param.get_id()
            )
        })
        .map(|r| r.rows_affected() > 0)?
        .ok_or_else(|| anyhow::anyhow!("no param content updated"))
}
pub async fn insert_or_update_content<P: IsParamKind>(
    db: &DB,
    die_id: Uuid,
    param: &P,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO die_params(id, die_id, kind, content) VALUES ($1, $2, $3, $4) ON CONFLICT(id, die_id, kind) DO UPDATE SET content = $4")
          .bind(param.get_id())
          .bind(die_id)
          .bind(P::KIND.as_ref())
          .bind(Json(param))
          .execute(db)
          .await
          .with_context(|| format!("failed to insert_or_update die param {}", P::KIND.as_ref()))
          .map(|r| r.rows_affected() > 0)?
          .ok_or_else(|| anyhow::anyhow!("no param changed"))
}
pub async fn get<P: IsParamKind>(
    db: &DB,
    die_id: Uuid,
    param_id: Uuid,
) -> anyhow::Result<Option<P>> {
    sqlx::query_scalar::<_, Json<P>>(
        "SELECT content FROM die_params WHERE id = $1 AND die_id = $2 AND kind = $3",
    )
    .bind(param_id)
    .bind(die_id)
    .bind(P::KIND.as_ref())
    .fetch_optional(db)
    .await
    .with_context(|| format!("failed to get of {}", P::KIND.as_ref()))
    .map(|p| p.map(|j| j.0))
}

pub async fn insert_or_update_clip<P: IsParamKind>(
    db: &DB,
    die_id: Uuid,
    param: &P,
    file_id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO clips(die_id, owner_id, owner_kind, file_id, created_at) VALUES ($1, $2, $3, $4, $5) ON CONFLICT(die_id, owner_id, owner_kind) DO UPDATE SET file_id = $4")
        .bind(die_id)
        .bind(param.get_id())
        .bind(P::KIND.as_ref())
        .bind(file_id)
        .execute(db)
        .await
        .with_context(|| format!("failed to insert_or_update_clip{}", P::KIND.as_ref()))
        .map(|r| r.rows_affected() > 0)?
        .ok_or_else(|| anyhow::anyhow!("no clip changed"))
}
