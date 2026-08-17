use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    APIError, APIResult,
    die::db::{Die, DieParamKind},
    params::domain::{
        CellRotation, CellType, Grid, Guide, GuideKind, GuideKindLineAxis, HumanAnnotation,
        HumanAnnotationClass, HumanAnnotationShape, HumanAnnotationSource, IgnoreRect, IsParamKind,
        LayerKind, LayerShape, Net, Pin, ROI, Rect, Shape, ShapeLabel,
    },
    tiles::TileTree,
};
use crate::{die, params::domain::CellInstance};
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

    let cells = db::list::<CellInstance>(&state.db, die_id)
        .await
        .context("failed to get cells")?;
    let cells = cells
        .into_iter()
        .map(|n| n.into())
        .collect::<Vec<CellInstanceRequest>>();

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
            let param = TryInto::<$db_typ>::try_into((id, param)).context("failed to covert params")?;

            db::insert_or_update_content::<$db_typ>(&state.db, die_id, &param)
                .await
                .context("failed to insert or update param")?;

            let new_revision = die::db::increment_annotation_revision(&state.db, die_id)
                .await
                .context("failed to increment revision")?;

            let kind = $db_typ::KIND;
            if kind == DieParamKind::CellInstance || kind == DieParamKind::CellType {
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

die_param!(cell, CellInstance);
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
pub struct CellInstanceRequest {
    id: Uuid,
    cell_type_id: Uuid,
    x: u32,
    y: u32,
}
impl From<(Uuid, CellInstanceRequest)> for CellInstance {
    fn from(value: (Uuid, CellInstanceRequest)) -> Self {
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
impl From<CellInstance> for CellInstanceRequest {
    fn from(value: CellInstance) -> Self {
        Self {
            id: value.id,
            cell_type_id: value.cell_type_id,
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellTypeLayerShape {
    pub id: Uuid,
    pub kind: String,
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub label: Option<ShapeLabel>,
}
impl From<LayerShape> for CellTypeLayerShape {
    fn from(v: LayerShape) -> Self {
        match v.shape {
            Shape::Rect {
                x,
                y,
                width,
                height,
            } => Self {
                id: v.id,
                kind: "rect".to_string(),
                x: Some(x),
                y: Some(y),
                width: Some(width),
                height: Some(height),
                label: v.label,
            },

            _ => todo!(),
        }
    }
}
impl TryFrom<CellTypeLayerShape> for LayerShape {
    type Error = anyhow::Error;
    fn try_from(value: CellTypeLayerShape) -> Result<Self, Self::Error> {
        Ok(match value.kind.as_str() {
            "rect" => Self {
                id: value.id,
                shape: Shape::Rect {
                    x: value.x.context("no x")?,
                    y: value.y.context("no y")?,
                    width: value.width.context("no width")?,
                    height: value.height.context("no height")?,
                },
                label: value.label,
                forced_type: None,
                custom_name: None,
            },
            k => todo!("{k}"),
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellTypeRequest {
    id: Uuid,
    name: String,
    crop_rect: Rect,
    layers: HashMap<LayerKind, Vec<CellTypeLayerShape>>,
}
impl From<CellType> for CellTypeRequest {
    fn from(value: CellType) -> Self {
        Self {
            id: value.id,
            name: value.name,
            crop_rect: value.crop_rect,
            layers: value
                .layers
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().map(|s| s.into()).collect()))
                .collect(),
        }
    }
}
impl TryFrom<(Uuid, CellTypeRequest)> for CellType {
    type Error = anyhow::Error;
    fn try_from(value: (Uuid, CellTypeRequest)) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.1.id,
            name: value.1.name,
            crop_rect: value.1.crop_rect,
            layers: value
                .1
                .layers
                .into_iter()
                .map(|(k, v)| {
                    v.into_iter()
                        .map(|s| s.try_into())
                        .try_collect()
                        .map(|v| (k, v))
                })
                .try_collect()?,
            matched: false,
        })
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
    cells: Vec<CellInstanceRequest>,
    grids: Vec<GridRequest>,

    pins: Vec<PinRequest>,
    annotations: Vec<HumanAnnotationRequest>,
    rois: Vec<ROIRequest>,
    ignores: Vec<IgnoreRectRequest>,

    /// Cell-placement guides (RE aid, not ML)
    guides: Vec<GuideRequest>,
}
