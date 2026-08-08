use anyhow::Context;
use chrono::Local;
use uuid::Uuid;

use crate::{DB, file::db::File, tiles::TileLocation};

pub async fn assign(
    db: &DB,
    die_id: Uuid,
    loc: &TileLocation,
    file_id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO tiles(die_id, z, x, y, file_id, created_at) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(die_id)
    .bind(loc.z)
    .bind(loc.x)
    .bind(loc.y)
    .bind(file_id)
    .bind(Local::now())
    .execute(db)
    .await
    .context("failed to assign tile")
    .map(|_| ())
}

pub async fn get_file(db: &DB, die_id: Uuid, loc: &TileLocation) -> anyhow::Result<Option<File>> {
    sqlx::query_as("SELECT f.id, f.die_id, f.name, f.mime, f.bytes FROM tiles as t JOIN files as f ON f.id = t.file_id WHERE t.die_id = $1 AND t.z = $2 AND t.x = $3 AND t.y = $4")
        .bind(die_id)
        .bind(loc.z)
        .bind(loc.x)
        .bind(loc.y)
        .fetch_optional(db)
        .await
        .context("failed to get tile file")
}
