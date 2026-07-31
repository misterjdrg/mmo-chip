use std::sync::Arc;

use anyhow::Context as _;
use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::APIResult;

pub async fn list(state: State<Arc<crate::State>>, Path(die_id): Path<Uuid>) -> APIResult<()> {
    todo!()
}

pub async fn insert(state: State<Arc<crate::State>>, Path(die_id): Path<Uuid>) -> APIResult<()> {
    todo!()
}

pub async fn insert_node(
    state: State<Arc<crate::State>>,
    Path((die_id, net_id, node_id)): Path<(Uuid, Uuid, Uuid)>,
) -> APIResult<()> {
    todo!()
}
pub async fn delete_node(
    state: State<Arc<crate::State>>,
    Path((die_id, net_id, node_id)): Path<(Uuid, Uuid, Uuid)>,
) -> APIResult<()> {
    todo!()
}
pub async fn insert_edge(
    state: State<Arc<crate::State>>,
    Path((die_id, net_id, edge_id)): Path<(Uuid, Uuid, Uuid)>,
) -> APIResult<()> {
    todo!()
}
pub async fn delete_edge(
    state: State<Arc<crate::State>>,
    Path((die_id, net_id, edge_id)): Path<(Uuid, Uuid, Uuid)>,
) -> APIResult<()> {
    todo!()
}
pub async fn insert_shape(
    state: State<Arc<crate::State>>,
    Path((die_id, celltype_id, layer_id, shape_id)): Path<(Uuid, Uuid, Uuid, Uuid)>,
) -> APIResult<()> {
    todo!()
}
pub async fn delete_shape(
    state: State<Arc<crate::State>>,
    Path((die_id, celltype_id, layer_id, shape_id)): Path<(Uuid, Uuid, Uuid, Uuid)>,
) -> APIResult<()> {
    todo!()
}
