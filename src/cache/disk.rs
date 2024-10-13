use crate::cache::items::{CacheDataType, CacheKey};
use crate::cache::items::{CachedData, TileId};
use moka::future::{Cache, FutureExt, PredicateId};
use moka::PredicateError;
use rustc_hash::FxBuildHasher;
use std::future::Future;
use std::path::PathBuf;
use strum::IntoEnumIterator;
use tokio::fs::{read_dir, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone)]
pub struct DiskCache {
    disk_path: PathBuf,
    metadata: Cache<CacheKey, PathBuf, FxBuildHasher>,
}

impl DiskCache {
    pub(crate) async fn new(disk_path: PathBuf, max_capacity: u64) -> Self {
        let metadata = Cache::<CacheKey, PathBuf>::builder()
            .max_capacity(max_capacity)
            .support_invalidation_closures()
            .async_eviction_listener(|_, path, _| {
                async move {
                    if path.exists() {
                        let _ = tokio::fs::remove_file(path).await;
                    }
                }
                    .boxed()
            })
            .build_with_hasher(FxBuildHasher::default());

        // ディスクからの読み込み処理
        for data_type in CacheDataType::iter() {
            let path = disk_path.join(data_type.to_string());
            if !path.exists() || !path.is_dir() {
                continue;
            }

            if let Ok(mut entries) = read_dir(&path).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let tile_id = match entry.file_name().into_string() {
                        Ok(file_name) => match file_name.parse::<TileId>() {
                            Ok(tile_id) => tile_id,
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    };

                    let key = CacheKey {
                        data_type,
                        tile_id,
                    };

                    metadata.insert(key, entry.path()).await;
                }
            }
        }

        Self {
            disk_path,
            metadata,
        }
    }
    pub(crate) async fn insert(&self, key: &CacheKey, data: &CachedData) -> std::io::Result<()> {
        let file_path = self.disk_path.join(key.to_string());

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .await?;

        let bytes = bincode::serialize(data).unwrap();

        file.write_all(&bytes).await?;
        file.flush().await?;

        self.metadata.insert(key.clone(), file_path).await;

        Ok(())
    }

    async fn get_or_insert_with(
        &self,
        key: &CacheKey,
        compute_fn: impl Future<Output=CachedData>,
    ) -> anyhow::Result<CachedData> {
        if let Some(file_path) = self.metadata.get(key).await {
            let file = OpenOptions::new().read(true).open(file_path).await?;
            let mut reader = tokio::io::BufReader::new(file);

            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).await?;

            let data = bincode::deserialize(&buf)?;

            return Ok(data);
        }

        let data = compute_fn.await;

        let file_path = self.disk_path.join(key.to_string());

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .await?;

        let bytes = bincode::serialize(&data)?;

        file.write_all(&bytes).await?;
        file.flush().await?;

        self.metadata.insert(*key, file_path).await;

        Ok(data)
    }

    pub(crate) async fn invalidate_entries_if<F>(&self, predicate: F) -> Result<PredicateId, PredicateError>
    where
        F: Fn(&CacheKey, &PathBuf) -> bool
        + Send
        + Sync
        + 'static
        + Clone,
    {
        self.metadata.invalidate_entries_if(predicate)
    }
}
