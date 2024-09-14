use axum::serve;
use openapi::server::new;
use tokio::net::TcpListener;

use crate::apis::ServerImpl;

mod apis;

#[tokio::main]
async fn main() {
    let router = new(ServerImpl {});
    let listener = TcpListener::bind("localhost:3000").await.unwrap();

    serve(listener, router).await.unwrap();
}
