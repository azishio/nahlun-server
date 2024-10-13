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
    let router = new(ServerImpl::new()).layer(
        CorsLayer::new()
            .allow_origin(AllowOrigin::exact("http://localhost:3001".parse().unwrap()))
            .allow_methods(vec![Method::GET]),
    );
    let listener = TcpListener::bind("localhost:3000").await.unwrap();

    serve(listener, router).await.unwrap();
}
