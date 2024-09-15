use std::future::Future;
use std::hash::BuildHasher;
use std::path::PathBuf;

use moka::future::{Cache, FutureExt, PredicateId};
use moka::PredicateError;
use rustc_hash::FxBuildHasher;
use strum::EnumDiscriminants;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

// キャッシュキーの定義
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum CacheKey {
    Uuid(Uuid),
}

// キャッシュデータの定義
#[derive(Clone, EnumDiscriminants)]
pub enum CachedData {
    // (仮)
    StringData(String),
}

impl CachedData {
    // キャッシュデータをバイト列に変換
    fn as_bytes(&self) -> (CachedDataDiscriminants, &[u8]) {
        match self {
            CachedData::StringData(s) => (CachedDataDiscriminants::StringData, s.as_bytes()),
        }
    }

    fn from_bytes(dtype: CachedDataDiscriminants, bytes: &[u8]) -> Self {
        match dtype {
            CachedDataDiscriminants::StringData => {
                CachedData::StringData(String::from_utf8(bytes.to_vec()).unwrap())
            }
        }
    }
}

// 2層キャッシュの構造体
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

pub struct DiskCache {
    disk_path: PathBuf,
    metadata: Cache<CacheKey, (CachedDataDiscriminants, PathBuf), FxBuildHasher>,
}

impl DiskCache {
    pub(crate) fn new(disk_path: PathBuf, max_capacity: u64) -> Self {
        Self {
            disk_path,
            metadata: Cache::<CacheKey, (CachedDataDiscriminants, PathBuf)>::builder()
                .max_capacity(max_capacity)
                .support_invalidation_closures()
                .async_eviction_listener(|_, (_, path), _| {
                    async move {
                        if path.exists() {
                            let _ = tokio::fs::remove_file(path).await;
                        }
                    }
                    .boxed()
                })
                .build_with_hasher(FxBuildHasher::default()),
        }
    }
    async fn insert(&self, key: &CacheKey, data: &CachedData) -> std::io::Result<()> {
        let file_path = self.get_file_path(key);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .await?;

        let (dtype, bytes) = data.as_bytes();

        file.write_all(bytes).await?;
        file.flush().await?;

        self.metadata.insert(key.clone(), (dtype, file_path)).await;

        Ok(())
    }

    async fn get_or_insert_with(
        &self,
        key: &CacheKey,
        compute_fn: impl Future<Output = CachedData>,
    ) -> anyhow::Result<CachedData> {
        if let Some((dtype, file_path)) = self.metadata.get(key).await {
            let file = OpenOptions::new().read(true).open(file_path).await?;
            let mut reader = tokio::io::BufReader::new(file);

            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).await?;

            let data = CachedData::from_bytes(dtype.clone(), &buf);

            return Ok(data);
        }

        let data = compute_fn.await;
        let (dtype, bytes) = data.as_bytes();

        let file_path = self.get_file_path(key);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .await?;

        file.write_all(bytes).await?;
        file.flush().await?;

        self.metadata.insert(key.clone(), (dtype, file_path)).await;

        Ok(data)
    }

    async fn invalidate_entries_if<F>(&self, predicate: F) -> Result<PredicateId, PredicateError>
    where
        F: Fn(&CacheKey, &(CachedDataDiscriminants, PathBuf)) -> bool
            + Send
            + Sync
            + 'static
            + Clone,
    {
        self.metadata.invalidate_entries_if(predicate)
    }

    fn get_file_path(&self, key: &CacheKey) -> PathBuf {
        self.disk_path
            .join(FxBuildHasher::default().hash_one(&key).to_string())
    }
}
