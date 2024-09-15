use axum::serve;
use neo4rs::{ConfigBuilder, Graph};
use openapi::server::new;
use tokio::net::TcpListener;

use crate::apis::ServerImpl;
use crate::cache::MultiLayerCache;
use crate::env::EnvVars;

mod apis;
mod cache;
mod db;
mod env;

#[derive(Clone)]
struct ServerState {}

#[tokio::main]
async fn main() {
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
    );

    let router = new(ServerImpl::with_graph_and_cache(graph, cache));
    let listener = TcpListener::bind("localhost:3000").await.unwrap();

    serve(listener, router).await.unwrap();
}
