use neo4rs::{Labels, Point2D};
use openapi::models::Coord3D;
use serde::Deserialize;
use uuid::Uuid;

/// 河川のノードを表すノード
/// このノード同士を:RIVER_LINKラベルのリレーションシップで結ぶことで、河川の形状を表現する
/// このノード同士を:DELAUNAYラベルのリレーションシップで結ぶことで、隣接するノードを表現する
#[derive(Deserialize)]
pub struct RiverNode {
    pub labels: Labels,
    pub location: Point2D,
    pub hilbert18: u32,
    pub altitude: f32,
}

impl From<RiverNode> for openapi::models::RiverNode {
    fn from(node: RiverNode) -> openapi::models::RiverNode {
        let coord = Coord3D {
            longitude: node.location.x(),
            latitude: node.location.y(),
            altitude: node.altitude,
        };
        openapi::models::RiverNode {
            id: node.hilbert18 as i64,
            coord,
        }
    }
}

/// 水位計を表すノード
/// RiverNodeに対して:AFFECTSラベルのリレーションシップで結ぶことで、水位の予測に影響を与えるノードを指定する
#[derive(Deserialize)]
pub struct Sensor {
    pub name: String,
    pub id: Uuid,
    pub location: Point2D,
    pub altitude: f32,
    pub interval: u32,
}

/// 水位計のデータ
/// Sensorに対して:CURRENT_DATAラベルのリレーションシップで結ぶことで、水位計のデータを指定する
/// SensorData同士を:PREVIOUS_DATAラベルのリレーションシップで結ぶことで、データの時間軸を表現する
#[derive(Deserialize)]
pub struct SensorData {
    /// UNIX時間 [sec]
    pub unix_time: u64,
    /// バッテリー電圧 [mV]
    pub battery_voltage: f32,
    /// センサーが前回スリープした時間 [sec]
    pub previous_sleep_time: u32,
    /// 平均処理されたセンサーの値
    pub value: f32,
}

/// カスタムボクセルタイルのソースを表す
#[derive(Deserialize)]
pub struct VoxelModel {
    pub id: Uuid,
    pub name: String,
    pub max_ll: Point2D,
    pub min_ll: Point2D,
}
