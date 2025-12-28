use std::sync::Arc;

use crate::services::mqtt_service::MqttService;
use crate::services::vent_service::VentService;

#[derive(Clone)]
pub struct ApplicationState {
    pub vent_service: Arc<VentService>,
    pub mqtt_service: Arc<MqttService>,
}
