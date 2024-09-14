use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use openapi::apis::public_river::{ApiRiverNodesBulkGetResponse, ApiRiverNodesGetResponse, PublicRiver};
use openapi::models::{ApiRiverNodesBulkGetQueryParams, ApiRiverNodesGetQueryParams};
use crate::apis::ServerImpl;

impl PublicRiver for ServerImpl {
    async fn api_river_nodes_bulk_get(&self, method: Method, host: Host, cookies: CookieJar, query_params: ApiRiverNodesBulkGetQueryParams) -> Result<ApiRiverNodesBulkGetResponse, String> {
        todo!()
    }

    async fn api_river_nodes_get(&self, method: Method, host: Host, cookies: CookieJar, query_params: ApiRiverNodesGetQueryParams) -> Result<ApiRiverNodesGetResponse, String> {
        todo!()
    }
} 