use std::sync::Arc;

use crate::configuration::Settings;
use crate::error::AppError;
use crate::services::ble_service::BleService;
use crate::services::mqtt_service::MqttService;
use crate::services::vent_service::VentService;
use crate::state::ApplicationState;

/// Main application struct that orchestrates service and controller initialization
pub struct Application {
    pub state: ApplicationState,
    pub listen_address: String,
}

impl Application {
    /// Create a new Application with all services initialized in the proper order
    pub async fn new(config: &Settings, listen_address: &str) -> Result<Self, AppError> {
        let ble_service = BleService::new(config.ble.observer_timeout_secs).await?;
        let vent_service = Arc::new(VentService::new(
            ble_service,
            config.ble.connection_timeout_secs,
        ));
        let mqtt_service = Arc::new(
            MqttService::new(
                &config.mqtt.broker,
                config.mqtt.port,
                &config.mqtt.client_id,
                &config.mqtt.topic,
                config.mqtt.keep_alive_secs,
                config.mqtt.manufacturer_id,
            )
            .await?,
        );

        // Register MQTT service as an observer to BLE service for sensor data
        vent_service
            .register_ble_observer(mqtt_service.clone())
            .await;

        let state = ApplicationState {
            vent_service,
            mqtt_service,
        };

        Ok(Application {
            state,
            listen_address: listen_address.to_string(),
        })
    }

    /// Get the application state
    pub fn state(&self) -> &ApplicationState {
        &self.state
    }

    /// Get the listen address
    pub fn listen_address(&self) -> &str {
        &self.listen_address
    }

    /// Gracefully shutdown the application and cleanup resources
    pub async fn shutdown(&self) {
        tracing::info!("Initiating graceful shutdown...");

        // Stop BLE scanning
        if let Err(e) = self.state.vent_service.stop_scanning().await {
            tracing::warn!("Error stopping BLE scan during shutdown: {}", e);
        }

        // Disconnect from any connected devices
        if let Err(e) = self.state.vent_service.disconnect().await {
            tracing::warn!("Error disconnecting device during shutdown: {}", e);
        }

        // Disconnect MQTT client
        self.state.mqtt_service.disconnect().await;

        tracing::info!("Shutdown complete");
    }
}
