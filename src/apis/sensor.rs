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
        let body = body.unwrap();
        let ApiSensorsPostRequest { altitude, interval, scope, parent_node } = body;
        let ApiSensorsPostQueryParams { id } = query_params;

        let mut tnx = self.graph.start_txn().await.unwrap();
        let _ = tnx
            .run_queries([
                query(r#"
MERGE (sensor:Sensor{id:$id})
ON CREATE
 SET sensor.altitude=$altitude,
     sensor.scope=$scope,
     sensor.interval=$interval,
     sensor.parent_node=$parent_node
ON MATCH
 SET sensor.altitude=$altitude,
     sensor.scope=$scope,
     sensor.interval=$interval,
     sensor.parent_node=$parent_node

WITH sensor
MATCH (sensor)-[r:BELONGS_TO|AFFECTS]-(parent:RiverNode)
DELETE r
                "#),
                query(r#"
MATCH (parent:RiverNode{hilbert18:$parent_node})
MATCH (sensor:Sensor{id:$id})
CREATE (sensor)-[:BELONGS_TO]->(parent)

WITH sensor, parent
CALL apoc.path.expandConfig(parent, {
    relationshipFilter: 'RIVER_LINK',
    minLevel: 1,
    maxLevel: 1000,
    bfs: true,
    limit: 1000
}) YIELD path

WITH sensor, parent, last(nodes(path)) AS target,
     reduce(totalDistance = 0, rel in relationships(path) | totalDistance + rel.length) AS distance

WHERE distance <= $scope
CREATE (sensor)-[:AFFECTS {distance: distance}]->(target)
                "#)
            ]).await;

        tnx.commit().await.unwrap();

        Ok(ApiSensorsPostResponse::Status200)
    }
}
