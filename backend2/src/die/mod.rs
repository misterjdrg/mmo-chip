use std::sync::Arc;

use anyhow::Context;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{APIResult, die::db::Die, tiles::TileTree};

pub mod db;

pub async fn list(state: State<Arc<crate::State>>) -> APIResult<Vec<Die>> {
    Ok(Json(db::all_dies(&state.db).await?))
}
pub async fn get(state: State<Arc<crate::State>>, Path(die_id): Path<Uuid>) -> APIResult<Die> {
    let die = db::get(&state.db, die_id)
        .await
        .context("failed to get die")?
        .context("no die")?;
    Ok(Json(die))
}
pub async fn delete(state: State<Arc<crate::State>>, Path(die_id): Path<Uuid>) -> APIResult<()> {
    db::delete(&state.db, die_id)
        .await
        .context("failed to delete die")?;

    Ok(Json(()))
}
