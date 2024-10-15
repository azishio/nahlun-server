use axum::http::Method;
use axum::serve;
use openapi::server::new;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::apis::ServerImpl;

mod apis;
mod cache;
mod db;
mod env;
#[derive(Clone)]
struct ServerState {}

#[tokio::main]
async fn main() {
    let env = env::EnvVars::read_env().unwrap();

    let router = new(ServerImpl::new()).layer(
        CorsLayer::new()
            .allow_origin(AllowOrigin::exact(env.client_host.parse().unwrap()))
            .allow_methods(vec![Method::GET]),
    );
    let listener = TcpListener::bind(env.server_host).await.unwrap();

    serve(listener, router).await.unwrap();
}
