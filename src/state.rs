use std::sync::Arc;

use crate::services::vent_service::VentService;

#[derive(Clone)]
pub struct ApplicationState {
    pub vent_service: Arc<VentService>,
}
