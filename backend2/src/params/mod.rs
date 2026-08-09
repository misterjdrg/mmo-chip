use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    APIError, APIResult,
    die::db::{Die, DieParamKind},
    params::domain::{
        CellRotation, CellType, Grid, Guide, GuideKind, GuideKindLineAxis, HumanAnnotation,
        HumanAnnotationClass, HumanAnnotationShape, HumanAnnotationSource, IgnoreRect, IsParamKind,
        LayerKind, LayerShape, Net, Pin, ROI, Rect, Shape,
    },
    tiles::TileTree,
};
use crate::{die, params::domain::Cell};
use crate::{
    jobs,
    params::domain::{NetEdge, NetNode},
    realtime::RealtimeEvent,
};

pub mod db;
pub mod domain;

pub async fn list(
    state: State<Arc<crate::State>>,
    Path(die_id): Path<Uuid>,
) -> APIResult<Annotations> {
    let die = die::db::get(&state.db, die_id)
        .await
        .context("failed to get die")?
        .context("no die")?;

    let nets = db::list::<Net>(&state.db, die_id)
        .await
        .context("failed to get nets")?;
    let nets = nets
        .into_iter()
        .map(|n| n.into())
        .collect::<Vec<NetRequest>>();

    let cell_types = db::list::<CellType>(&state.db, die_id)
        .await
        .context("failed to get cell_types")?;
    let cell_types = cell_types
        .into_iter()
        .map(|n| n.into())
        .collect::<Vec<CellTypeRequest>>();

    let guides = db::list::<Guide>(&state.db, die_id)
        .await
        .context("failed to get guides")?;
    let guides = guides
        .into_iter()
        .map(|n| n.into())
        .collect::<Vec<GuideRequest>>();

    let pins = db::list::<Pin>(&state.db, die_id)
        .await
        .context("failed to get pins")?;
    let pins = pins
        .into_iter()
        .map(|n| n.into())
        .collect::<Vec<PinRequest>>();

    let annotations = db::list::<HumanAnnotation>(&state.db, die_id)
        .await
        .context("failed to get annotations")?;
    let annotations = annotations
        .into_iter()
        .map(|n| n.into())
        .collect::<Vec<HumanAnnotationRequest>>();

    let rois = db::list::<ROI>(&state.db, die_id)
        .await
        .context("failed to get rois")?;
    let rois = rois
        .into_iter()
        .map(|n| n.into())
        .collect::<Vec<ROIRequest>>();

    let ignores = db::list::<IgnoreRect>(&state.db, die_id)
        .await
        .context("failed to get ignores")?;
    let ignores = ignores
        .into_iter()
        .map(|n| n.into())
        .collect::<Vec<IgnoreRectRequest>>();

    let cells = db::list::<Cell>(&state.db, die_id)
        .await
        .context("failed to get cells")?;
    let cells = cells
        .into_iter()
        .map(|n| n.into())
        .collect::<Vec<CellRequest>>();

    let grids = db::list::<Grid>(&state.db, die_id)
        .await
        .context("failed to get grids")?;
    let grids = grids
        .into_iter()
        .map(|n| n.into())
        .collect::<Vec<GridRequest>>();

    Ok(Json(Annotations {
        version: 2,
        revision: die.annotation_revision,
        annotations,
        cell_types,
        cells,
        grids,
        guides,
        ignores,
        nets,
        pins,
        rois,
    }))
}

pub async fn insert(
    state: State<Arc<crate::State>>,
    Path(die_id): Path<Uuid>,
    Json(rq): Json<Annotations>,
) -> APIResult<()> {
    unimplemented!("unused")
}

pub async fn insert_node(
    state: State<Arc<crate::State>>,
    Path((die_id, net_id, node_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(node): Json<NetNode>,
) -> APIResult<ParamChangeResponse> {
    let mut net = db::get::<Net>(&state.db, die_id, net_id)
        .await
        .context("failed to get net")?
        .context("no net")?;

    net.nodes.push(node);

    db::update_content(&state.db, die_id, &net)
        .await
        .context("failed to update node")?;

    let new_revision = die::db::increment_annotation_revision(&state.db, die_id).await?;
    state.rt_sender.send(RealtimeEvent::AnnotationChange {
        die_id,
        new_revision,
    });

    Ok(Json(ParamChangeResponse::Ok {
        ok: true,
        new_revision,
    }))
}
pub async fn delete_node(
    state: State<Arc<crate::State>>,
    Path((die_id, net_id, node_id)): Path<(Uuid, Uuid, Uuid)>,
) -> APIResult<ParamChangeResponse> {
    let mut net = db::get::<Net>(&state.db, die_id, net_id)
        .await
        .context("failed to get net")?
        .context("no net")?;

    net.nodes.retain(|n| n.id != node_id);

    db::update_content(&state.db, die_id, &net)
        .await
        .context("failed to update node")?;

    let new_revision = die::db::increment_annotation_revision(&state.db, die_id).await?;
    state.rt_sender.send(RealtimeEvent::AnnotationChange {
        die_id,
        new_revision,
    });

    Ok(Json(ParamChangeResponse::Ok {
        ok: true,
        new_revision,
    }))
}
pub async fn insert_edge(
    state: State<Arc<crate::State>>,
    Path((die_id, net_id, edge_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(edge): Json<NetEdge>,
) -> APIResult<ParamChangeResponse> {
    let mut net = db::get::<Net>(&state.db, die_id, net_id)
        .await
        .context("failed to get net")?
        .context("no net")?;

    net.edges.push(edge);

    db::update_content(&state.db, die_id, &net)
        .await
        .context("failed to update node")?;

    let new_revision = die::db::increment_annotation_revision(&state.db, die_id).await?;
    state.rt_sender.send(RealtimeEvent::AnnotationChange {
        die_id,
        new_revision,
    });

    Ok(Json(ParamChangeResponse::Ok {
        ok: true,
        new_revision,
    }))
}
pub async fn delete_edge(
    state: State<Arc<crate::State>>,
    Path((die_id, net_id, edge_id)): Path<(Uuid, Uuid, Uuid)>,
) -> APIResult<ParamChangeResponse> {
    let mut net = db::get::<Net>(&state.db, die_id, net_id)
        .await
        .context("failed to get net")?
        .context("no net")?;

    net.edges.retain(|e| e.id != edge_id);

    db::update_content(&state.db, die_id, &net)
        .await
        .context("failed to update node")?;

    let new_revision = die::db::increment_annotation_revision(&state.db, die_id).await?;
    state.rt_sender.send(RealtimeEvent::AnnotationChange {
        die_id,
        new_revision,
    });

    Ok(Json(ParamChangeResponse::Ok {
        ok: true,
        new_revision,
    }))
}
pub async fn insert_shape(
    state: State<Arc<crate::State>>,
    Path((die_id, celltype_id, layer, shape_id)): Path<(Uuid, Uuid, String, Uuid)>,
    Json(rq): Json<LayerShape>,
) -> APIResult<ParamChangeResponse> {
    let layer = layer.parse::<LayerKind>()?;
    let mut cell_type = db::get::<CellType>(&state.db, die_id, celltype_id)
        .await?
        .context("no celltype")?;

    cell_type.layers.entry(layer).or_default().push(rq);

    db::update_content(&state.db, die_id, &cell_type)
        .await
        .context("failed to update shape")?;

    let new_revision = die::db::increment_annotation_revision(&state.db, die_id).await?;
    state.rt_sender.send(RealtimeEvent::AnnotationChange {
        die_id,
        new_revision,
    });

    Ok(Json(ParamChangeResponse::Ok {
        ok: true,
        new_revision,
    }))
}
pub async fn delete_shape(
    state: State<Arc<crate::State>>,
    Path((die_id, celltype_id, layer, shape_id)): Path<(Uuid, Uuid, String, Uuid)>,
) -> APIResult<ParamChangeResponse> {
    let layer = layer.parse::<LayerKind>()?;
    let mut cell_type = db::get::<CellType>(&state.db, die_id, celltype_id)
        .await?
        .context("no celltype")?;

    let layer = cell_type.layers.get_mut(&layer).context("no layer")?;
    if !layer.iter().any(|s| s.id == shape_id) {
        return Err(APIError::General(anyhow::anyhow!("no shape")));
    }
    layer.retain(|s| s.id != shape_id);

    db::update_content(&state.db, die_id, &cell_type)
        .await
        .context("failed to update shape")?;

    let new_revision = die::db::increment_annotation_revision(&state.db, die_id).await?;
    state.rt_sender.send(RealtimeEvent::AnnotationChange {
        die_id,
        new_revision,
    });

    Ok(Json(ParamChangeResponse::Ok {
        ok: true,
        new_revision,
    }))
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ParamChangeResponse {
    Ok {
        ok: bool,
        #[serde(rename = "rev")]
        new_revision: u32,
    },
    Err {
        error: String,
    },
}

macro_rules! die_param_put {
    ($kind: ident, $db_typ: ident) => {
        pub async fn ${concat($kind, _put)}(
            state: State<Arc<crate::State>>,
            Path((die_id, id)): Path<(Uuid, Uuid)>,
            Json(param): Json<${concat($db_typ, Request)}>,
        ) -> APIResult<ParamChangeResponse> {
            let param = Into::<$db_typ>::into((id, param));

            db::insert_or_update_content::<$db_typ>(&state.db, die_id, &param)
                .await
                .context("failed to insert or update param")?;

            let new_revision = die::db::increment_annotation_revision(&state.db, die_id)
                .await
                .context("failed to increment revision")?;

            let kind = $db_typ::KIND;
            if kind == DieParamKind::Cell || kind == DieParamKind::CellType {
                let job_id = jobs::db::create_clip_job(&state.db, die_id, id)
                    .await
                    .context("failed to create clip job")?;

                state
                    .job_sender
                    .send(job_id)
                    .await
                    .context("failed to push job on the queue")?;
            }

            state.rt_sender.send(RealtimeEvent::AnnotationChange {
                die_id,
                new_revision,
            });

            Ok(Json(ParamChangeResponse::Ok { ok: true, new_revision }))
        }
    };
}
macro_rules! die_param_delete {
    ($kind: ident, $db_typ: ident) => {
        pub async fn ${concat($kind, _delete)}(
            state: State<Arc<crate::State>>,
            Path((die_id, id)): Path<(Uuid, Uuid)>,
        ) -> APIResult<ParamChangeResponse> {
            let res = db::delete_one::<$db_typ>(&state.db, die_id, id)
                .await
                .context("failed to delete param")?;

            let new_revision = die::db::increment_annotation_revision(&state.db, die_id)
                .await
                .context("failed to increment revision")?;

            state.rt_sender.send(RealtimeEvent::AnnotationChange {
                die_id,
                new_revision,
            });

            Ok(Json(ParamChangeResponse::Ok { ok: true, new_revision }))
        }
    };
}
macro_rules! die_param {
    ($kind: ident, $db_typ: ident) => {
        die_param_put!($kind, $db_typ);
        die_param_delete!($kind, $db_typ);
    };
}

die_param!(cell, Cell);
die_param!(cell_type, CellType);
die_param!(net, Net);
die_param!(grid, Grid);
die_param!(pin, Pin);
die_param!(annotation, HumanAnnotation);
die_param!(roi, ROI);
die_param!(ignore_rect, IgnoreRect);
die_param!(guide, Guide);

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellRequest {
    id: Uuid,
    cell_type_id: Uuid,
    x: u32,
    y: u32,
}
impl From<(Uuid, CellRequest)> for Cell {
    fn from(value: (Uuid, CellRequest)) -> Self {
        Self {
            id: value.1.id,
            cell_type_id: value.1.cell_type_id,
            x: value.1.x,
            y: value.1.y,
            flipped_vertical: false,
            flipped_horizontal: false,
            rotation: CellRotation::Rot0,
            merged: false,
        }
    }
}
impl From<Cell> for CellRequest {
    fn from(value: Cell) -> Self {
        Self {
            id: value.id,
            cell_type_id: value.cell_type_id,
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellTypeRequest {
    id: Uuid,
    name: String,
    crop_rect: Rect,
}
impl From<CellType> for CellTypeRequest {
    fn from(value: CellType) -> Self {
        Self {
            id: value.id,
            name: value.name,
            crop_rect: value.crop_rect,
        }
    }
}
impl From<(Uuid, CellTypeRequest)> for CellType {
    fn from(value: (Uuid, CellTypeRequest)) -> Self {
        Self {
            id: value.1.id,
            name: value.1.name,
            crop_rect: value.1.crop_rect,
            layers: HashMap::new(),
            matched: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NetRequest {
    id: Uuid,
    name: String,
    nodes: Vec<NetNode>,
    edges: Vec<NetEdge>,
}
impl From<Net> for NetRequest {
    fn from(value: Net) -> Self {
        Self {
            id: value.id,
            name: value.name,
            nodes: value.nodes,
            edges: value.edges,
        }
    }
}
impl From<(Uuid, NetRequest)> for Net {
    fn from(value: (Uuid, NetRequest)) -> Self {
        Self {
            id: value.1.id,
            name: value.1.name,
            nodes: value.1.nodes,
            edges: value.1.edges,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GridRequest {
    id: Uuid,
}
impl From<Grid> for GridRequest {
    fn from(value: Grid) -> Self {
        todo!()
    }
}
impl From<(Uuid, GridRequest)> for Grid {
    fn from(value: (Uuid, GridRequest)) -> Self {
        todo!()
    }
}

#[derive(Serialize, Deserialize)]
pub struct PinRequest {
    id: Uuid,
    name: String,
    x: u32,
    y: u32,
    pin: u32,
}
impl From<Pin> for PinRequest {
    fn from(value: Pin) -> Self {
        Self {
            id: value.id,
            name: value.name,
            pin: value.pin,
            x: value.x,
            y: value.y,
        }
    }
}
impl From<(Uuid, PinRequest)> for Pin {
    fn from(value: (Uuid, PinRequest)) -> Self {
        Self {
            id: value.1.id,
            name: value.1.name,
            pin: value.1.pin,
            x: value.1.x,
            y: value.1.y,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HumanAnnotationRequest {
    id: Uuid,
    class: HumanAnnotationClass,
    source: Option<HumanAnnotationSource>,
    geometry: HumanAnnotationShape,
}
impl From<HumanAnnotation> for HumanAnnotationRequest {
    fn from(value: HumanAnnotation) -> Self {
        Self {
            id: value.id,
            class: value.class,
            source: value.source,
            geometry: value.geometry,
        }
    }
}
impl From<(Uuid, HumanAnnotationRequest)> for HumanAnnotation {
    fn from(value: (Uuid, HumanAnnotationRequest)) -> Self {
        Self {
            id: value.1.id,
            class: value.1.class,
            source: value.1.source,
            geometry: value.1.geometry,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ROIRequest {
    id: Uuid,
    classes: Vec<HumanAnnotationClass>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}
impl From<ROI> for ROIRequest {
    fn from(value: ROI) -> Self {
        Self {
            id: value.id,
            classes: value.classes,
            x: value.rect.x,
            y: value.rect.y,
            width: value.rect.width,
            height: value.rect.height,
        }
    }
}
impl From<(Uuid, ROIRequest)> for ROI {
    fn from(value: (Uuid, ROIRequest)) -> Self {
        Self {
            id: value.1.id,
            rect: Rect {
                x: value.1.x,
                y: value.1.y,
                width: value.1.width,
                height: value.1.height,
            },
            classes: value.1.classes,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IgnoreRectRequest {
    id: Uuid,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}
impl From<IgnoreRect> for IgnoreRectRequest {
    fn from(value: IgnoreRect) -> Self {
        Self {
            id: value.id,
            x: value.rect.x,
            y: value.rect.y,
            width: value.rect.width,
            height: value.rect.height,
        }
    }
}
impl From<(Uuid, IgnoreRectRequest)> for IgnoreRect {
    fn from(value: (Uuid, IgnoreRectRequest)) -> Self {
        Self {
            id: value.1.id,
            rect: Rect {
                x: value.1.x,
                y: value.1.y,
                width: value.1.width,
                height: value.1.height,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum GuideRequest {
    #[serde(rename = "line")]
    Line {
        id: Uuid,
        axis: GuideKindLineAxis,
        pos: u32,
    },
    #[serde(rename = "segment")]
    Segment {
        id: Uuid,
        x1: u32,
        y1: u32,
        x2: u32,
        y2: u32,
    },
}
impl From<Guide> for GuideRequest {
    fn from(value: Guide) -> Self {
        match value.kind {
            GuideKind::Line { axis, pos } => Self::Line {
                id: value.id,
                axis,
                pos,
            },
            GuideKind::Segment { x1, y1, x2, y2 } => Self::Segment {
                id: value.id,
                x1,
                y1,
                x2,
                y2,
            },
        }
    }
}
impl From<(Uuid, GuideRequest)> for Guide {
    fn from(value: (Uuid, GuideRequest)) -> Self {
        match value.1 {
            GuideRequest::Line { id, axis, pos } => Self {
                id,
                kind: GuideKind::Line { axis, pos },
            },
            GuideRequest::Segment { id, x1, y1, x2, y2 } => Self {
                id,
                kind: GuideKind::Segment { x1, y1, x2, y2 },
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotations {
    version: u32,

    /// Monotonically incremented every time the annotations are written.
    /// Clients use it to spot stale caches when a WS notification arrives.
    #[serde(rename = "rev")]
    revision: u32,

    nets: Vec<NetRequest>,
    cell_types: Vec<CellTypeRequest>,
    cells: Vec<CellRequest>,
    grids: Vec<GridRequest>,

    pins: Vec<PinRequest>,
    annotations: Vec<HumanAnnotationRequest>,
    rois: Vec<ROIRequest>,
    ignores: Vec<IgnoreRectRequest>,

    /// Cell-placement guides (RE aid, not ML)
    guides: Vec<GuideRequest>,
}
