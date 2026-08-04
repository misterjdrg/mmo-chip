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

use crate::{APIResult, jobs::JobKind, tiles::DieInfo};
use db::DieSummary;

pub mod db;

pub async fn list(state: State<Arc<crate::State>>) -> APIResult<Vec<DieSummary>> {
    Ok(Json(db::list_die_summaries(&state.db).await?))
}
pub async fn get(state: State<Arc<crate::State>>, Path(die_id): Path<Uuid>) -> APIResult<()> {
    todo!()
}
pub async fn delete(state: State<Arc<crate::State>>, Path(die_id): Path<Uuid>) -> APIResult<()> {
    todo!()
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
