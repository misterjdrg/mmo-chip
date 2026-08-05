use std::{io, ops::Deref, sync::Arc, time::Instant};

use anyhow::Context;
use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use chrono::Local;
use futures::StreamExt;
use image::{DynamicImage, ImageFormat, ImageReader, codecs::jpeg::JpegEncoder};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    APIError, APIResult, Config, DB,
    die::{self, db as die_db},
    file,
    jobs::db::JobKind,
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

    let file_id = file::db::create(&state.db, &filename, mime, &bytes)
        .await
        .context("failed to upload image")?;

    let tree = TileTree::new(dims.0, dims.1, state.config.tile_size);
    let levels = tree.build_levels();
    let die_id = die::db::create(&state.db, &filename, file_id, &tree, &levels)
        .await
        .context("failed to create die")?;

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

pub async fn worker(config: Config, db: DB, mut recv: mpsc::Receiver<Uuid>) {
    while let Some(job_id) = recv.recv().await {
        log::info!("worker: peeking job: {job_id}");
        let start = Instant::now();

        match process(&config, &db, job_id)
            .await
            .with_context(|| format!("failed to process job: {job_id}"))
            .log_error()
        {
            Ok(_) => {
                let _ = db::set_status(&db, job_id, db::JobStatus::Done)
                    .await
                    .log_error();
                log::info!("worker: job done {job_id} in {:?}", Instant::now() - start);
            }
            Err(_) => {
                let _ = db::set_status(&db, job_id, db::JobStatus::Failed)
                    .await
                    .log_error();
                log::info!(
                    "worker: job failed {job_id} in {:?}",
                    Instant::now() - start
                );
            }
        }
    }
}
pub async fn process(config: &Config, db: &DB, job_id: Uuid) -> anyhow::Result<()> {
    let job = db::get(db, job_id)
        .await
        .context("failed to get job")?
        .context("no job")?;

    match job.kind.0 {
        JobKind::ImportDie { die_id } => {
            process_die_tiling(config, db, job_id, die_id).await?;
        }
    }
    Ok(())
}

pub async fn process_die_tiling(
    config: &Config,
    db: &DB,
    job_id: Uuid,
    die_id: Uuid,
) -> anyhow::Result<()> {
    db::set_started_at(db, job_id)
        .await
        .context("failed mark job as started")?;
    let die = die::db::get(db, die_id)
        .await
        .context("failed to get die")?
        .context("no die")?;

    let tree = TileTree::new(die.width, die.height, die.tile_size);

    let file = file::db::get(db, die.original_file_id)
        .await
        .context("failed to get file")?
        .context("no file")?;
    let shot = Arc::new(
        tokio::task::spawn_blocking(move || image::load_from_memory(&file.bytes))
            .await
            .context("failed at die decode task")?
            .context("failed to load image")?,
    );

    futures::stream::iter(tree.all_tiles())
        .for_each_concurrent(config.tile_concurency as usize, async |(loc, clip)| {
            tile_die(db, die.id, shot.clone(), loc, clip).await.log();
        })
        .await;

    // should be no active links after foreach
    drop(Arc::try_unwrap(shot).unwrap());

    db::set_finished_at(db, job_id)
        .await
        .context("failed mark job as finished")?;
    Ok(())
}
async fn tile_die(
    db: &DB,
    die_id: Uuid,
    shot: Arc<DynamicImage>,
    loc: TileLocation,
    clip: TileClip,
) -> anyhow::Result<()> {
    let buf = tokio::task::spawn_blocking(move || {
        let cropped = shot.crop_imm(clip.src.x, clip.src.y, clip.src.w, clip.src.h);
        let resized = cropped.resize_exact(
            clip.dst_w,
            clip.dst_h,
            image::imageops::FilterType::Lanczos3,
        );
        drop(cropped);
        let mut buf = vec![];
        let encoder = JpegEncoder::new_with_quality(&mut buf, 90);
        resized.write_with_encoder(encoder);
        drop(clip);
        buf
    })
    .await
    .context("failed to join")?;

    let file_id = file::db::create(
        db,
        &format!("tile_{}_{}_{}.jpeg", loc.z, loc.x, loc.y),
        "image/jpeg",
        &buf,
    )
    .await
    .context("failed to create tile file")?;
    drop(buf);

    tiles::db::assign(db, die_id, &loc, file_id)
        .await
        .context("failed to set tile")?;

    Ok(())
}
