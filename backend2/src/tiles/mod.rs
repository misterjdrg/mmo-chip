use std::{iter, sync::Arc, time::Duration};

use anyhow::Context;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use reqwest::{StatusCode, header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    APIResult,
    params::domain::{CellInstance, CellType},
    util::Log,
};

pub mod db;
mod domain;
pub use domain::*;

pub async fn cell_crop(
    state: State<Arc<crate::State>>,
    Path((die_id, cell_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let mut file = None;

    // Waiting 10 seconds for tile, if not present
    for i in 0..40 {
        let Ok(r) = db::get_clip_file::<CellInstance>(&state.db, die_id, cell_id)
            .await
            .context("failed to get file for cell clip")
            .log_error()
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        if let Some(f) = r {
            file = Some(f);
            break;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    match file {
        Some(f) => (([(header::CONTENT_TYPE, f.mime)]), f.bytes).into_response(),
        None => (StatusCode::NOT_FOUND).into_response(),
    }
}

pub async fn celltypes_crop(
    state: State<Arc<crate::State>>,
    Path((die_id, celltype_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let mut file = None;

    // Waiting 10 seconds for tile, if not present
    for i in 0..40 {
        let Ok(r) = db::get_clip_file::<CellType>(&state.db, die_id, celltype_id)
            .await
            .context("failed to get file for cell type clip")
            .log_error()
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        if let Some(f) = r {
            file = Some(f);
            break;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    match file {
        Some(f) => (([(header::CONTENT_TYPE, f.mime)]), f.bytes).into_response(),
        None => (StatusCode::NOT_FOUND).into_response(),
    }
}

pub async fn get(
    state: State<Arc<crate::State>>,
    Path((die_id, z, x, y)): Path<(Uuid, u32, u32, u32)>,
) -> Response {
    let loc = TileLocation { z, x, y };

    let mut file = None;

    // Waiting 10 seconds for tile, if not present
    for i in 0..40 {
        let Ok(r) = db::get_file(&state.db, die_id, &loc)
            .await
            .context("failed to get file for tile")
            .log_error()
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        if let Some(f) = r {
            file = Some(f);
            break;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    match file {
        Some(f) => (([(header::CONTENT_TYPE, f.mime)]), f.bytes).into_response(),
        None => (StatusCode::NOT_FOUND).into_response(),
    }
}
