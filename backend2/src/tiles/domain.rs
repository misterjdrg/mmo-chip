use std::iter;

use serde::{Deserialize, Serialize};

use crate::util::AABB;

#[derive(Debug)]
pub struct TileClip {
    pub src: AABB,
    pub dst_w: u32,
    pub dst_h: u32,
}
#[derive(Debug)]
pub struct TileLocation {
    pub z: u32,
    pub x: u32,
    pub y: u32,
}
pub struct DieInfo {
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
    pub max_zoom_level: u32,
}

impl DieInfo {
    pub fn new(width: u32, height: u32, tile_size: u32) -> Self {
        let max_dimension = width.max(height);
        let max_zoom_level = (max_dimension as f64 / tile_size as f64)
            .log2()
            .ceil()
            .max(0.0) as u32;

        Self {
            width,
            height,
            tile_size,
            max_zoom_level,
        }
    }
    fn get_tile_clip(&self, level: &LevelInfo, loc: &TileLocation) -> TileClip {
        let src_x = loc.x * self.tile_size;
        let src_y = loc.y * self.tile_size;
        let dst_w = self.tile_size.min(level.width - src_x);
        eprintln!("h: {} y: {src_y}", level.height);
        let dst_h = self.tile_size.min(level.height - src_y);

        let src_x = src_x * level.scale;
        let src_y = src_y * level.scale;
        let src_w = (self.width - src_x).min(dst_w * level.scale);
        let src_h = (self.height - src_y).min(dst_h * level.scale);

        TileClip {
            src: AABB {
                x: src_x,
                y: src_y,
                w: src_w,
                h: src_h,
            },
            dst_w,
            dst_h,
        }
    }
    pub fn build_levels(&self) -> Vec<LevelInfo> {
        let mut infos = vec![];
        for z in 0..(self.max_zoom_level + 1) {
            let scale = 1 << (self.max_zoom_level - z);
            let width = 1.max(self.width / scale);
            let height = 1.max(self.height / scale);
            let columns = 1.max(width / self.tile_size);
            let rows = 1.max(height / self.tile_size);

            infos.push(LevelInfo {
                z,
                width,
                height,
                columns,
                rows,
                scale,
            });
        }
        infos
    }

    pub fn tiles(&self) -> impl Iterator<Item = (TileLocation, TileClip)> {
        let mut z = 0;
        let mut x = 0;
        let mut y = 0;
        let levels = self.build_levels();

        iter::from_fn(move || {
            loop {
                let level = levels.get(z as usize)?;
                if x < level.columns {
                    let loc = TileLocation { z, x, y };
                    let clip = self.get_tile_clip(&level, &loc);
                    x += 1;
                    return Some((loc, clip));
                }
                if y < level.rows {
                    x = 0;
                    y += 1;
                    continue;
                }

                x = 0;
                y = 0;
                z += 1;
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelInfo {
    pub z: u32,
    pub width: u32,
    pub height: u32,
    pub columns: u32,
    pub rows: u32,
    pub scale: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// builds a complete zoom pyramid
    #[test]
    fn pyramid() {
        assert_eq!(
            DieInfo {
                width: 4096,
                height: 2048,
                tile_size: 512,
                max_zoom_level: 3
            }
            .build_levels(),
            vec![
                LevelInfo {
                    z: 0,
                    width: 512,
                    height: 256,
                    columns: 1,
                    rows: 1,
                    scale: 8
                },
                LevelInfo {
                    z: 1,
                    width: 1024,
                    height: 512,
                    columns: 2,
                    rows: 1,
                    scale: 4
                },
                LevelInfo {
                    z: 2,
                    width: 2048,
                    height: 1024,
                    columns: 4,
                    rows: 2,
                    scale: 2
                },
                LevelInfo {
                    z: 3,
                    width: 4096,
                    height: 2048,
                    columns: 8,
                    rows: 4,
                    scale: 1
                },
            ]
        );
    }

    #[test]
    fn tile_count() {
        let die = DieInfo::new(4096, 2048, 512);
        let count: u32 = die.build_levels().iter().map(|l| l.rows * l.columns).sum();

        assert_eq!(
            count as usize,
            die.tiles()
                .inspect(|d| {
                    dbg!(d);
                })
                .count()
        );
    }
}
