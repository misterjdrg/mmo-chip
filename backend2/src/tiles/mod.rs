use std::{iter, sync::Arc};

use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::APIResult;

mod domain;
pub use domain::*;

pub async fn cell_crop(
    state: State<Arc<crate::State>>,
    Path((die_id, cell_id)): Path<(Uuid, Uuid)>,
) -> APIResult<()> {
    todo!()
}

pub async fn celltypes_crop(
    state: State<Arc<crate::State>>,
    Path((die_id, celltype_id)): Path<(Uuid, Uuid)>,
) -> APIResult<()> {
    todo!()
}

pub async fn get(
    state: State<Arc<crate::State>>,
    Path((die_id, z, x, y)): Path<(Uuid, u32, u32, u32)>,
) -> APIResult<()> {
    todo!()
}
