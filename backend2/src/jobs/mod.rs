use std::sync::Arc;

use anyhow::Context;
use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{APIResult, DB, die::db as die_db, tiles::DieInfo, util};

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
#[derive(Serialize, Deserialize)]
pub enum JobKind {
    ImportDie { die_id: Uuid },
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
    let img = image::load_from_memory(&bytes).context("failed to parse image")?;
    let file_id = die_db::create_file(&state.db, &filename, "image/*", &bytes)
        .await
        .context("failed to upload image")?;

    let die_info = DieInfo::new(img.width(), img.height(), state.config.tile_size);
    let levels = die_info.build_levels();
    let die_id = die_db::create_die(&state.db, &filename, file_id, &die_info, &levels)
        .await
        .context("failed to create die")?;

    let job_id = db::create_job(&state.db, &JobKind::ImportDie { die_id })
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

pub async fn worker(db: DB, mut recv: mpsc::Receiver<Uuid>) {
    // kernel: sharp.kernel.lanczos3
    // jpeg({ quality: 90 })
    while let Some(job_id) = recv.recv().await {
        eprintln!("worker: peeking job: {job_id}");
    }
}
