use core::str;
use std::ops::Deref;

use anyhow::Context;
use axum::{Router, body::Bytes, extract::Multipart, handler::Handler};
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub trait RouterExt {
    type S: Clone + Send + Sync + 'static;
    fn die_param<T1: 'static, T2: 'static>(
        self,
        param: &str,
        list_fn: impl Handler<T1, Self::S>,
        delete_fn: impl Handler<T2, Self::S>,
    ) -> Self;
}

impl<S> RouterExt for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    type S = S;
    fn die_param<T1: 'static, T2: 'static>(
        self,
        param: &str,
        list_fn: impl Handler<T1, S>,
        delete_fn: impl Handler<T2, S>,
    ) -> Self {
        self.route(
            &format!("/api/dies/{{die_id}}/{param}/{{id}}"),
            axum::routing::get(list_fn).delete(delete_fn),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AABB {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
impl AABB {
    fn as_array(&self) -> [u32; 4] {
        [self.x, self.y, self.w, self.h]
    }
}

pub async fn get_single_file(mut multipart: Multipart) -> anyhow::Result<Option<(String, Bytes)>> {
    let mut file = None;
    while let Some(part) = multipart
        .next_field()
        .await
        .context("failed to read multipart")?
    {
        let name = part.name().context("can't read part name")?;

        match (name, file) {
            ("file", None) => {
                file = Some((
                    part.file_name().context("no filename")?.to_string(),
                    part.bytes().await.context("failed to read part bytes")?,
                ));
            }
            ("file", Some(_)) => {
                anyhow::bail!("only one file")
            }
            _ => {
                anyhow::bail!("expected `file` got {name}")
            }
        }
    }
    Ok(file)
}

pub trait Log {
    fn log(self) -> Self;
    fn log_error(self) -> Self;
}

impl<T> Log for anyhow::Result<T> {
    fn log(self) -> Self {
        if let Err(ref e) = self {
            log::info!("error: {e:?}");
        }
        self
    }
    fn log_error(self) -> Self {
        if let Err(ref e) = self {
            log::error!("error: {e:?}");
        }
        self
    }
}
