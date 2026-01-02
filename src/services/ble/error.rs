use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum BleError {
    #[error("No Bluetooth adapter available")]
    NoAdapter,

    #[error("Device not found")]
    DeviceNotFound,

    #[error("Device not connected")]
    NotConnected,

    #[error("Operation timed out")]
    Timeout,

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("Characteristic not found")]
    CharacteristicNotFound,

    #[error("Backend error: {0}")]
    Backend(String),
}
