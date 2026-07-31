use std::sync::Arc;

use axum::extract::{Path, State};
use uuid::Uuid;

use crate::APIResult;

pub async fn status(state: State<Arc<crate::State>>) -> APIResult<()> {
    todo!()
}
pub async fn models(state: State<Arc<crate::State>>) -> APIResult<()> {
    todo!()
}
pub async fn set_model(state: State<Arc<crate::State>>) -> APIResult<()> {
    todo!()
}
pub async fn inference_jobs(state: State<Arc<crate::State>>) -> APIResult<()> {
    todo!()
}
pub async fn job_by_die(
    state: State<Arc<crate::State>>,
    Path(die_id): Path<Uuid>,
) -> APIResult<()> {
    todo!()
}

pub async fn start_job_for_die(
    state: State<Arc<crate::State>>,
    Path(die_id): Path<Uuid>,
) -> APIResult<()> {
    todo!()
}

pub async fn stop_job_for_die(
    state: State<Arc<crate::State>>,
    Path(die_id): Path<Uuid>,
) -> APIResult<()> {
    todo!()
}
pub async fn get_vias_for_tile(
    state: State<Arc<crate::State>>,
    Path((die_id, z, x, y)): Path<(Uuid, u32, u32, u32)>,
) -> APIResult<()> {
    todo!()
}

pub async fn get_vias_for_tile2(state: State<Arc<crate::State>>) -> APIResult<()> {
    todo!()
}
pub async fn get_vias_for_tile_in_bbox(state: State<Arc<crate::State>>) -> APIResult<()> {
    todo!()
}

pub async fn get_heatmap_for_tile(
    state: State<Arc<crate::State>>,
    Path((die_id, z, x, y)): Path<(Uuid, u32, u32, u32)>,
) -> APIResult<()> {
    todo!()
}
pub async fn get_heatmap_in_bbox(
    state: State<Arc<crate::State>>,
    Path(die_id): Path<Uuid>,
) -> APIResult<()> {
    todo!()
}

pub async fn train(state: State<Arc<crate::State>>) -> APIResult<()> {
    todo!()
}

pub async fn get_job_by_id(
    state: State<Arc<crate::State>>,
    Path(job_id): Path<Uuid>,
) -> APIResult<()> {
    todo!()
}

pub async fn export(state: State<Arc<crate::State>>, Path(die_id): Path<Uuid>) -> APIResult<()> {
    todo!()
}
