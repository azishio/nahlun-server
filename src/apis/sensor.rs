use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use neo4rs::query;
use openapi::apis::sensor::{ApiSensorsDeleteResponse, ApiSensorsPutResponse, Sensor};
use openapi::models::{
    ApiSensorsDeleteQueryParams, ApiSensorsPutQueryParams, ApiSensorsPutRequest,
};

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

    async fn api_sensors_put(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        query_params: ApiSensorsPutQueryParams,
        body: Option<ApiSensorsPutRequest>,
    ) -> Result<ApiSensorsPutResponse, String> {
        let body = body.unwrap();
        let ApiSensorsPutRequest {
            altitude,
            scope,
            interval,
            parent_node
        } = body;

        let query = query(
            r#"
            MERGE (sensor:Sensor{id:$id})
            ON CREATE
                SET altitude = $altitude,
                    scope = $scope,
                    interval = $interval
            ON MATCH
                SET altitude = $altitude,
                    scope = $scope,
                    interval = $interval
            RETURN sensor
            "#,
        )
            .param("id", query_params.id.to_string())
            .param("altitude", altitude)
            .param("scope", scope)
            .param("interval", interval);

        let _ = self.graph.execute(query).await.unwrap();

        Ok(ApiSensorsPutResponse::Status200)
    }
}
