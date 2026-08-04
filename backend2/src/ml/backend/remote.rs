use reqwest::{Client, ClientBuilder};
use uuid::Uuid;

use crate::{
    Config,
    ml::domain::{InferenceJob, Model, ResetModel, Status},
    tiles::TileLocation,
    util::AABB,
};

pub struct RemoteBackend {
    client: Client,
    base: String,
    predict_pad: u32,
}
impl RemoteBackend {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            client: ClientBuilder::new().build()?,
            base: config.ml_sidecar_url.clone(),
            predict_pad: config.ml_predict_pad,
        })
    }

    pub fn status(&self) -> anyhow::Result<Status> {
        Ok(Status::Offline)
    }
    pub fn models(&self) -> anyhow::Result<Vec<Model>> {
        Ok(vec![])
    }

    /// Switches the sidecar's resident checkpoint. Every cached prediction is
    /// tied to the old checkpoint, so we wipe the prediction cache + ML jobs.
    pub fn reset_model(&self, name: &str) -> anyhow::Result<ResetModel> {
        todo!()
    }

    /// All per-die inference jobs — drives the library page's status badges.
    pub fn inference_jobs(&self) -> anyhow::Result<Vec<InferenceJob>> {
        todo!()
    }

    pub fn inference_job(&self, die_id: Uuid) -> anyhow::Result<Option<InferenceJob>> {
        todo!()
    }
    pub fn inference_job_start(&self, die_id: Uuid) -> anyhow::Result<Option<bool>> {
        todo!()
    }
    pub fn inference_job_stop(&self, die_id: Uuid) -> anyhow::Result<Option<bool>> {
        todo!()
    }
    pub fn vias(&self, die_id: Uuid, loc: TileLocation) -> anyhow::Result<()> {
        todo!()
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
        todo!()
    }

    pub fn vias_aabb(&self, die_id: Uuid, aabb: &AABB) -> anyhow::Result<Vec<()>> {
        todo!()
    }

    pub fn heatmap_cached(&self, die_id: Uuid, loc: TileLocation) -> anyhow::Result<()> {
        todo!()
    }
    pub fn heatmap_uncached(&self, die_id: Uuid, loc: TileLocation) -> anyhow::Result<()> {
        todo!()
    }
    pub fn train(&self) -> anyhow::Result<()> {
        todo!()
    }
}
