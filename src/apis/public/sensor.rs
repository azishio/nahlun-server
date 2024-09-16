use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use neo4rs::query;
use openapi::apis::public_sensor::{ApiSensorsGetResponse, PublicSensor};
use openapi::models::{ApiSensorsGet200Response, ApiSensorsGetQueryParams};

use crate::apis::ServerImpl;
use crate::db;

#[async_trait]
impl PublicSensor for ServerImpl {
    async fn api_sensors_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        query_params: ApiSensorsGetQueryParams,
    ) -> Result<ApiSensorsGetResponse, String> {
        let ApiSensorsGetQueryParams { id, data_limit } = query_params;

        let query = query(
            r#"
                MATCH (sensor:Sensor{id: $id})
                MATCH (sensor)-[:CURRENT_DATA]->(current:SensorData)
                MATCH path = (current)-[:PREVIOUS_DATA*0..$data_limit]->(previous:SensorData)
                WITH sensor path
                ORDER BY length(path) DESC
                LIMIT 1
                RETURN sensor, nodes(path) as result
                "#,
        )
        .param("id", id.to_string())
        .param("data_limit", data_limit.unwrap_or(1));

        let mut result = self.graph.execute(query).await.unwrap();

        let sensor = if let Ok(Some(row)) = result.next().await {
            let sensor: db::Sensor = row.get("sensor").unwrap();
            sensor.into()
        } else {
            todo!()
        };

        let data = {
            let mut buf = Vec::new();
            while let Ok(Some(row)) = result.next().await {
                let node: db::SensorData = row.get("result").unwrap();
                buf.push(node.into());
            }
            buf
        };

        Ok(ApiSensorsGetResponse::Status200(ApiSensorsGet200Response {
            sensor,
            data,
        }))
    }
}
