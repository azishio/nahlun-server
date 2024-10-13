use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EnvVars {
    pub neo4j_uri: String,
    pub neo4j_auth: String,
    pub neo4j_db: String,
    pub socketio_host: String,
    pub disk_cache_base_path: String,
    pub disk_cache_max_size: u64,
    pub memory_cache_max_size: u64,
}

impl EnvVars {
    pub(crate) fn read_env() -> envy::Result<Self> {
        envy::from_env::<Self>()
    }
}
