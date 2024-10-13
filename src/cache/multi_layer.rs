use crate::cache::disk::DiskCache;
use crate::cache::items::CacheDataType;
use crate::cache::items::{CacheKey, CachedData};
use moka::future::Cache;
use rustc_hash::FxBuildHasher;
use std::path::PathBuf;

// 2層キャッシュの構造体
#[derive(Clone)]
pub struct MultiLayerCache {
    memory: Cache<CacheKey, CachedData, FxBuildHasher>,
    disk: DiskCache,
}

impl MultiLayerCache {
    // 新しい2層キャッシュを初期化
    pub async fn new(memory_capacity: u64, disk_capacity: u64, disk_path: PathBuf) -> Self {
        let memory = Cache::<CacheKey, CachedData>::builder()
            .max_capacity(memory_capacity)
            .support_invalidation_closures()
            .build_with_hasher(FxBuildHasher::default());

        Self {
            memory,
            disk: DiskCache::new(disk_path.clone(), disk_capacity).await,
        }
    }

    // キャッシュにデータを登録（必要ならば計算）
    pub async fn get_or_compute<F>(&self, key: CacheKey, compute_fn: F) -> CachedData
    where
        F: FnOnce() -> CachedData,
    {
        self.memory
            .entry_by_ref(&key)
            .or_insert_with(async { self.get_or_compute(key, compute_fn).await })
            .await
            .into_value()
    }

    // 特定の種類のエントリをキャッシュから削除
    pub async fn evict(&self, data_type: CacheDataType) {
        let predicate_memory = move |key: &CacheKey, _: &CachedData| -> bool {
            key.data_type == data_type
        };

        let predicate_disk = move |key: &CacheKey, _: &PathBuf| -> bool {
            key.data_type == data_type
        };

        self.memory.invalidate_entries_if(predicate_memory).unwrap();
        self.disk.invalidate_entries_if(predicate_disk).await.unwrap();
    }
}
