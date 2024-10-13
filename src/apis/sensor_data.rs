use crate::apis::ServerImpl;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use openapi::apis::sensor_data::{PostApiSensorsDataResponse, SensorData};
use openapi::models::{PostApiSensorsDataQueryParams, PostApiSensorsDataRequest};

impl SensorData for ServerImpl {
    async fn post_api_sensors_data(
        &self, _method: Method,
        _host: Host,
        _cookies: CookieJar,
        query_params: PostApiSensorsDataQueryParams,
        body: Option<PostApiSensorsDataRequest>,
    ) -> Result<PostApiSensorsDataResponse, String> {
        todo!()
    }
}
