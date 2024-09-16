use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use openapi::apis::private_sensor::{
    ApiSensorsDeleteResponse, ApiSensorsPutResponse, PrivateSensor,
};
use openapi::models::{
    ApiSensorsDeleteQueryParams, ApiSensorsPutQueryParams, ApiSensorsPutRequest,
};

use crate::apis::ServerImpl;

#[async_trait]
impl PrivateSensor for ServerImpl {
    async fn api_sensors_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        query_params: ApiSensorsDeleteQueryParams,
    ) -> Result<ApiSensorsDeleteResponse, String> {
        todo!()
    }

    async fn api_sensors_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        query_params: ApiSensorsPutQueryParams,
        body: Option<ApiSensorsPutRequest>,
    ) -> Result<ApiSensorsPutResponse, String> {
        todo!()
    }
}
