#![feature(macro_metavar_expr_concat)]
#![allow(unused)]

use std::{env, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context;
use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use serde_json::json;
use sqlx::{ConnectOptions, Sqlite, sqlite::SqliteConnectOptions};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use crate::{realtime::RealtimeEvent, util::RouterExt};

mod die;
mod file;
mod jobs;
mod ml;
mod params;
mod realtime;
mod tiles;
mod util;

pub type APIResult<T> = Result<Json<T>, APIError>;
pub type DB = sqlx::Pool<Sqlite>;

pub struct State {
    config: Config,
    db: DB,
    job_sender: mpsc::Sender<Uuid>,
    ml_backend: ml::Backend,

    rt_sender: broadcast::Sender<RealtimeEvent>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    simple_logger::init_with_env().context("failed to init logger")?;

    let config = Config::from_env();

    let db = sqlx::sqlite::SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .create_if_missing(true)
            .log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(5))
            .filename(&config.db),
    )
    .await
    .context("failed to connect with db")?;
    //let db = sqlx::sqlite::SqlitePool::connect(":memory:")
    //    .await
    //    .context("failed to connect with db")?;

    sqlx::migrate!()
        .run(&db)
        .await
        .context("failed to run migrations on db")?;

    let ml_backend = ml::Backend::new_remote(&config).context("failed to create ml backend")?;
    let (job_sender, job_recv) = mpsc::channel(16);

    tokio::spawn(jobs::worker(config.clone(), db.clone(), job_recv));

    let (rt_sender, _) = broadcast::channel(16);

    log::info!("Starting rust backend on port: {}", config.port);

    let router = Router::new()
        .route(
            "/api/dies/{die_id}/annotations",
            get(params::list).put(params::insert),
        )
        .route(
            "/api/dies/{die_id}/nets/{net_id}/nodes/{node_id}",
            delete(params::delete_node).put(params::insert_node),
        )
        .route(
            "/api/dies/{die_id}/nets/{net_id}/edges/{edge_id}",
            delete(params::delete_edge).put(params::insert_edge),
        )
        .route(
            "/api/dies/{die_id}/cell-types/{celltype_id}/layers/{layer_id}/shapes/{shape_id}",
            delete(params::delete_shape).put(params::insert_shape),
        )
        .route("/api/dies", get(die::list))
        .route("/api/dies/{die_id}", get(die::get).delete(die::delete))
        .route("/api/dies/import", post(jobs::die_import))
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
        .route("/api/import-jobs", get(jobs::list_import))
        .route("/api/import-jobs/{job_id}", get(jobs::get_import))
        .die_param("cells", params::cell_put, params::cell_delete)
        .die_param(
            "cell-types",
            params::cell_type_put,
            params::cell_type_delete,
        )
        .die_param("nets", params::net_put, params::net_delete)
        .die_param("grids", params::grid_put, params::grid_delete)
        .die_param("pins", params::pin_put, params::pin_delete)
        .die_param(
            "annotations",
            params::annotation_put,
            params::annotation_delete,
        )
        .die_param("rois", params::roi_put, params::roi_delete)
        .die_param(
            "ignores",
            params::ignore_rect_put,
            params::ignore_rect_delete,
        )
        .die_param("guides", params::guide_put, params::guide_delete)
        .route("/api/ws", get(realtime::websocket))
        .with_state(Arc::new(State {
            config: config.clone(),
            db,
            job_sender,
            ml_backend,

            rt_sender,
        }))
        .layer(DefaultBodyLimit::disable());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .with_context(|| format!("failed to bind to port: {}", config.port))?;

    axum::serve(listener, router.into_make_service())
        .await
        .context("fail axum::serve")
}

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub db: PathBuf,
    pub tile_size: u32,
    pub limit_input_pixels: Option<u32>,
    pub tile_concurency: u32,

    pub ml_sidecar_url: String,
    pub ml_predict_pad: u32,
}

impl Config {
    fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .ok()
                .as_deref()
                .unwrap_or("3001")
                .parse()
                .unwrap(),
            db: env::var("CHIPTOOL_DB")
                .ok()
                .as_deref()
                .unwrap_or("./db.sqlite")
                .parse()
                .unwrap(),
            tile_size: env::var("CHIPTOOL_TILE_SIZE")
                .ok()
                .as_deref()
                .unwrap_or("512")
                .parse()
                .unwrap(),
            limit_input_pixels: env::var("CHIPTOOL_LIMIT_INPUT_PIXELS")
                .ok()
                .map(|v| v.parse().unwrap()),
            tile_concurency: env::var("CHIPTOOL_TILE_CONCURRENCY")
                .ok()
                .as_deref()
                .unwrap_or("4")
                .parse()
                .unwrap(),

            ml_sidecar_url: env::var("CHIPTOOL_ML_SIDECAR_URL")
                .ok()
                .as_deref()
                .unwrap_or("http://127.0.0.1:8001")
                .to_string(),
            ml_predict_pad: env::var("CHIPTOOL_ML_PREDICT_PAD")
                .ok()
                .as_deref()
                .unwrap_or("128")
                .parse()
                .unwrap(),
        }
    }
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
