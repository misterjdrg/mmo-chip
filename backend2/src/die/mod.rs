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

macro_rules! die_param_get {
    ($kind: ident) => {
        pub async fn ${concat($kind, _get)}(
            state: State<Arc<crate::State>>,
            Path((die_id, id)): Path<(Uuid, Uuid)>,
        ) -> APIResult<()> {
            todo!()
        }
    };
}
macro_rules! die_param_delete {
    ($kind: ident) => {
        pub async fn ${concat($kind, _delete)}(
            state: State<Arc<crate::State>>,
            Path((die_id, id)): Path<(Uuid, Uuid)>,
        ) -> APIResult<()> {
            todo!()
        }
    };
}
macro_rules! die_param {
    ($kind: ident) => {
        die_param_get!($kind);
        die_param_delete!($kind);
    };
}

die_param!(cell);
die_param!(cell_type);
die_param!(net);
die_param!(grid);
die_param!(pin);
die_param!(annotation);
die_param!(roi);
die_param!(ignore);
die_param!(guide);
