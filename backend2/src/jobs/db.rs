use anyhow::Context;
use chrono::Local;
use sqlx::types::Json;
use uuid::Uuid;

use crate::{DB, jobs::JobKind};

pub async fn create_job(db: &DB, kind: &JobKind) -> anyhow::Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO jobs(id, kind, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(Json(kind))
    .bind("status")
    .bind(Local::now())
    .bind(Local::now())
    .execute(db)
    .await
    .context("failed to insert job")
    .map(|_| id)
}
