use anyhow::Context;
use chrono::Local;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::DB;

#[derive(FromRow)]
pub struct File {
    pub id: Uuid,
    pub die_id: Uuid,
    pub name: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

pub async fn create(
    db: &DB,
    die_id: Uuid,
    name: &str,
    mime: &str,
    bytes: &[u8],
) -> anyhow::Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO files(id, die_id, name, mime, bytes, created_at) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(id)
        .bind(die_id)
        .bind(name)
        .bind(mime)
        .bind(bytes)
        .bind(Local::now())
        .execute(db)
        .await
        .context("failed to insert file")
        .map(|_| id)
}
pub async fn get(db: &DB, file_id: Uuid) -> anyhow::Result<Option<File>> {
    sqlx::query_as("SELECT id, die_id, name, mime, bytes FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(db)
        .await
        .context("failed to get file by id")
}
