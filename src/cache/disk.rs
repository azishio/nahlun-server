use crate::cache::items::{CachedData, CachedDataDiscriminants};
use crate::cache::CacheKey;
use moka::future::{Cache, FutureExt, PredicateId};
use moka::PredicateError;
use rustc_hash::FxBuildHasher;
use std::future::Future;
use std::hash::BuildHasher;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone)]
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
    pub(crate) async fn insert(&self, key: &CacheKey, data: &CachedData) -> std::io::Result<()> {
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
        compute_fn: impl Future<Output=CachedData>,
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

    pub(crate) async fn invalidate_entries_if<F>(&self, predicate: F) -> Result<PredicateId, PredicateError>
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
