use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use neo4rs::query;
use openapi::apis::sensor::{ApiSensorsDeleteResponse, ApiSensorsPostResponse, ApiSensorsPutResponse, Sensor};
use openapi::models::{ApiSensorsDeleteQueryParams, ApiSensorsPostQueryParams, ApiSensorsPutQueryParams, ApiSensorsPutRequest};

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

    async fn api_sensors_post(&self, method: Method, host: Host, cookies: CookieJar, query_params: ApiSensorsPostQueryParams, body: Option<ApiSensorsPutRequest>) -> Result<ApiSensorsPostResponse, String>
    {
        let body = body.unwrap();
        let ApiSensorsPutRequest {
            altitude,
            scope,
            interval,
            parent_node
        } = body;

        let query = query(
            r#"
// 1. 既存のSensorノードとBELONGS_TO関係の取得
OPTIONAL MATCH (sensor:Sensor {id: $id})
OPTIONAL MATCH (sensor)-[:BELONGS_TO]->(currentParent:RiverNode)
WITH sensor, currentParent, sensor.scope AS oldScope, currentParent.id AS oldParentId

// 2. SensorノードのUpsertとプロパティの設定
MERGE (sensor:Sensor {id: $id})
  ON CREATE SET
    sensor.altitude = $altitude,
    sensor.scope = $scope,
    sensor.interval = $interval
  ON MATCH SET
    sensor.altitude = $altitude,
    sensor.scope = $scope,
    sensor.interval = $interval

// 3. 新しいparent_nodeとscopeの設定
WITH sensor, oldScope, oldParentId, $parent_node AS newParentId, $scope AS newScope

// 4. scopeやparent_nodeの変更を検出し、変更があった場合のみ処理を実行
CALL apoc.do.when(
  // 条件: parent_nodeが異なる、またはscopeが異なる場合
  (oldParentId IS NULL OR oldParentId <> newParentId) OR (oldScope IS NULL OR oldScope <> newScope),
  '
    // 4.1. 既存のBELONGS_TO関係とAFFECTS関係の削除
    MATCH (sensor)-[r:BELONGS_TO]->(:RiverNode)
    DELETE r
    MATCH (sensor)-[a:AFFECTS]->(:RiverNode)
    DELETE a

    // 4.2. 新しいBELONGS_TO関係の作成
    MATCH (newParent:RiverNode {hilbert18: newParentId})
    CREATE (sensor)-[:BELONGS_TO]->(newParent)

    // 4.3. 新しいAFFECTS関係の作成
    CALL apoc.path.expandConfig(newParent, {
      relationshipFilter: "RIVER_LINK",
      labelFilter: "+RiverNode",
      bfs: true,
      maxLevel: 1000
    }) YIELD path

    WITH sensor, path, $newScope AS maxScope,
         reduce(total = 0, r IN relationships(path) | total + r.length) AS distance
    WHERE distance <= maxScope

    UNWIND nodes(path) AS node
    // distanceプロパティをAFFECTS関係に追加
    MERGE (sensor)-[affects:AFFECTS]->(node)
      ON CREATE SET affects.distance = distance
      ON MATCH SET affects.distance = distance

    // BELONGS_TO関係のあるノードにはAFFECTS関係を削除
    MATCH (sensor)-[:BELONGS_TO]->(parent:RiverNode)
    MATCH (sensor)-[a:AFFECTS]->(parent)
    DELETE a

  ',
  '', // 条件が偽の場合は何もしない
  {sensor: sensor, newParentId: newParentId, newScope: newScope}
) YIELD value

// 5. クエリの終了
RETURN sensor
            "#,
        )
            .param("id", query_params.id.to_string())
            .param("altitude", altitude)
            .param("scope", scope)
            .param("interval", interval)
            .param("parent_node", parent_node);

        let _ = self.graph.execute(query).await.unwrap();

        Ok(ApiSensorsPostResponse::Status200)
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
// 1. 既存のSensorノードとBELONGS_TO関係の取得
OPTIONAL MATCH (sensor:Sensor {id: $id})
OPTIONAL MATCH (sensor)-[:BELONGS_TO]->(currentParent:RiverNode)
WITH sensor, currentParent, sensor.scope AS oldScope, currentParent.id AS oldParentId

// 2. SensorノードのUpsertとプロパティの設定
MERGE (sensor:Sensor {id: $id})
  ON CREATE SET
    sensor.altitude = $altitude,
    sensor.scope = $scope,
    sensor.interval = $interval
  ON MATCH SET
    sensor.altitude = $altitude,
    sensor.scope = $scope,
    sensor.interval = $interval

// 3. 新しいparent_nodeとscopeの設定
WITH sensor, oldScope, oldParentId, $parent_node AS newParentId, $scope AS newScope

// 4. scopeやparent_nodeの変更を検出し、変更があった場合のみ処理を実行
CALL apoc.do.when(
  // 条件: parent_nodeが異なる、またはscopeが異なる場合
  (oldParentId IS NULL OR oldParentId <> newParentId) OR (oldScope IS NULL OR oldScope <> newScope),
  '
    // 4.1. 既存のBELONGS_TO関係とAFFECTS関係の削除
    MATCH (sensor)-[r:BELONGS_TO]->(:RiverNode)
    DELETE r
    MATCH (sensor)-[a:AFFECTS]->(:RiverNode)
    DELETE a

    // 4.2. 新しいBELONGS_TO関係の作成
    MATCH (newParent:RiverNode {hilbert18: newParentId})
    CREATE (sensor)-[:BELONGS_TO]->(newParent)

    // 4.3. 新しいAFFECTS関係の作成
    CALL apoc.path.expandConfig(newParent, {
      relationshipFilter: "RIVER_LINK",
      labelFilter: "+RiverNode",
      bfs: true,
      maxLevel: 1000
    }) YIELD path

    WITH sensor, path, $newScope AS maxScope,
         reduce(total = 0, r IN relationships(path) | total + r.length) AS distance
    WHERE distance <= maxScope

    UNWIND nodes(path) AS node
    // distanceプロパティをAFFECTS関係に追加
    MERGE (sensor)-[affects:AFFECTS]->(node)
      ON CREATE SET affects.distance = distance
      ON MATCH SET affects.distance = distance

    // BELONGS_TO関係のあるノードにはAFFECTS関係を削除
    MATCH (sensor)-[:BELONGS_TO]->(parent:RiverNode)
    MATCH (sensor)-[a:AFFECTS]->(parent)
    DELETE a

  ',
  '', // 条件が偽の場合は何もしない
  {sensor: sensor, newParentId: newParentId, newScope: newScope}
) YIELD value

// 5. クエリの終了
RETURN sensor
            "#,
        )
            .param("id", query_params.id.to_string())
            .param("altitude", altitude)
            .param("scope", scope)
            .param("interval", interval)
            .param("parent_node", parent_node);

        let _ = self.graph.execute(query).await.unwrap();

        Ok(ApiSensorsPutResponse::Status200)
    }
}
