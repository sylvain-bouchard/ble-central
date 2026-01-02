/// Centralized error handling for the BLE Central Gateway
///
/// This module provides a consistent error type hierarchy that unifies
/// errors from different layers: BLE operations, MQTT communication,
/// configuration parsing, and application-level errors.
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use std::fmt;

/// The main error type for the application
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    /// BLE-related errors
    #[error("BLE error: {0}")]
    Ble(#[from] btleplug::Error),

    /// BLE backend errors
    #[error("BLE backend error: {0}")]
    BleBackend(#[from] crate::services::ble::error::BleError),

    /// MQTT-related errors
    #[error("MQTT error: {0}")]
    Mqtt(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// Device not found
    #[allow(dead_code)]
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// No Bluetooth adapter available
    #[error("No Bluetooth adapter found")]
    NoAdapter,

    /// Device not connected
    #[error("Device not connected")]
    NotConnected,

    /// Invalid input/request
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Internal error (unexpected conditions)
    #[error("Internal error: {0}")]
    Internal(String),
}

/// JSON error response for API endpoints
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Implement IntoResponse for AppError to integrate with Axum
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DeviceNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::NotConnected => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NoAdapter => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::Ble(_) | AppError::BleBackend(_) | AppError::Mqtt(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::Config(_) | AppError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };

        let body = Json(ErrorResponse {
            error: error_message,
        });

        (status, body).into_response()
    }
}

/// Allow conversion from rumqttc::ClientError to AppError
impl From<rumqttc::ClientError> for AppError {
    fn from(err: rumqttc::ClientError) -> Self {
        AppError::Mqtt(err.to_string())
    }
}

/// Allow conversion from rumqttc::ConnectionError to AppError
impl From<rumqttc::ConnectionError> for AppError {
    fn from(err: rumqttc::ConnectionError) -> Self {
        AppError::Mqtt(err.to_string())
    }
}

/// Allow conversion from std::io::Error to AppError
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

/// Allow conversion from String to AppError
impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Internal(err)
    }
}

/// Custom Display implementation for better error messages
impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}
