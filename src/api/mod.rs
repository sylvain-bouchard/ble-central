use crate::services::ble::BtleplugBackend;
use crate::state::ApplicationState;

pub mod health;
pub mod mqtt;
pub mod vent;

pub type DefaultApplicationState = ApplicationState<BtleplugBackend>;
