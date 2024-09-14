use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use openapi::apis::public_voxel_model::{ApiVoxelModelsBulkGetResponse, ApiVoxelModelsGetResponse, PublicVoxelModel};
use openapi::models::{ApiVoxelModelsBulkGetQueryParams, ApiVoxelModelsGetQueryParams};
use crate::apis::ServerImpl;

impl PublicVoxelModel for ServerImpl{
    async fn api_voxel_models_bulk_get(&self, method: Method, host: Host, cookies: CookieJar, query_params: ApiVoxelModelsBulkGetQueryParams) -> Result<ApiVoxelModelsBulkGetResponse, String> {
        todo!()
    }

    async fn api_voxel_models_get(&self, method: Method, host: Host, cookies: CookieJar, query_params: ApiVoxelModelsGetQueryParams) -> Result<ApiVoxelModelsGetResponse, String> {
        todo!()
    }
}