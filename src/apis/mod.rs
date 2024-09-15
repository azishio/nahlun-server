//! パスごとに呼び出される処理を定義

use neo4rs::Graph;

use crate::cache::MultiLayerCache;

mod private;
mod public;
mod tile;

/// パスごとの処理内容をimplするための構造体
#[derive(Clone)]
pub struct ServerImpl {
    graph: Graph,
    cache: MultiLayerCache,
}

impl ServerImpl {
    /// 新しいServerImplを作成する
    pub fn with_graph_and_cache(graph: Graph, cache: MultiLayerCache) -> Self {
        Self { graph, cache }
    }
}

impl AsRef<ServerImpl> for ServerImpl {
    fn as_ref(&self) -> &ServerImpl {
        self
    }
}
