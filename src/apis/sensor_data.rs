use crate::apis::ServerImpl;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use chrono::Local;
use neo4rs::query;
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
        let body = body.unwrap();
        let PostApiSensorsDataRequest {
            distance,
            battery_voltage,
            previous_sleep_time,
            network_status,
        } = body;

        let query = query(
            r#"
// 1. センサーの取得
MATCH (sensor:Sensor {id: $id})

// 2. オプショナルマッチで現在のデータを取得
OPTIONAL MATCH (sensor)-[:CURRENT_DATA]->(current_data:SensorData)

// 3. 新しいセンサーデータの作成
CREATE (new_data:SensorData {
    distance: $distance,
    battery_voltage: $battery_voltage,
    previous_sleep_time: $previous_sleep_time,
    network_status: $network_status,
    time: $time
})

// 4. 条件分岐: current_data が存在するかどうか
CALL apoc.do.when(
    current_data IS NOT NULL,
    // IF 部分: current_data が存在する場合の処理
    '
    // CURRENT_DATA リレーションシップを削除
    MATCH (sensor)-[:CURRENT_DATA]->(current_data)
    DELETE (sensor)-[:CURRENT_DATA]->(current_data)
    
    // 新しい CURRENT_DATA リレーションシップを作成
    CREATE (sensor)-[:CURRENT_DATA]->(new_data)
    
    // PREVIOUS_DATA リレーションシップを作成
    CREATE (new_data)-[:PREVIOUS_DATA]->(current_data)
    ',
    // ELSE 部分: current_data が存在しない場合の処理
    '
    // 新しい CURRENT_DATA リレーションシップを作成
    CREATE (sensor)-[:CURRENT_DATA]->(new_data)
    ',
    // パラメータ
    {sensor: sensor, current_data: current_data, new_data: new_data}
) YIELD value

// -----------------------------
// 各ノードの水位を計算

// 5. センサーが所属する親河川ノードの取得
MATCH (sensor)-[:BELONGS_TO]->(parent_river_node:RiverNode)

// 6. センサーの水位データを計算し、WaterLevel ノードを作成または更新
// 水位は sensor の altitude から distance を引いた値
MERGE (parent_river_node)-[:WATER_LEVEL]->(parent_water_level:WaterLevel)
  ON CREATE SET parent_water_level.value = sensor.altitude - $distance
  ON MATCH SET parent_water_level.value = sensor.altitude - $distance

// 7. AFFECTS 関係を持つすべての河川ノードとその WaterLevel を取得
MATCH (sensor)-[:AFFECTS]->(affected_river_node:RiverNode)
MATCH (affected_river_node)<-[affects_link:AFFECTS]-(:Sensor)-[:BELONGS_TO]->(:RiverNode)-[:WATER_LEVEL]->(water_level:WaterLevel)

// 8. 加重平均の計算に必要な値を準備
WITH affected_river_node, water_level.value AS value, 1.0 / affects_link.distance AS weight

// 9. 水位の加重平均を計算
WITH affected_river_node, 
     SUM(value * weight) AS weightedSum, 
     SUM(weight) AS totalWeight

// 10. 加重平均を計算結果として保持
WITH affected_river_node, 
     weightedSum / totalWeight AS avgWaterLevel

// 11. 計算された加重平均水位を WaterLevel ノードに設定
MERGE (affected_river_node)-[:WATER_LEVEL]->(wl:WaterLevel)
  ON CREATE SET wl.value = avgWaterLevel
  ON MATCH SET wl.value = avgWaterLevel

// 12. クエリの終了
RETURN sensor.interval AS interval
            "#,
        )
            .param("id", query_params.id.to_string())
            .param("distance", distance)
            .param("battery_voltage", battery_voltage)
            .param("previous_sleep_time", previous_sleep_time)
            .param("network_status", network_status)
            .param("time", Local::now().to_rfc3339());

        let interval = self.graph.execute(query)
            .await.unwrap().next()
            .await.unwrap().unwrap()
            .to::<i32>().unwrap();

        // TODO より細かい粒度でデータの更新を伝える
        self.socketio_client.emit("broadcast_request", Local::now().to_rfc3339()).unwrap();

        Ok(PostApiSensorsDataResponse::Status200_OK(interval))
    }
}
