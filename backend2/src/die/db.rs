use anyhow::Context;
use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use uuid::Uuid;

use crate::{
    DB,
    tiles::{DieInfo, LevelInfo},
    util::DateTime,
};

#[derive(Serialize, Deserialize)]
pub struct Die {
    id: Uuid,
    name: String,
    original_file_id: Uuid,
    width: u32,
    height: u32,
    max_zoom_level: u32,
    zoom_levels: Vec<ZoomLevel>,

    annotation_version: u32,
    annotation_revision: u32,

    created_at: DateTime,
    updated_at: DateTime,
}
#[derive(Serialize, Deserialize, FromRow)]
#[serde(rename = "camelCase")]
pub struct DieSummary {
    id: Uuid,
    name: String,
    original_file_name: String,
    width: u32,
    height: u32,
    tile_size: u32,
    max_zoom_level: u32,
    created_at: String,
    updated_at: String,
    //tile_progress: Option<DieTileProgress>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct DieTileProgress {
    total_tiles: u32,
    completed_tiles: u32,
    percentage: u32,
}

#[derive(Serialize, Deserialize)]
pub enum DieParamKind {
    Net,
    CellType,
    Cell,
    Grid,
    Pin,
    Annotation,
    ROI,
    Ignore,
    Guide,
}
#[derive(Serialize, Deserialize)]
pub struct ZoomLevel {}

pub async fn list_die_summaries(db: &DB) -> anyhow::Result<Vec<DieSummary>> {
    sqlx::query_as("SELECT d.id, d.name, f.name as original_file_name, d.width, d.height, d.tile_size, d.max_zoom_level, d.created_at, d.updated_at FROM dies as d JOIN files as f ON d.original_file_id = f.id")
        .fetch_all(db)
        .await
        .context("failed to list die summary")
}

pub fn get_by_id(db: &DB, die_id: Uuid) -> anyhow::Result<Option<Die>> {
    todo!()
}
pub fn get_params<'de, P: params::IsParamKind>(
    db: &DB,
    die_id: Uuid,
) -> anyhow::Result<Vec<WithId<P>>> {
    todo!()
}

pub async fn create_die(
    db: &DB,
    name: &str,
    file_id: Uuid,
    die_info: &DieInfo,
    zoom_levels: &[LevelInfo],
) -> anyhow::Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO dies(id, name, original_file_id, width, height, tile_size, max_zoom_level, zoom_levels, annotation_version, annotation_revision, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)")
        .bind(id)
        .bind(name)
        .bind(file_id)
        .bind(die_info.width)
        .bind(die_info.height)
        .bind(die_info.tile_size)
        .bind(die_info.max_zoom_level)
        .bind(Json(zoom_levels))
        .bind("v2")
        .bind("1")
        .bind(Local::now())
        .bind(Local::now())
        .execute(db)
        .await
        .context("failed to insert die")
        .map(|_| id)
}
pub async fn create_file(db: &DB, name: &str, mime: &str, bytes: &[u8]) -> anyhow::Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO files(id, name, mime, bytes) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(name)
        .bind(mime)
        .bind(bytes)
        .execute(db)
        .await
        .context("failed to insert file")
        .map(|_| id)
}

#[derive(Serialize, Deserialize)]
pub struct WithId<T> {
    id: Uuid,
    t: T,
}

pub mod params {
    use serde::{Deserialize, Serialize};

    use crate::die::db::DieParamKind;

    pub trait IsParamKind: Serialize + Deserialize<'static> {
        const KIND: DieParamKind;
    }

    #[derive(Serialize, Deserialize)]
    pub struct Net {}
    impl IsParamKind for Net {
        const KIND: DieParamKind = DieParamKind::Net;
    }
    #[derive(Serialize, Deserialize)]
    pub struct Cell {}
    impl IsParamKind for Cell {
        const KIND: DieParamKind = DieParamKind::Cell;
    }
    #[derive(Serialize, Deserialize)]
    pub struct CellType {}
    impl IsParamKind for CellType {
        const KIND: DieParamKind = DieParamKind::CellType;
    }
    #[derive(Serialize, Deserialize)]
    pub struct Grid {}
    impl IsParamKind for Grid {
        const KIND: DieParamKind = DieParamKind::Grid;
    }
    #[derive(Serialize, Deserialize)]
    pub struct Pin {}
    impl IsParamKind for Pin {
        const KIND: DieParamKind = DieParamKind::Pin;
    }
    #[derive(Serialize, Deserialize)]
    pub struct Annotation {}
    impl IsParamKind for Annotation {
        const KIND: DieParamKind = DieParamKind::Annotation;
    }
    #[derive(Serialize, Deserialize)]
    pub struct ROI {}
    impl IsParamKind for ROI {
        const KIND: DieParamKind = DieParamKind::ROI;
    }
    #[derive(Serialize, Deserialize)]
    pub struct Ignore {}
    impl IsParamKind for Ignore {
        const KIND: DieParamKind = DieParamKind::Ignore;
    }
    #[derive(Serialize, Deserialize)]
    pub struct Guide {}
    impl IsParamKind for Guide {
        const KIND: DieParamKind = DieParamKind::Guide;
    }
}
