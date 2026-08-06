use anyhow::Context;
use sqlx::types::Json;
use uuid::Uuid;

use crate::{DB, params::domain::IsParamKind};

pub async fn list<'de, P: IsParamKind>(db: &DB, die_id: Uuid) -> anyhow::Result<Vec<P>> {
    Ok(sqlx::query_scalar::<_, Json<P>>(
        "SELECT content FROM die_params WHERE die_id = $1 AND kind = $2",
    )
    .bind(die_id)
    .bind(P::KIND.as_ref())
    .fetch_all(db)
    .await
    .context("failed to delete die")?
    .into_iter()
    .map(|p| p.0)
    .collect())
}
