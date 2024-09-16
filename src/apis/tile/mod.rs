//! 幾つかのタイルデータを公開

use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use openapi::apis::tile::{Tile, TilesLandZxyGetResponse, TilesWaterZxyGetResponse};
use openapi::models::{TilesLandZxyGetPathParams, TilesWaterZxyGetPathParams};

use crate::apis::ServerImpl;

#[async_trait]
impl Tile for ServerImpl {
    async fn tiles_land_zxy_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: TilesLandZxyGetPathParams,
    ) -> Result<TilesLandZxyGetResponse, String> {
        todo!()
    }

    async fn tiles_water_zxy_get(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        path_params: TilesWaterZxyGetPathParams,
    ) -> Result<TilesWaterZxyGetResponse, String> {
        todo!()
    }
}
