use std::sync::Arc;

use anyhow::Context as _;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{APIResult, realtime::RealtimeEvent};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotations {
    version: u32,

    /// Monotonically incremented every time the annotations are written.
    /// Clients use it to spot stale caches when a WS notification arrives.
    #[serde(rename = "rev")]
    revision: u32,

    nets: Vec<Net>,
    cell_types: Vec<CellType>,
    cells: Vec<Cell>,
    grids: Vec<Grid>,

    pins: Vec<()>,
    annotations: Vec<()>,
    rios: Vec<()>,
    ignores: Vec<()>,

    /// Cell-placement guides (RE aid, not ML)
    guides: Vec<()>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Net {
    id: Uuid,
    name: String,
    node: Vec<NetNode>,
    edges: Vec<NetEdge>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetNode {
    id: Uuid,
    x: u32,
    y: u32,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetEdge {
    id: Uuid,
    from: Uuid,
    to: Uuid,

    /// None ⇒ unknown layer.
    /// Stamped per-segment at draw time
    /// editable later from the inspector.
    /// Persisted verbatim inside the net. */
    layer: Option<WireLayer>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WireLayer {
    Poly,
    Metal1,
    Metal2,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellType {
    id: Uuid,
    name: String,
    crop_rect: Rect,
    layers: Option<()>,

    /// Some(true) once promoted to a real, human-confirmed type via the merge-cells tool.
    /// None or Some(false) ⇒ an auto-created singleton placeholder
    /// (a cell that has only been labelled, not yet classified)
    /// grouped under "Unmatched". */
    matched: Option<bool>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    id: Uuid,
    cell_type_id: Uuid,
    x: u32,
    y: u32,

    /// Vertical mirror (existing).
    #[serde(rename = "flippedV")]
    flipped_vertical: Option<bool>,
    /// orizontal mirror, applied for display + ML export. Set while aligning
    /// a candidate in the merge-cells tool.
    #[serde(rename = "flippedH")]
    flipped_horizontal: Option<bool>,

    /// Orientation in degrees clockwise.
    rotation: u32,

    /// Set true once this instance has been classified & confirmed by the user
    /// via the merge-cells tool (drives the candidate "done" checkmark).
    merged: Option<bool>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rect {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Grid {
    id: Uuid,
    name: String,
    columns: Vec<GridColumn>,
    row_height: u32,
    column_offsets: Vec<u32>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GridColumn {
    x: u32,
    width: u32,
}

impl Default for Annotations {
    fn default() -> Self {
        Self {
            version: 2,
            revision: 0,
            nets: vec![],
            cell_types: vec![],
            cells: vec![],
            grids: vec![],
            annotations: vec![],
            guides: vec![],
            ignores: vec![],
            pins: vec![],
            rios: vec![],
        }
    }
}

pub async fn list(
    state: State<Arc<crate::State>>,
    Path(die_id): Path<Uuid>,
) -> APIResult<Annotations> {
    Ok(Json(Annotations::default()))
}

pub async fn insert(state: State<Arc<crate::State>>, Path(die_id): Path<Uuid>) -> APIResult<()> {
    state.rt_sender.send(RealtimeEvent::AnnotationChange {
        die_id,
        new_revision: 0,
    });
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
