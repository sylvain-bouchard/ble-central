use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::domain::sensor::{SensorData, SensorReadings};
use crate::services::ble_service::BleDataObserver;

#[derive(Clone)]
pub struct MqttService {
    client: AsyncClient,
}

impl MqttService {
    /// Create a new MqttService with the given broker configuration
    pub async fn new(
        broker: &str,
        port: u16,
        client_id: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut mqtt_options = MqttOptions::new(client_id, broker, port);
        mqtt_options.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

        // Spawn a task to handle MQTT events with error throttling
        tokio::spawn(async move {
            let mut last_error: Option<String> = None;
            let mut consecutive_count = 0u32;
            let mut suppressed = false;

            loop {
                if let Err(e) = eventloop.poll().await {
                    let error_msg = format!("{:?}", e);

                    // Check if this is the same error as before
                    if last_error.as_ref() == Some(&error_msg) {
                        consecutive_count += 1;

                        // Log warning only once when hitting 10 repetitions
                        if consecutive_count == 10 && !suppressed {
                            warn!(
                                "MQTT error: {} (repeated {} times, suppressing further messages)",
                                error_msg, consecutive_count
                            );
                            suppressed = true;
                        }
                    } else {
                        // New error type
                        if last_error.is_some() && consecutive_count > 0 {
                            info!(
                                "Previous MQTT error resolved after {} occurrences",
                                consecutive_count + 1
                            );
                        }
                        error!("MQTT error: {}", error_msg);
                        last_error = Some(error_msg);
                        consecutive_count = 0;
                        suppressed = false;
                    }
                }
            }
        });

        Ok(MqttService { client })
    }

    /// Send an MQTT message to the specified topic
    pub async fn send_message(
        &self,
        topic: &str,
        payload: &[u8],
        qos: QoS,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client.publish(topic, qos, false, payload).await?;
        info!(
            "Published to {}: {:?}",
            topic,
            String::from_utf8_lossy(payload)
        );
        Ok(())
    }

    /// Send a string message to the specified topic
    #[allow(dead_code)]
    pub async fn send_string_message(
        &self,
        topic: &str,
        message: &str,
        qos: QoS,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.send_message(topic, message.as_bytes(), qos).await
    }

    /// Send a JSON message to the specified topic
    #[allow(dead_code)]
    pub async fn send_json_message(
        &self,
        topic: &str,
        value: &serde_json::Value,
        qos: QoS,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json_str = serde_json::to_string(value)?;
        self.send_message(topic, json_str.as_bytes(), qos).await
    }
}

#[async_trait::async_trait]
impl BleDataObserver for MqttService {
    async fn on_sensor_data(&self, id: String, manufacturer_id: u16, data: Arc<[u8]>) {
        // Only process manufacturer ID 0xFFFF (65535)
        if manufacturer_id != 0xFFFF {
            return;
        }

        info!(
            "Received sensor data from {} (mfg_id: 0x{:04X}): {} bytes",
            id,
            manufacturer_id,
            data.len()
        );

        // Parse the sensor data into SensorReadings
        match SensorReadings::from_manufacturer_data(&data) {
            Some(readings) => {
                let payload = readings.to_json();
                let topic = "living_room/air_quality/data";

                if let Err(error) = self
                    .send_message(topic, payload.as_bytes(), QoS::AtLeastOnce)
                    .await
                {
                    error!("Failed to publish sensor data to MQTT: {:?}", error);
                } else {
                    info!("Published sensor data to {}: {}", topic, payload);
                }
            }
            None => {
                error!(
                    "Failed to parse sensor data from device {}: {} bytes",
                    id,
                    data.len()
                );
            }
        }
    }
}

#[cfg(test)]
mod mqtt_service_tests {
    #[tokio::test]
    async fn test_mqtt_service_creation() {
        // Note: This test requires a running MQTT broker
        // For unit testing, you might want to use a mock instead
        // This is a placeholder test that documents the API

        // let service = MqttService::new("localhost", 1883, "test-client")
        //     .await
        //     .expect("Failed to create MQTT service");

        // let result = service
        //     .send_string_message("test/topic", "hello", QoS::AtMostOnce)
        //     .await;

        // assert!(result.is_ok());
    }
}
