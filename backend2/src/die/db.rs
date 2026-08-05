use anyhow::Context;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use uuid::Uuid;

use crate::{
    DB,
    tiles::{LevelInfo, TileTree},
};

#[derive(FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Die {
    pub id: Uuid,
    pub name: String,
    pub original_file_id: Uuid,
    pub original_file_name: String,
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
    pub max_zoom_level: u32,
    #[serde(rename = "levels")]
    pub zoom_levels: Json<Vec<ZoomLevel>>,

    pub annotation_version: u32,

    /// Monotonically incremented every time the annotations are written.
    /// Clients use it to spot stale caches when a WS notification arrives.
    pub annotation_revision: u32,
    pub ml_config: Json<Option<MLConfig>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MLConfig {
    /// Via radius in source px → Gaussian sigma = radius * 0.5. Chip-global
    pub point_via_size: u32,
    /// Default trace stroke width in source px. Chip-global
    pub trace_width: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct ZoomLevel {
    z: u32,
    width: u32,
    height: u32,
    columns: u32,
    rows: u32,
    scale: u32,
}

pub async fn all_dies(db: &DB) -> anyhow::Result<Vec<Die>> {
    sqlx::query_as("SELECT d.id, d.name, d.original_file_id, f.name as original_file_name, d.width, d.height, d.tile_size, d.max_zoom_level, d.zoom_levels, d.annotation_version, d.annotation_revision, d.ml_config, d.created_at, d.updated_at FROM dies as d JOIN files as f ON d.original_file_id = f.id")
        .fetch_all(db)
        .await
        .context("failed to list all dies")
}

pub async fn get(db: &DB, die_id: Uuid) -> anyhow::Result<Option<Die>> {
    sqlx::query_as("SELECT d.id, d.name, d.original_file_id, f.name as original_file_name, d.width, d.height, d.tile_size, d.max_zoom_level, d.zoom_levels, d.annotation_version, d.annotation_revision, d.ml_config, d.created_at, d.updated_at FROM dies as d JOIN files as f ON d.original_file_id = f.id WHERE d.id = $1")
        .bind(die_id)
        .fetch_optional(db)
        .await
        .context("failed to get die by id")
}
pub async fn delete(db: &DB, die_id: Uuid) -> anyhow::Result<bool> {
    sqlx::query("DELETE FROM dies WHERE id = $1")
        .bind(die_id)
        .execute(db)
        .await
        .context("failed to delete die")
        .map(|r| r.rows_affected() > 0)
}
pub async fn get_params<'de, P: params::IsParamKind>(
    db: &DB,
    die_id: Uuid,
) -> anyhow::Result<Vec<WithId<P>>> {
    todo!()
}

pub async fn create(
    db: &DB,
    name: &str,
    file_id: Uuid,
    tree: &TileTree,
    zoom_levels: &[LevelInfo],
) -> anyhow::Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO dies(id, name, original_file_id, width, height, tile_size, max_zoom_level, zoom_levels, annotation_version, annotation_revision, ml_config, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)")
        .bind(id)
        .bind(name)
        .bind(file_id)
        .bind(tree.width)
        .bind(tree.height)
        .bind(tree.tile_size)
        .bind(tree.max_zoom_level)
        .bind(Json(zoom_levels))
        .bind(2)
        .bind(0)
        .bind(Json(None::<MLConfig>))
        .bind(Local::now())
        .bind(Local::now())
        .execute(db)
        .await
        .context("failed to insert die")
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
