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

// 5. 新しいデータを返す
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
