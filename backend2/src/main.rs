use std::sync::Arc;

use anyhow::Context;
use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use serde_json::json;

mod annotations;
mod die;
mod jobs;
mod ml;
mod tiles;

pub type APIResult<T> = Result<Json<T>, APIError>;

pub struct State {}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    simple_logger::init_with_env().context("failed to init logger")?;

    log::info!("Starting rust backend on port: 11111");

    let router = Router::new()
        .route(
            "/api/dies/{die_id}/annotations",
            get(annotations::list).put(annotations::insert),
        )
        .route(
            "/api/dies/{die_id}/nets/{net_id}/nodes/{node_id}",
            delete(annotations::delete_node).put(annotations::insert_node),
        )
        .route(
            "/api/dies/{die_id}/nets/{net_id}/edges/{edge_id}",
            delete(annotations::delete_edge).put(annotations::insert_edge),
        )
        .route(
            "/api/dies/{die_id}/cell-types/{celltype_id}/layers/{layer_id}/shapes/{shape_id}",
            delete(annotations::delete_shape).put(annotations::insert_shape),
        )
        .route("/api/dies", get(die::list))
        .route("/api/dies/{die_id}", get(die::get).delete(die::delete))
        .route("/api/dies/import", post(die::import))
        .route("/api/health", get(async || Json(json!({"ok": true}))))
        .route("/api/ml/status", get(ml::status))
        .route("/api/ml/models", get(ml::models))
        .route("/api/ml/model", post(ml::set_model))
        .route("/api/ml/inference-jobs", get(ml::inference_jobs))
        .route("/api/dies/{die_id}/ml/job", get(ml::job_by_die))
        .route(
            "/api/dies/{die_id}/ml/job/start",
            post(ml::start_job_for_die),
        )
        .route("/api/dies/{die_id}/ml/job/stop", post(ml::stop_job_for_die))
        .route(
            "/api/dies/{die_id}/vias/tile/{z}/{x}/{y}",
            get(ml::get_vias_for_tile),
        )
        .route("/api/dies/{die_id}/vias/tiles", get(ml::get_vias_for_tile2))
        .route(
            "/api/dies/{die_id}/vias",
            get(ml::get_vias_for_tile_in_bbox),
        )
        .route(
            "/api/dies/{die_id}/heatmap/tile/{z}/{x}/{y}",
            get(ml::get_heatmap_for_tile),
        )
        .route("/api/dies/{die_id}/heatmap", get(ml::get_heatmap_in_bbox))
        .route("/api/ml/train", post(ml::train))
        .route("/api/ml/jobs/{job_id}", get(ml::get_job_by_id))
        .route(
            "/api/dies/{die_id}/cells/{cell_id}/crop",
            get(tiles::cell_crop),
        )
        .route(
            "/api/dies/{die_id}/cell-types/{celltype_id}/crop",
            get(tiles::celltypes_crop),
        )
        .route("/api/dies/{die_id}/tiles/{z}/{x}/{y}", get(tiles::get))
        .route("/api/dies/{die_id}/ml-export", post(ml::export))
        .with_state(Arc::new(State {}));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:11111")
        .await
        .context("failed to bind to port: 11111")?;

    axum::serve(listener, router.into_make_service())
        .await
        .context("fail axum::serve")?;

    Ok(())
}

#[derive(Debug)]
pub enum APIError {
    General(anyhow::Error),
}

impl From<anyhow::Error> for APIError {
    fn from(value: anyhow::Error) -> Self {
        Self::General(value)
    }
}

impl IntoResponse for APIError {
    fn into_response(self) -> axum::response::Response {
        log::error!("api error: {self:?}");
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self:#?}")).into_response()
    }
}
