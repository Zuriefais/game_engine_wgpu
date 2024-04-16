pub mod cell_assets;

use ecolor::Rgba;
use glam::{IVec2, Vec3};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum CellPhysicsType {
    Sand,
    Fluid,
    Tap(String),
    Solid,
}

pub const CELL_SIZE: Vec3 = Vec3::new(10.0, 10.0, 10.0);

pub const CHUNK_SIZE: IVec2 = IVec2::new(100, 100);

pub const CHUNK_SIZE_LEN: usize = (CHUNK_SIZE.x * CHUNK_SIZE.y) as usize;
