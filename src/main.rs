use axum::http::Method;
use axum::serve;
use neo4rs::{ConfigBuilder, Graph};
use openapi::server::new;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::apis::ServerImpl;
use crate::env::EnvVars;
use cache::multi_layer::MultiLayerCache;

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
    ).await;

    let router = new(ServerImpl::with_graph_and_cache(graph, cache)).layer(
        CorsLayer::new()
            .allow_origin(AllowOrigin::exact("http://localhost:3001".parse().unwrap()))
            .allow_methods(vec![Method::GET]),
    );
    let listener = TcpListener::bind("localhost:3000").await.unwrap();

    serve(listener, router).await.unwrap();
}
