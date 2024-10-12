mod disk;
pub(crate) mod multi_layer;
mod items;

use uuid::Uuid;

const CACHE_DIR: &str = "/var/nahlund/server/cache";

// キャッシュキーの定義
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum CacheKey {
    Uuid(Uuid),
}

