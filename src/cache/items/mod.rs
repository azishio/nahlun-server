use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;


#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct TileId {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}
// キャッシュキーの定義
#[derive(Hash, Eq, PartialEq, Clone, Copy, EnumDiscriminants)]
pub enum CacheKey {
    LandTile(TileId),
    WaterTile(TileId),
    CustomVoxelModelTile(TileId),
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CachedData {
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
    pub registered_at: u64,
}
