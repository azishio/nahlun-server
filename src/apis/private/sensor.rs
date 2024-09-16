use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use openapi::apis::private_sensor::{
    ApiSensorsBulkDeleteResponse, ApiSensorsDeleteResponse, ApiSensorsPutResponse,
    ApiSersorsBulkPutResponse, PrivateSensor,
};
use openapi::models::{ApiSensorsDeleteQueryParams, ApiSensorsPutQueryParams, SensorRegistration};

use crate::apis::ServerImpl;

#[async_trait]
impl PrivateSensor for ServerImpl {
    async fn api_sensors_bulk_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ApiSensorsBulkDeleteResponse, String> {
        todo!()
    }

    async fn api_sensors_delete(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        query_params: ApiSensorsDeleteQueryParams,
    ) -> Result<ApiSensorsDeleteResponse, String> {
        todo!()
    }

    async fn api_sensors_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
        query_params: ApiSensorsPutQueryParams,
        body: Option<SensorRegistration>,
    ) -> Result<ApiSensorsPutResponse, String> {
        todo!()
    }

    async fn api_sersors_bulk_put(
        &self,
        method: Method,
        host: Host,
        cookies: CookieJar,
    ) -> Result<ApiSersorsBulkPutResponse, String> {
        todo!()
    }
}
