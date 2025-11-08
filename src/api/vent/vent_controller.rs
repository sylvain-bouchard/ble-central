use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::state::ApplicationState;

// Request/Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanRequest {
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub device_id: String,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectResponse {
    pub status: String,
    pub device_id: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VentStatusResponse {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// VentApiController handles all API logic for vent and BLE operations
#[derive(Clone)]
pub struct VentApiController {
    // Stateless controller - no fields needed
    // Methods will be called with ApplicationState passed from handlers
}

impl VentApiController {
    pub fn new() -> Self {
        VentApiController {}
    }

    /// Initialize BLE scanning
    pub async fn scan(
        &self,
        state: &ApplicationState,
        _payload: ScanRequest,
    ) -> Result<MessageResponse, (StatusCode, ErrorResponse)> {
        state.vent_service.initialize().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: format!("Failed to initialize scan: {}", e),
                },
            )
        })?;

        Ok(MessageResponse {
            status: "scanning".to_string(),
            message: "BLE scan started".to_string(),
        })
    }

    /// Connect to a discovered BLE device
    pub async fn connect(
        &self,
        state: &ApplicationState,
        device_id: String,
        payload: ConnectRequest,
    ) -> Result<ConnectResponse, (StatusCode, ErrorResponse)> {
        let timeout_secs = payload.timeout_secs.unwrap_or(10);

        state
            .vent_service
            .connect(&device_id, timeout_secs)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error: format!("Failed to connect to device: {}", e),
                    },
                )
            })?;

        Ok(ConnectResponse {
            status: "connected".to_string(),
            device_id,
            message: "Successfully connected to device".to_string(),
        })
    }

    /// Open the vent via BLE
    pub async fn open(
        &self,
        state: &ApplicationState,
    ) -> Result<MessageResponse, (StatusCode, ErrorResponse)> {
        state.vent_service.open_vent().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: format!("Failed to open vent: {}", e),
                },
            )
        })?;

        Ok(MessageResponse {
            status: "open".to_string(),
            message: "Vent opened successfully".to_string(),
        })
    }

    /// Close the vent via BLE
    pub async fn close(
        &self,
        state: &ApplicationState,
    ) -> Result<MessageResponse, (StatusCode, ErrorResponse)> {
        state.vent_service.close_vent().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: format!("Failed to close vent: {}", e),
                },
            )
        })?;

        Ok(MessageResponse {
            status: "closed".to_string(),
            message: "Vent closed successfully".to_string(),
        })
    }

    /// Get current vent status
    pub async fn status(&self, state: &ApplicationState) -> VentStatusResponse {
        let status = state.vent_service.get_vent_status().await;
        VentStatusResponse {
            status: format!("{:?}", status),
        }
    }

    /// Disconnect from the device
    pub async fn disconnect(
        &self,
        state: &ApplicationState,
    ) -> Result<MessageResponse, (StatusCode, ErrorResponse)> {
        state.vent_service.disconnect().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: format!("Failed to disconnect: {}", e),
                },
            )
        })?;

        Ok(MessageResponse {
            status: "disconnected".to_string(),
            message: "Disconnected from device".to_string(),
        })
    }

    /// Stop BLE scanning without connecting
    pub async fn stop_scan(
        &self,
        state: &ApplicationState,
    ) -> Result<MessageResponse, (StatusCode, ErrorResponse)> {
        state.vent_service.stop_scanning().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: format!("Failed to stop scanning: {}", e),
                },
            )
        })?;

        Ok(MessageResponse {
            status: "stopped".to_string(),
            message: "BLE scanning stopped".to_string(),
        })
    }
}

#[cfg(test)]
mod vent_controller_tests {
    include!("vent_controller_tests.rs");
}
