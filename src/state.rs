use std::sync::Arc;

use crate::api::vent::vent_controller::VentApiController;
use crate::services::vent_service::VentService;
use crate::services::mqtt_service::MqttService;

#[derive(Clone)]
pub struct ApplicationState {
    pub vent_service: Arc<VentService>,
    pub vent_api_controller: Arc<VentApiController>,
    pub mqtt_service: Arc<MqttService>,
}
