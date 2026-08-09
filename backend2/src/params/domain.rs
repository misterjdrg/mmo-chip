use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::die::db::DieParamKind;

pub trait IsParamKind: Serialize + DeserializeOwned + Send + Unpin + 'static {
    const KIND: DieParamKind;

    fn get_id(&self) -> Uuid;
}

macro_rules! impl_is_param_kind {
    ($typ: ident) => {
        impl IsParamKind for $typ {
            const KIND: DieParamKind = DieParamKind::$typ;

            fn get_id(&self) -> Uuid {
                self.id
            }
        }
    };
}

impl_is_param_kind!(Net);
impl_is_param_kind!(Cell);
impl_is_param_kind!(CellType);
impl_is_param_kind!(Grid);
impl_is_param_kind!(Pin);
impl_is_param_kind!(ROI);
impl_is_param_kind!(IgnoreRect);
impl_is_param_kind!(Guide);
impl_is_param_kind!(HumanAnnotation);

#[derive(Serialize, Deserialize)]
pub struct NetNode {
    pub id: Uuid,
    pub x: f32,
    pub y: f32,
}
#[derive(Serialize, Deserialize)]
pub enum WireLayer {
    Metal1,
    Metal2,
    Poly,
}
#[derive(Serialize, Deserialize)]
pub struct NetEdge {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Uuid,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<WireLayer>,
}

#[derive(Serialize, Deserialize)]
pub struct Net {
    pub id: Uuid,
    pub name: String,
    pub nodes: Vec<NetNode>,
    pub edges: Vec<NetEdge>,
}

#[derive(Serialize, Deserialize)]
pub enum CellRotation {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
}
#[derive(Serialize, Deserialize)]
pub struct Cell {
    pub id: Uuid,
    pub cell_type_id: Uuid,
    pub x: u32,
    pub y: u32,
    pub flipped_vertical: bool,
    pub flipped_horizontal: bool,
    pub rotation: CellRotation,
    pub merged: bool,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LayerKind {
    Diffusion,
    Polysilicon,
    Metal1,
    Metal2,
    Contact,
    Via,
    WireHitbox,
}
impl FromStr for LayerKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "diffusion" => Self::Diffusion,
            "polysilicon" => Self::Polysilicon,
            "metal1" => Self::Metal1,
            "metal2" => Self::Metal2,
            "contact" => Self::Contact,
            "via1" => Self::Via,
            "wire_hitbox" => Self::WireHitbox,
            _ => anyhow::bail!("invalid layer: {s}"),
        })
    }
}
#[derive(Serialize, Deserialize)]
pub struct LayerShape {
    pub id: Uuid,
    pub shape: Shape,
    pub label: Option<ShapeLabel>,
    pub forced_type: Option<DiffusionType>,
    pub custom_name: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Shape {
    #[serde(rename = "rect")]
    Rect {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    #[serde(rename = "line")]
    Line {
        x1: u32,
        x2: u32,
        y1: u32,
        y2: u32,
        width: u32,
    },
    #[serde(rename = "point")]
    Point { x: u32, y: u32, width: u32 },
    #[serde(rename = "circle")]
    Circle { x: u32, y: u32, radius: u32 },
    #[serde(rename = "polygon")]
    Polygon { points: Vec<(u32, u32)> },
}
#[derive(Serialize, Deserialize)]
pub enum DiffusionType {
    P,
    N,
}
#[derive(Serialize, Deserialize)]
pub enum ShapeLabel {
    Vcc,
    Gnd,
    IO,
    Input,
    Output,
}

#[derive(Serialize, Deserialize)]
pub struct CellType {
    pub id: Uuid,
    pub name: String,
    pub crop_rect: Rect,
    pub layers: HashMap<LayerKind, Vec<LayerShape>>,
    pub matched: bool,
}
#[derive(Serialize, Deserialize)]
pub struct GridColumn {
    pub x: u32,
    pub width: u32,
}
#[derive(Serialize, Deserialize)]
pub struct Grid {
    pub id: Uuid,
    pub name: String,
    pub columns: Vec<GridColumn>,
    pub row_height: u32,
    pub column_offsets: Vec<u32>,
}
#[derive(Serialize, Deserialize)]
pub struct Pin {
    pub id: Uuid,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub pin: u32,
}

#[derive(Serialize, Deserialize)]
pub enum HumanAnnotationClass {
    #[serde(rename = "point_via")]
    PointVia,
    #[serde(rename = "irregular_via")]
    IrregularVia,
    #[serde(rename = "trace")]
    Trace,
}

#[derive(Serialize, Deserialize)]
pub enum HumanAnnotationSource {
    #[serde(rename = "human")]
    Human,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum HumanAnnotationShape {
    #[serde(rename = "point")]
    Point { x: u32, y: u32 },

    #[serde(rename = "rectangle")]
    Rectangle {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    #[serde(rename = "polygon")]
    Polygon { points: Vec<Point> },
}
#[derive(Serialize, Deserialize)]
pub struct Point {
    x: u32,
    y: u32,
}

#[derive(Serialize, Deserialize)]
pub struct HumanAnnotation {
    pub id: Uuid,
    pub class: HumanAnnotationClass,
    pub geometry: HumanAnnotationShape,
    pub source: Option<HumanAnnotationSource>,
}
#[derive(Serialize, Deserialize)]
pub struct ROI {
    pub id: Uuid,
    pub rect: Rect,
    pub classes: Vec<HumanAnnotationClass>,
}
#[derive(Serialize, Deserialize)]
pub struct IgnoreRect {
    pub id: Uuid,
    pub rect: Rect,
}
#[derive(Serialize, Deserialize)]
pub enum GuideKindLineAxis {
    #[serde(rename = "x")]
    X,
    #[serde(rename = "y")]
    Y,
}
#[derive(Serialize, Deserialize)]
pub enum GuideKind {
    Line { axis: GuideKindLineAxis, pos: u32 },
    Segment { x1: u32, y1: u32, x2: u32, y2: u32 },
}
#[derive(Serialize, Deserialize)]
pub struct Guide {
    pub id: Uuid,
    pub kind: GuideKind,
}
