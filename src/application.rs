use std::error::Error;
use std::sync::Arc;

use crate::api::vent::vent_controller::VentApiController;
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
    pub async fn new(
        mqtt_broker: &str,
        mqtt_port: u16,
        mqtt_client_id: &str,
        listen_address: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let ble_service = BleService::new().await;
        let vent_service = Arc::new(VentService::new(ble_service));
        let mqtt_service =
            Arc::new(MqttService::new(mqtt_broker, mqtt_port, mqtt_client_id).await?);

        // Register MQTT service as an observer to BLE service for sensor data
        vent_service
            .register_ble_observer(mqtt_service.clone())
            .await;

        let vent_api_controller = Arc::new(VentApiController::new());

        let state = ApplicationState {
            vent_service,
            mqtt_service,
            vent_api_controller,
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
}
