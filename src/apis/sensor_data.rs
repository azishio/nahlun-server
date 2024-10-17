use crate::apis::ServerImpl;
use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use chrono::Local;
use neo4rs::query;
use openapi::apis::sensor_data::{PostApiSensorsDataResponse, SensorData};
use openapi::models::{PostApiSensorsDataQueryParams, PostApiSensorsDataRequest};

#[async_trait]
impl SensorData for ServerImpl {
    async fn post_api_sensors_data(
        &self, _method: Method,
        _host: Host,
        _cookies: CookieJar,
        query_params: PostApiSensorsDataQueryParams,
        body: Option<PostApiSensorsDataRequest>,
    ) -> Result<PostApiSensorsDataResponse, String> {
        let body = body.unwrap();
        let PostApiSensorsDataRequest {
            distance,
            battery_voltage,
            previous_sleep_time,
            network_status,
        } = body;

        let _ = self.graph.execute(query(
            r#"
// 1. センサーの取得
MATCH (sensor:Sensor {id: $id})

// 2. オプショナルマッチで現在のデータを取得
OPTIONAL MATCH (sensor)-[:CURRENT_DATA]->(currentData:SensorData)

// 3. 新しいセンサーデータの作成
CREATE (newData:SensorData {
    distance: $distance,
    battery_voltage: $battery_voltage,
    previous_sleep_time: $previous_sleep_time,
    network_status: $network_status,
    time: $time
})

WITH sensor, currentData, newData

// 4. 条件分岐: currentData が存在するかどうか
CALL apoc.do.when(
    currentData IS NOT NULL,
    '
    // CURRENT_DATA リレーションシップを削除
    MATCH (sensor)-[r:CURRENT_DATA]->(currentData)
    DELETE r

    // 新しい CURRENT_DATA リレーションシップを作成
    CREATE (sensor)-[:CURRENT_DATA]->(newData)

    // PREVIOUS_DATA リレーションシップを作成
    CREATE (newData)-[:PREVIOUS_DATA]->(currentData)
    ',
    '
    // 新しい CURRENT_DATA リレーションシップを作成
    CREATE (sensor)-[:CURRENT_DATA]->(newData)
    ',
    // パラメータ
    {sensor: sensor, currentData: currentData, newData: newData}
) YIELD value

WITH sensor

// 5. センサーが所属する親河川ノードの取得
MATCH (sensor)-[:BELONGS_TO]->(parentRiverNode:RiverNode)

// 6. センサーの水位データを計算し、WaterLevel ノードを作成または更新
// 水位は sensor の altitude から distance を引いた値
MERGE (parentRiverNode)-[:WATER_LEVEL]->(parentWaterLevel:WaterLevel)
  ON CREATE SET parentWaterLevel.value = sensor.altitude - $distance
  ON MATCH SET parentWaterLevel.value = sensor.altitude - $distance

WITH sensor, parentRiverNode, parentWaterLevel

// 7. AFFECTS 関係を持つすべての河川ノードとその WaterLevel を取得
MATCH (sensor)-[:AFFECTS]->(affectedRiverNode:RiverNode)
MATCH (affectedRiverNode)<-[affectsLink:AFFECTS]-(:Sensor)-[:BELONGS_TO]->(:RiverNode)-[:WATER_LEVEL]->(waterLevel:WaterLevel)

// 8. 加重平均の計算に必要な値を準備
WITH affectedRiverNode, waterLevel.value AS value, 1.0 / affectsLink.distance AS weight

// 9. 水位の加重平均を計算
WITH affectedRiverNode,
     SUM(value * weight) AS weightedSum,
     SUM(weight) AS totalWeight

// 10. 加重平均を計算結果として保持
WITH affectedRiverNode,
     weightedSum / totalWeight AS avgWaterLevel

// 11. 計算された加重平均水位を WaterLevel ノードに設定
MERGE (affectedRiverNode)-[:WATER_LEVEL]->(wl:WaterLevel)
  ON CREATE SET wl.value = avgWaterLevel
  ON MATCH SET wl.value = avgWaterLevel
            "#,
        )
            .param("id", query_params.id.to_string())
            .param("distance", distance)
            .param("battery_voltage", battery_voltage)
            .param("previous_sleep_time", previous_sleep_time)
            .param("network_status", network_status)
            .param("time", Local::now().to_rfc3339())).await.unwrap();

        let interval = self.graph.execute(query(
            r#"
MATCH (sensor:Sensor {id: $id})
RETURN tointeger(sensor.interval) AS interval
            "#,
        )
            .param("id", query_params.id.to_string()))
            .await.unwrap().next()
            .await.unwrap().unwrap()
            .to::<i32>().unwrap();


        // TODO より細かい粒度でデータの更新を伝える
        self.socketio_client.emit("broadcast_request", Local::now().to_rfc3339()).await.unwrap();

        Ok(PostApiSensorsDataResponse::Status200_OK(interval))
    }
}
