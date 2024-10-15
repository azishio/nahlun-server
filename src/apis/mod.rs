//! パスごとに呼び出される処理を定義

use crate::cache::multi_layer::MultiLayerCache;
use crate::env::EnvVars;
use neo4rs::{ConfigBuilder, Graph};
use rust_socketio::asynchronous::{Client, ClientBuilder};

mod tile;
mod sensor;
mod sensor_data;

/// パスごとの処理内容をimplするための構造体
#[derive(Clone)]
pub struct ServerImpl {
    graph: Graph,
    cache: MultiLayerCache,
    http_client: reqwest::Client,
    socketio_client: Client,
}
impl ServerImpl {
    /// 新しいServerImplを作成する
    pub async fn new() -> Self {
        let env = EnvVars::read_env().unwrap();

        let graph = {
            let mut auth = env.neo4j_auth.split('/');
            let neo4j_user = auth.next().unwrap();
            let neo4j_password = auth.next().unwrap();

            let config = ConfigBuilder::default()
                .uri(env.neo4j_uri)
                .user(neo4j_user)
                .password(neo4j_password)
                .db(env.neo4j_db)
                .build()
                .unwrap();
            Graph::connect(config).await.unwrap()
        };

        let cache = MultiLayerCache::new(
            env.memory_cache_max_size,
            env.disk_cache_max_size,
            env.disk_cache_base_path.into(),
        ).await;


        let http_client = reqwest::Client::new();
        let socketio_client = ClientBuilder::new(env.socketio_host)
            .namespace("/water_surface")
            .connect()
            .await
            .expect("Failed to connect to socket.io server");

        Self {
            graph,
            cache,
            http_client,
            socketio_client,
        }
    }
}

impl AsRef<ServerImpl> for ServerImpl {
    fn as_ref(&self) -> &ServerImpl {
        self
    }
}
