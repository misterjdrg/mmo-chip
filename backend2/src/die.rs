use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use uuid::Uuid;

use crate::APIResult;

#[derive(Serialize)]
pub struct Die {}

pub async fn list(state: State<Arc<crate::State>>) -> APIResult<Vec<Die>> {
    Ok(Json(vec![]))
}
pub async fn get(state: State<Arc<crate::State>>, Path(die_id): Path<Uuid>) -> APIResult<()> {
    todo!()
}
pub async fn delete(state: State<Arc<crate::State>>, Path(die_id): Path<Uuid>) -> APIResult<()> {
    todo!()
}
pub async fn import(state: State<Arc<crate::State>>) -> APIResult<()> {
    todo!()
}

macro_rules! die_param_get {
    ($kind: ident) => {
        pub async fn ${concat($kind, _get)}(
            state: State<Arc<crate::State>>,
            Path((die_id, id)): Path<(Uuid, Uuid)>,
        ) -> APIResult<()> {
            todo!()
        }
    };
}
macro_rules! die_param_delete {
    ($kind: ident) => {
        pub async fn ${concat($kind, _delete)}(
            state: State<Arc<crate::State>>,
            Path((die_id, id)): Path<(Uuid, Uuid)>,
        ) -> APIResult<()> {
            todo!()
        }
    };
}
macro_rules! die_param {
    ($kind: ident) => {
        die_param_get!($kind);
        die_param_delete!($kind);
    };
}

die_param!(cell);
die_param!(cell_type);
die_param!(net);
die_param!(grid);
die_param!(pin);
die_param!(annotation);
die_param!(roi);
die_param!(ignore);
die_param!(guide);
