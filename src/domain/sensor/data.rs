/// Trait for sensor data that can be serialized to JSON
pub trait SensorData: Send + Sync {
    fn to_json(&self) -> String;
}
