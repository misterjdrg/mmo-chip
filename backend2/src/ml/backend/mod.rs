use uuid::Uuid;

use crate::{
    Config,
    ml::domain::{InferenceJob, Model, ResetModel, Status},
    tiles::TileLocation,
    util::AABB,
};

pub mod local;
pub mod remote;

pub enum Backend {
    Remote(remote::RemoteBackend),
    Local(local::LocalBackend),
}
impl Backend {
    pub fn new_remote(config: &Config) -> anyhow::Result<Self> {
        Ok(Self::Remote(remote::RemoteBackend::new(config)?))
    }

    pub fn status(&self) -> anyhow::Result<Status> {
        match self {
            Self::Remote(b) => b.status(),
            Self::Local(b) => b.status(),
        }
    }
    pub fn models(&self) -> anyhow::Result<Vec<Model>> {
        match self {
            Self::Remote(b) => b.models(),
            Self::Local(b) => b.models(),
        }
    }

    /// Switches the sidecar's resident checkpoint. Every cached prediction is
    /// tied to the old checkpoint, so we wipe the prediction cache + ML jobs.
    pub fn reset_model(&self, name: &str) -> anyhow::Result<ResetModel> {
        match self {
            Self::Remote(b) => b.reset_model(name),
            Self::Local(b) => b.reset_model(name),
        }
    }

    /// All per-die inference jobs — drives the library page's status badges.
    pub fn inference_jobs(&self) -> anyhow::Result<Vec<InferenceJob>> {
        match self {
            Self::Remote(b) => b.inference_jobs(),
            Self::Local(b) => b.inference_jobs(),
        }
    }

    pub fn inference_job(&self, die_id: Uuid) -> anyhow::Result<Option<InferenceJob>> {
        match self {
            Self::Remote(b) => b.inference_job(die_id),
            Self::Local(b) => b.inference_job(die_id),
        }
    }
    pub fn inference_job_start(&self, die_id: Uuid) -> anyhow::Result<Option<bool>> {
        match self {
            Self::Remote(b) => b.inference_job_start(die_id),
            Self::Local(b) => b.inference_job_start(die_id),
        }
    }
    pub fn inference_job_stop(&self, die_id: Uuid) -> anyhow::Result<Option<bool>> {
        match self {
            Self::Remote(b) => b.inference_job_stop(die_id),
            Self::Local(b) => b.inference_job_stop(die_id),
        }
    }
    pub fn vias(&self, die_id: Uuid, loc: TileLocation) -> anyhow::Result<()> {
        match self {
            Self::Remote(b) => b.vias(die_id, loc),
            Self::Local(b) => b.vias(die_id, loc),
        }
    }

    // One request for a whole native-tile range — replaces N per-tile calls
    // when the overlay loads a zoomed-out view. Cached-only by nature: a tile
    // with no cached prediction is omitted from `tiles`.
    pub fn vias_aabb_one_level(
        &self,
        die_id: Uuid,
        z: u32,
        aabb: &AABB,
    ) -> anyhow::Result<Vec<()>> {
        match self {
            Self::Remote(b) => b.vias_aabb_one_level(die_id, z, aabb),
            Self::Local(b) => b.vias_aabb_one_level(die_id, z, aabb),
        }
    }

    pub fn vias_aabb(&self, die_id: Uuid, aabb: &AABB) -> anyhow::Result<Vec<()>> {
        match self {
            Self::Remote(b) => b.vias_aabb(die_id, aabb),
            Self::Local(b) => b.vias_aabb(die_id, aabb),
        }
    }

    pub fn heatmap_cached(&self, die_id: Uuid, loc: TileLocation) -> anyhow::Result<()> {
        match self {
            Self::Remote(b) => b.heatmap_cached(die_id, loc),
            Self::Local(b) => b.heatmap_cached(die_id, loc),
        }
    }
    pub fn heatmap_uncached(&self, die_id: Uuid, loc: TileLocation) -> anyhow::Result<()> {
        match self {
            Self::Remote(b) => b.heatmap_uncached(die_id, loc),
            Self::Local(b) => b.heatmap_uncached(die_id, loc),
        }
    }
    pub fn train(&self) -> anyhow::Result<()> {
        match self {
            Self::Remote(b) => b.train(),
            Self::Local(b) => b.train(),
        }
    }
}
