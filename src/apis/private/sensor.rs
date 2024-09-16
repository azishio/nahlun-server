use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use openapi::apis::private_sensor::{
    ApiSensorsDeleteResponse, ApiSersorsPutResponse, PrivateSensor,
};
use openapi::models::Sensor;
use uuid::Uuid;

use crate::apis::ServerImpl;

#[async_trait]
impl PrivateSensor for ServerImpl {
    async fn api_sensors_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: Option<Vec<Uuid>>,
    ) -> Result<ApiSensorsDeleteResponse, String> {
        todo!()
    }

    async fn api_sersors_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        body: Option<Vec<Sensor>>,
    ) -> Result<ApiSersorsPutResponse, String> {
        todo!()
    }
}
