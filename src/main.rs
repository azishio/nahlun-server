use axum::http::Method;
use axum::routing::get;
use axum::serve;
use openapi::server::new;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::apis::ServerImpl;
use crate::env::EnvVars;

mod apis;
mod cache;
mod env;
#[derive(Clone)]
struct ServerState {}

#[tokio::main]
async fn main() {
    let env = EnvVars::read_env().unwrap();
    let EnvVars { server_host, client_host, .. } = env;

    let listener = TcpListener::bind(server_host.clone()).await.unwrap();

    let router = new(ServerImpl::new().await)
        .route("/", get(|| async move {
            const VERSION: &'static str = env!("CARGO_PKG_VERSION");
            format!("Hello, Nahlun! by {server_host}\nI'm Server Container v{VERSION}.\n")
        }))
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::exact(client_host.parse().unwrap()))
                .allow_methods(vec![Method::GET]),
        );

    serve(listener, router).await.unwrap();
}
