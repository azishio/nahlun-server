//! パスごとに呼び出される処理を定義

use neo4rs::Graph;
use reqwest::Client;

use crate::cache::MultiLayerCache;

mod private;
mod public;
mod tile;

/// パスごとの処理内容をimplするための構造体
#[derive(Clone)]
pub struct ServerImpl {
    graph: Graph,
    cache: MultiLayerCache,
    http_client: Client,
}

impl ServerImpl {
    /// 新しいServerImplを作成する
    pub fn with_graph_and_cache(graph: Graph, cache: MultiLayerCache) -> Self {
        let http_client = Client::new();
        Self {
            graph,
            cache,
            http_client,
        }
    }
}

impl AsRef<ServerImpl> for ServerImpl {
    fn as_ref(&self) -> &ServerImpl {
        self
    }
}
