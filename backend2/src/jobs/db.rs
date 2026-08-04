use anyhow::Context;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use uuid::Uuid;

use crate::{DB, file::db::File};

#[derive(Deserialize, Serialize, Debug)]
pub enum JobKind {
    ImportDie { die_id: Uuid },
}
#[derive(Deserialize, Serialize, Debug)]
pub enum JobStatus {
    Queued,
    Done,
    Failed,
}

#[derive(Debug, FromRow)]
pub struct Job {
    pub id: Uuid,
    pub kind: Json<JobKind>,
    pub status: Json<JobStatus>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

pub async fn create_import_die_job(db: &DB, die_id: Uuid) -> anyhow::Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO jobs(id, kind, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(Json(JobKind::ImportDie { die_id }))
    .bind(Json(JobStatus::Queued))
    .bind(Local::now())
    .bind(Local::now())
    .execute(db)
    .await
    .context("failed to insert job")
    .map(|_| id)
}

pub async fn get(db: &DB, job_id: Uuid) -> anyhow::Result<Option<Job>> {
    sqlx::query_as("SELECT id, kind, status, created_at, updated_at, started_at, finished_at FROM jobs WHERE id = $1")
        .bind(job_id)
        .fetch_optional(db)
        .await
        .context("failed to get job")
}
pub async fn set_started_at(db: &DB, job_id: Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE jobs SET started_at = $2 WHERE id = $1")
        .bind(job_id)
        .bind(Local::now())
        .execute(db)
        .await
        .context("failed to set started_at")
        .map(|_| ())
}
pub async fn set_finished_at(db: &DB, job_id: Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE jobs SET finished_at = $2 WHERE id = $1")
        .bind(job_id)
        .bind(Local::now())
        .execute(db)
        .await
        .context("failed to set finished_at")
        .map(|_| ())
}
pub async fn set_status(db: &DB, job_id: Uuid, status: JobStatus) -> anyhow::Result<()> {
    sqlx::query("UPDATE jobs SET status = $2 WHERE id = $1")
        .bind(job_id)
        .bind(Json(status))
        .execute(db)
        .await
        .context("failed to set status")
        .map(|_| ())
}
