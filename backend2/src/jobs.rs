use std::sync::Arc;

use axum::extract::{Path, State};
use uuid::Uuid;

use crate::APIResult;

pub async fn list(state: State<Arc<crate::State>>) -> APIResult<()> {
    todo!()
}
pub async fn get(state: State<Arc<crate::State>>, Path(job_id): Path<Uuid>) -> APIResult<()> {
    todo!()
}
