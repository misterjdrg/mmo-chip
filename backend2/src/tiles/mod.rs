use std::{iter, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use reqwest::{StatusCode, header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{APIResult, util::Log};

pub mod db;
mod domain;
pub use domain::*;

pub async fn cell_crop(
    state: State<Arc<crate::State>>,
    Path((die_id, cell_id)): Path<(Uuid, Uuid)>,
) -> APIResult<()> {
    todo!()
}

pub async fn celltypes_crop(
    state: State<Arc<crate::State>>,
    Path((die_id, celltype_id)): Path<(Uuid, Uuid)>,
) -> APIResult<()> {
    todo!()
}

pub async fn get(
    state: State<Arc<crate::State>>,
    Path((die_id, z, x, y)): Path<(Uuid, u32, u32, u32)>,
) -> Response {
    let loc = TileLocation { z, x, y };

    match db::get_file(&state.db, die_id, &loc)
        .await
        .context("failed to get file for tile")
        .log_error()
    {
        Ok(Some(f)) => (([(header::CONTENT_TYPE, f.mime)]), f.bytes).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND).into_response(),
        Err(e) => {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
