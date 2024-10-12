use crate::cache::disk::DiskCache;
use crate::cache::items::{CachedData, CachedDataDiscriminants};
use crate::cache::CacheKey;
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
    pub fn new(memory_capacity: u64, disk_capacity: u64, disk_path: PathBuf) -> Self {
        let memory = Cache::<CacheKey, CachedData>::builder()
            .max_capacity(memory_capacity)
            .support_invalidation_closures()
            .build_with_hasher(FxBuildHasher::default());

        Self {
            memory,
            disk: DiskCache::new(disk_path.clone(), disk_capacity),
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

    // 特定の条件でキャッシュのエントリを無効化
    pub async fn invalidate_entries_if<F>(&self, predicate: F) -> anyhow::Result<()>
    where
        F: Fn(&CacheKey) -> bool + Send + Sync + 'static + Clone,
    {
        // メモリキャッシュのエントリを無効化する条件
        let predicate_for_memory = {
            let predicate = predicate.clone();
            move |key: &CacheKey, _: &CachedData| -> bool { predicate(key) }
        };
        // ディスクキャッシュのエントリを無効化する条件
        let predicate_for_disk = {
            move |key: &CacheKey, _: &(CachedDataDiscriminants, PathBuf)| -> bool { predicate(key) }
        };

        // メモリキャッシュのエントリを条件に基づいて無効化
        self.memory.invalidate_entries_if(predicate_for_memory)?;
        // ディスクキャッシュも同様に無効化する
        self.disk.invalidate_entries_if(predicate_for_disk).await?;

        Ok(())
    }
}
