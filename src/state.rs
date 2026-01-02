use std::sync::Arc;

use crate::services::ble::BleBackend;
use crate::services::mqtt_service::MqttService;
use crate::services::vent_service::VentService;

pub struct ApplicationState<B: BleBackend> {
    pub vent_service: Arc<VentService<B>>,
    pub mqtt_service: Arc<MqttService>,
}

impl<B: BleBackend> Clone for ApplicationState<B> {
    fn clone(&self) -> Self {
        Self {
            vent_service: Arc::clone(&self.vent_service),
            mqtt_service: Arc::clone(&self.mqtt_service),
        }
    }
}
