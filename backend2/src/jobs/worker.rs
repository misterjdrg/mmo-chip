use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use anyhow::Context;
use futures::StreamExt;
use image::{DynamicImage, ImageReader, codecs::jpeg::JpegEncoder};
use tokio::{
    sync::{broadcast, mpsc},
    time::Instant,
};
use uuid::Uuid;

use crate::{
    die, file,
    jobs::db::JobKind,
    params::{
        self,
        domain::{Cell, CellType, IsParamKind, Rect},
    },
    realtime::{self, DieImportState, RealtimeEvent},
    tiles::{self, TileClip, TileLocation, TileTree},
    util::Log,
};

use super::db;

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

    let (clip, kind) = match true {
        _ if let Some(cell) = params::db::get::<Cell>(&state.db, die_id, owner_id).await? => {
            let cell_type = params::db::get::<CellType>(&state.db, die_id, cell.cell_type_id)
                .await?
                .context("no type for cell")?;

            (
                Rect {
                    x: cell.x,
                    y: cell.y,
                    width: cell_type.crop_rect.width,
                    height: cell_type.crop_rect.height,
                },
                Cell::KIND,
            )
        }
        _ if let Some(cell_type) =
            params::db::get::<CellType>(&state.db, die_id, owner_id).await? =>
        {
            let mut rect = cell_type.crop_rect;

            if rect.x == 0 && rect.y == 0 && rect.width > 0 {
                if let Some(c) = params::db::list::<Cell>(&state.db, die_id)
                    .await?
                    .into_iter()
                    .filter(|c| c.cell_type_id == cell_type.id)
                    .next()
                {
                    rect.x = c.x;
                    rect.y = c.y;
                }
            }

            (rect, CellType::KIND)
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

    let shot = tokio::task::spawn_blocking(move || -> anyhow::Result<Arc<DynamicImage>> {
        let mut decoder = ImageReader::new(io::Cursor::new(&file.bytes));
        decoder.no_limits();

        decoder = decoder
            .with_guessed_format()
            .context("failed to guess image format")?;

        Ok(Arc::new(
            decoder.decode().context("failed to decode image")?,
        ))
    })
    .await
    .context("failed at die decode task")?
    .context("failed to load image")?;

    let buf = tokio::task::spawn_blocking(move || {
        let cropped = shot.crop_imm(clip.x, clip.y, clip.width, clip.height);
        let mut buf = vec![];
        let encoder = JpegEncoder::new_with_quality(&mut buf, 90);
        cropped.write_with_encoder(encoder);
        drop(cropped);
        buf
    })
    .await
    .context("failed to join")?;

    let file_id = file::db::create(
        &state.db,
        die_id,
        &format!("clip_{owner_id}.jpeg"),
        "image/jpeg",
        &buf,
    )
    .await
    .context("failed to create clip file")?;
    drop(buf);

    tiles::db::set_clip(&state.db, die_id, owner_id, kind, file_id).await?;

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

    let shot = tokio::task::spawn_blocking(move || -> anyhow::Result<Arc<DynamicImage>> {
        let mut decoder = ImageReader::new(io::Cursor::new(&file.bytes));
        decoder.no_limits();

        decoder = decoder
            .with_guessed_format()
            .context("failed to guess image format")?;

        Ok(Arc::new(
            decoder.decode().context("failed to decode image")?,
        ))
    })
    .await
    .context("failed at die decode task")?
    .context("failed to load image")?;

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
        die_id,
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
