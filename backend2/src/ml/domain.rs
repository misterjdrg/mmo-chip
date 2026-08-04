use reqwest::{Client, ClientBuilder};
use uuid::Uuid;

use crate::{Config, tiles::TileLocation, util::AABB};

pub enum Status {
    Offline,
    Online {
        status: String,
        device: String,
        checkpoint: Option<String>,
        checkpoint_hash: Option<String>,
        encoder: String,
        model_loaded: bool,
        training_active: bool,
    },
}

pub struct Model {
    name: String,
    hash: Option<String>,
    size_bytes: u64,
    resident: bool,
}

pub struct ResetModel {
    checkpoint: Option<String>,
    checkpoint_hash: Option<String>,
    model_loaded: bool,
}
pub struct InferenceJob;
