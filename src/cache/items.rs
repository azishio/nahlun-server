use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct TileId {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

impl Display for TileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}_{}", self.x, self.y, self.z)
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum CacheDataType {
    LandTile,
    WaterTile,
    CustomVoxelModelTile,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct CacheKey {
    pub data_type: CacheDataType,
    pub tile_id: TileId,
}

impl Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.data_type, self.tile_id)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CachedData {
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
    pub registered_at: u64,
}
