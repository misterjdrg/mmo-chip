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
use image::{DynamicImage, ImageFormat, ImageReader, codecs::jpeg::JpegEncoder};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

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

pub async fn worker(state: Arc<crate::State>, mut recv: mpsc::Receiver<Uuid>) {
    while let Some(job_id) = recv.recv().await {
        let start = Instant::now();

        match process(state.clone(), job_id)
            .await
            .with_context(|| format!("failed to process job: {job_id}"))
            .log_error()
        {
            Ok(_) => {
                let _ = db::set_status(&state.db, job_id, db::JobStatus::Done)
                    .await
                    .log_error();
                log::info!("worker: job done {job_id} in {:?}", Instant::now() - start);
            }
            Err(_) => {
                let _ = db::set_status(&state.db, job_id, db::JobStatus::Failed)
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
pub async fn process(state: Arc<crate::State>, job_id: Uuid) -> anyhow::Result<()> {
    let job = db::get(&state.db, job_id)
        .await
        .context("failed to get job")?
        .context("no job")?;

    match job.kind.0 {
        JobKind::ImportDie { die_id } => {
            process_die_tiling(state.clone(), job_id, die_id).await?;
        }
        JobKind::ClipCell {
            die_id,
            owner: owner_id,
        } => process_clip_cell(state.clone(), job_id, die_id, owner_id).await?,
    }
    Ok(())
}
pub async fn process_clip_cell(
    state: Arc<crate::State>,
    job_id: Uuid,
    die_id: Uuid,
    owner_id: Uuid,
) -> anyhow::Result<()> {
    db::set_started_at(&state.db, job_id)
        .await
        .context("failed mark job as started")?;

    let clip = match true {
        _ if let Some(cell) = params::db::get::<Cell>(&state.db, die_id, owner_id).await? => {
            todo!()
        }
        _ if let Some(cell_type) =
            params::db::get::<CellType>(&state.db, die_id, owner_id).await? =>
        {
            cell_type.crop_rect
        }
        _ => anyhow::bail!("Parameter not found"),
    };
    let die = die::db::get(&state.db, die_id)
        .await
        .context("failed to get die")?
        .context("no die")?;
    let file = file::db::get(&state.db, die.original_file_id)
        .await
        .context("failed to get file")?
        .context("no file")?;
    let shot = Arc::new(
        tokio::task::spawn_blocking(move || image::load_from_memory(&file.bytes))
            .await
            .context("failed at die decode task")?
            .context("failed to load image")?,
    );

    db::set_finished_at(&state.db, job_id)
        .await
        .context("failed mark job as finished")?;

    Ok(())
}

pub async fn process_die_tiling(
    state: Arc<crate::State>,
    job_id: Uuid,
    die_id: Uuid,
) -> anyhow::Result<()> {
    // for frontend to subscribe to channel
    tokio::time::sleep(Duration::from_millis(250)).await;

    db::set_started_at(&state.db, job_id)
        .await
        .context("failed mark job as started")?;

    state.rt_sender.send(RealtimeEvent::DieImportStateChange {
        die_id,
        state: DieImportState::Peeked,
    });

    let die = die::db::get(&state.db, die_id)
        .await
        .context("failed to get die")?
        .context("no die")?;

    let tree = TileTree::new(die.width, die.height, die.tile_size);

    let file = file::db::get(&state.db, die.original_file_id)
        .await
        .context("failed to get file")?
        .context("no file")?;

    state.rt_sender.send(RealtimeEvent::DieImportStateChange {
        die_id,
        state: DieImportState::ShotDecoding,
    });

    let shot = Arc::new(
        tokio::task::spawn_blocking(move || image::load_from_memory(&file.bytes))
            .await
            .context("failed at die decode task")?
            .context("failed to load image")?,
    );

    state.rt_sender.send(RealtimeEvent::DieImportStateChange {
        die_id,
        state: DieImportState::ShotDecoded,
    });

    let counters_recv = Arc::new((AtomicU32::new(0), tree.build_levels().tile_count()));
    let counters_send = Arc::clone(&counters_recv);
    let rt_sender = state.rt_sender.clone();
    let (_, _) = tokio::join!(
        futures::stream::iter(tree.all_tiles()).for_each_concurrent(
            state.config.tile_concurency as usize,
            |(loc, clip)| {
                let counters_send = Arc::clone(&counters_send);
                let shot = Arc::clone(&shot);
                let state = Arc::clone(&state);
                async move {
                    dice_single_tile(state, die_id, shot, loc, clip, counters_send)
                        .await
                        .log();
                }
            }
        ),
        tiling_state_broadcaster(die_id, counters_recv, rt_sender),
    );

    // should be no active links after foreach
    drop(Arc::try_unwrap(shot).unwrap());

    db::set_finished_at(&state.db, job_id)
        .await
        .context("failed mark job as finished")?;

    die::db::set_imported(&state.db, die.id)
        .await
        .context("failed update imported flag")?;

    state.rt_sender.send(RealtimeEvent::DieImportStateChange {
        die_id,
        state: DieImportState::Done,
    });
    Ok(())
}

async fn dice_single_tile(
    state: Arc<crate::State>,
    die_id: Uuid,
    shot: Arc<DynamicImage>,
    loc: TileLocation,
    clip: TileClip,
    counters: Arc<(AtomicU32, u32)>,
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
        &state.db,
        &format!("tile_{}_{}_{}.jpeg", loc.z, loc.x, loc.y),
        "image/jpeg",
        &buf,
    )
    .await
    .context("failed to create tile file")?;
    drop(buf);

    tiles::db::assign(&state.db, die_id, &loc, file_id)
        .await
        .context("failed to set tile")?;

    counters.0.fetch_add(1, Ordering::Relaxed);

    Ok(())
}

async fn tiling_state_broadcaster(
    die_id: Uuid,
    counters: Arc<(AtomicU32, u32)>,
    rt: broadcast::Sender<realtime::RealtimeEvent>,
) {
    let mut current = counters.0.load(Ordering::Relaxed);

    rt.send(RealtimeEvent::DieImportStateChange {
        die_id,
        state: DieImportState::TileWritten {
            current,
            total: counters.1,
        },
    });

    while current < counters.1 {
        tokio::time::sleep(Duration::from_millis(200)).await;
        current = counters.0.load(Ordering::Relaxed);

        rt.send(RealtimeEvent::DieImportStateChange {
            die_id,
            state: DieImportState::TileWritten {
                current,
                total: counters.1,
            },
        });
    }
}
