use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use uuid::Uuid;

use crate::APIResult;

#[derive(Serialize)]
pub struct ImportJob {}

pub async fn list_import(state: State<Arc<crate::State>>) -> APIResult<Vec<ImportJob>> {
    Ok(Json(vec![]))
}
pub async fn get_import(
    state: State<Arc<crate::State>>,
    Path(job_id): Path<Uuid>,
) -> APIResult<()> {
    todo!()
}
