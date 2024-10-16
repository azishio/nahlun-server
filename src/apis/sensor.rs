use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use neo4rs::query;
use openapi::apis::sensor::{ApiSensorsDeleteResponse, ApiSensorsPostResponse, Sensor};
use openapi::models::{ApiSensorsDeleteQueryParams, ApiSensorsPostQueryParams, ApiSensorsPostRequest};

use crate::apis::ServerImpl;

#[async_trait]
impl Sensor for ServerImpl {
    async fn api_sensors_delete(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        query_params: ApiSensorsDeleteQueryParams,
    ) -> Result<ApiSensorsDeleteResponse, String> {
        let query = query(
            r#"
MATCH (sensor:Sensor{id:$id})
DETACH DELETE sensor
RETURN id
            "#,
        )
            .param("id", query_params.id.to_string());

        let _ = self.graph.execute(query).await.unwrap();

        Ok(ApiSensorsDeleteResponse::Status200)
    }

    async fn api_sensors_post(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        query_params: ApiSensorsPostQueryParams,
        body: Option<ApiSensorsPostRequest>,
    ) -> Result<ApiSensorsPostResponse, String> {
        todo!()
    }
}
