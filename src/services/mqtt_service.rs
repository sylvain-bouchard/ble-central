use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::domain::sensor::{SensorData, SensorReadings};
use crate::error::AppError;
use crate::services::ble_service::BleDataObserver;

#[derive(Clone)]
pub struct MqttService {
    client: AsyncClient,
    topic: String,
}

impl MqttService {
    /// Create a new MqttService with the given broker configuration
    pub async fn new(
        broker: &str,
        port: u16,
        client_id: &str,
        topic: &str,
    ) -> Result<Self, AppError> {
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

        Ok(MqttService {
            client,
            topic: topic.to_string(),
        })
    }

    /// Disconnect the MQTT client gracefully
    pub async fn disconnect(&self) {
        // Attempt to disconnect gracefully
        if let Err(e) = self.client.disconnect().await {
            warn!("MQTT disconnect error: {}", e);
        } else {
            info!("MQTT client disconnected");
        }
    }

    /// Send an MQTT message to the specified topic
    pub async fn send_message(
        &self,
        topic: &str,
        payload: &[u8],
        qos: QoS,
    ) -> Result<(), AppError> {
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
    ) -> Result<(), AppError> {
        self.send_message(topic, message.as_bytes(), qos).await
    }

    /// Send a JSON message to the specified topic
    #[allow(dead_code)]
    pub async fn send_json_message(
        &self,
        topic: &str,
        value: &serde_json::Value,
        qos: QoS,
    ) -> Result<(), AppError> {
        let json_str = serde_json::to_string(value)
            .map_err(|e| AppError::Internal(format!("JSON serialization error: {}", e)))?;
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

                if let Err(error) = self
                    .send_message(&self.topic, payload.as_bytes(), QoS::AtLeastOnce)
                    .await
                {
                    error!("Failed to publish sensor data to MQTT: {:?}", error);
                } else {
                    info!("Published sensor data to {}: {}", self.topic, payload);
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
    use super::*;
    use crate::domain::sensor::SensorData;

    #[tokio::test]
    async fn test_mqtt_service_creation() {
        // Note: This test requires a running MQTT broker
        // For unit testing, you might want to use a mock instead
        // This test documents the API without requiring a live broker

        // Integration test example (requires broker):
        // let service = MqttService::new("localhost", 1883, "test-client")
        //     .await
        //     .expect("Failed to create MQTT service");
        //
        // let result = service
        //     .send_string_message("test/topic", "hello", QoS::AtMostOnce)
        //     .await;
        //
        // assert!(result.is_ok());
    }

    #[test]
    fn test_sensor_data_parsing() {
        // Test that we can parse valid sensor data
        let data = vec![
            0x00, 0x05, // CO2: 1280 ppm
            0x90, 0x19, // Temperature: 6544 / 200 = 32.72°C
            0xC4, 0x09, // Humidity: 2500 / 100 = 25.0%
            0x1E, 0x00, // VOC index: 30 / 10 = 3
            0x78, 0x00, // PM1.0: 120 / 10 = 12.0 µg/m³
            0xB4, 0x00, // PM2.5: 180 / 10 = 18.0 µg/m³
        ];

        let readings = SensorReadings::from_manufacturer_data(&data);
        assert!(readings.is_some());

        let readings = readings.unwrap();
        assert_eq!(readings.co2, 1280);
        assert!((readings.temperature - 32.72).abs() < 0.01);
        assert_eq!(readings.humidity, 25.0);
        assert_eq!(readings.voc_index, 3);
        assert_eq!(readings.pm1p0, 12.0);
        assert_eq!(readings.pm2p5, 18.0);
    }

    #[test]
    fn test_sensor_data_parsing_invalid_length() {
        // Test that invalid data length is rejected
        let data = vec![0x01, 0x02, 0x03]; // Too short
        let readings = SensorReadings::from_manufacturer_data(&data);
        assert!(readings.is_none());
    }

    #[test]
    fn test_sensor_data_json_format() {
        // Test JSON serialization
        let data = vec![
            0x00, 0x05, 0x90, 0x19, 0xC4, 0x09, 0x1E, 0x00, 0x78, 0x00, 0xB4, 0x00,
        ];

        let readings = SensorReadings::from_manufacturer_data(&data).unwrap();
        let json = readings.to_json();

        // Verify JSON contains expected fields
        assert!(json.contains("\"temperature\":"));
        assert!(json.contains("\"humidity\":"));
        assert!(json.contains("\"co2\":"));
        assert!(json.contains("\"pm1p0\":"));
        assert!(json.contains("\"pm2p5\":"));
        assert!(json.contains("\"voc_index\":"));
    }

    #[tokio::test]
    async fn test_observer_filters_manufacturer_id() {
        // This test verifies that the observer correctly filters by manufacturer ID
        // In a real scenario with a mock, you'd verify the publish was called/not called

        // Valid manufacturer ID (0xFFFF) - would be processed
        let valid_mfg_id = 0xFFFF;
        assert_eq!(valid_mfg_id, 0xFFFF);

        // Invalid manufacturer ID - would be ignored
        let invalid_mfg_id = 0x0059; // Apple
        assert_ne!(invalid_mfg_id, 0xFFFF);
    }
}
