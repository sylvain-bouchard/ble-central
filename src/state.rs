use std::sync::Arc;

use crate::api::vent::vent_controller::VentApiController;
use crate::services::vent_service::VentService;

#[derive(Clone)]
pub struct ApplicationState {
    pub vent_service: Arc<VentService>,
    pub vent_api_controller: Arc<VentApiController>,
}
