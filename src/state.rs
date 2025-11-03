use std::sync::Arc;

use crate::domain::vent::controller::VentController;

#[derive(Clone)]
pub struct ApplicationState {
    pub vent: Arc<VentController>,
}
