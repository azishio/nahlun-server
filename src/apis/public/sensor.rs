use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use openapi::apis::public_sensor::{ApiSensorsGetResponse, PublicSensor};
use openapi::models::ApiSensorsGetQueryParams;
use crate::apis::ServerImpl;

impl PublicSensor for ServerImpl{
    async fn api_sensors_get(&self, method: Method, host: Host, cookies: CookieJar, query_params: ApiSensorsGetQueryParams) -> Result<ApiSensorsGetResponse, String> {
        todo!()
    }
}