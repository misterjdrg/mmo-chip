use std::{
    io,
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::Context;
use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use chrono::{Local, RoundingError::DurationExceedsTimestamp};
use futures::StreamExt;
use image::{DynamicImage, ImageFormat, ImageReader, Limits, codecs::jpeg::JpegEncoder};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

pub mod worker;

use crate::{
    APIError, APIResult, Config, DB,
    die::{self, db as die_db},
    file,
    jobs::db::JobKind,
    params::{
        self,
        domain::{Cell, CellType},
    },
    realtime::{self, DieImportState, RealtimeEvent},
    tiles::{self, TileClip, TileLocation, TileTree},
    util::{self, Log},
};

pub mod db;

#[derive(Serialize, Deserialize)]
pub enum ImportJobStatus {
    Queued,
    Running,
    Complided,
    Failed,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportJobPhase {
    Queued,
    Analyzing,
    Tiling,
    Persisting,
    Complited,
    Failed,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportJobProgress {
    phase: ImportJobPhase,
    message: String,
    total_levels: u32,
    completed_levels: u32,
    current_level: Option<u32>,
    current_level_tiles: u32,
    current_level_processed_tiles: u32,
    total_tiles: u32,
    processed_tiles: u32,
    percentage: u32,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResponse {
    id: Uuid,
    #[serde(rename = "type")]
    kind: String,
    status: ImportJobStatus,
    original_file_name: String,
    die_id: Option<Uuid>,
    error: Option<String>,
    created_at: String,
    updated_at: String,
    started_at: String,
    finished_at: String,
    progress: ImportJobProgress,
}

pub async fn die_import(
    state: State<Arc<crate::State>>,
    multipart: Multipart,
) -> APIResult<ImportResponse> {
    let (filename, bytes) = util::get_single_file(multipart)
        .await
        .context("failed to get file from request")?
        .context("no file")?;

    let filename = filename
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || ['-', '_', '.'].contains(c))
        .flat_map(|c| c.to_lowercase())
        .collect::<String>();
    let buf = bytes.deref();
    let reader = ImageReader::new(io::Cursor::new(buf));
    let reader = reader
        .with_guessed_format()
        .context("failed to guess format")?;
    let image_format = reader.format().context("no image format")?;
    let dims = reader
        .into_dimensions()
        .context("failed to decode image dimensions")?;

    let mime = match image_format {
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::Png => "image/png",
        fmt => {
            return Err(APIError::General(anyhow::anyhow!(
                "unsupporerted image format: {fmt:?}, only supported png, jpeg"
            )));
        }
    };

    let tree = TileTree::new(dims.0, dims.1, state.config.tile_size);
    let levels = tree.build_levels();
    let die_id = die::db::create(&state.db, &filename, &tree, &levels)
        .await
        .context("failed to create die")?;

    let file_id = file::db::create(&state.db, die_id, &filename, mime, &bytes)
        .await
        .context("failed to upload image")?;
    drop(bytes);

    die::db::set_original_file(&state.db, die_id, file_id).await?;

    let job_id = db::create_import_die_job(&state.db, die_id)
        .await
        .context("failed to create import die job")?;

    state
        .job_sender
        .send(job_id)
        .await
        .context("failed to push job on the queue")?;

    Ok(Json(ImportResponse {
        id: job_id,
        kind: "import-die".into(),
        status: ImportJobStatus::Queued,
        original_file_name: filename,
        die_id: Some(die_id),
        error: None,
        created_at: Local::now().to_rfc3339(),
        updated_at: Local::now().to_rfc3339(),
        started_at: Local::now().to_rfc3339(),
        finished_at: Local::now().to_rfc3339(),
        progress: ImportJobProgress {
            phase: ImportJobPhase::Queued,
            message: "Queued".into(),
            total_levels: 3,
            completed_levels: 0,
            current_level: None,
            current_level_tiles: 0,
            current_level_processed_tiles: 0,
            total_tiles: 1024,
            processed_tiles: 0,
            percentage: 0,
        },
    }))
}

pub async fn list_import(state: State<Arc<crate::State>>) -> APIResult<Vec<ImportResponse>> {
    Ok(Json(vec![]))
}
pub async fn get_import(
    state: State<Arc<crate::State>>,
    Path(job_id): Path<Uuid>,
) -> APIResult<()> {
    todo!()
}
