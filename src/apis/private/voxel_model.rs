use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use openapi::apis::private_voxel_model::{
    ApiVoxelModelsDeleteResponse, ApiVoxelModelsPutResponse, PrivateVoxelModel,
};
use openapi::models::{ApiVoxelModelsDeleteQueryParams, ApiVoxelModelsPutQueryParams};

use crate::apis::ServerImpl;

impl PrivateVoxelModel for ServerImpl {
    async fn api_voxel_models_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        query_params: ApiVoxelModelsDeleteQueryParams,
    ) -> Result<ApiVoxelModelsDeleteResponse, String> {
        todo!()
    }

    async fn api_voxel_models_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        query_params: ApiVoxelModelsPutQueryParams,
    ) -> Result<ApiVoxelModelsPutResponse, String> {
        todo!()
    }
}
