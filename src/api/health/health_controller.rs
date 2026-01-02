use axum::Json;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use super::super::DefaultApplicationState;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub services: ServiceHealth,
}

/// Individual service health statuses
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub ble: String,
    pub mqtt: String,
}

// Store application start time
static START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();

/// Initialize the start time (should be called once at application startup)
pub fn init_start_time() {
    START_TIME.get_or_init(SystemTime::now);
}

/// Health check endpoint - returns system status and service health
pub async fn health_check(state: &DefaultApplicationState) -> Json<HealthResponse> {
    let start_time = START_TIME.get().copied().unwrap_or_else(SystemTime::now);
    let uptime_seconds = SystemTime::now()
        .duration_since(start_time)
        .unwrap_or_default()
        .as_secs();

    // Check BLE service health by checking if we have an adapter
    let ble_status = if state.vent_service.has_adapter().await {
        "ok".to_string()
    } else {
        "no_adapter".to_string()
    };

    let mqtt_status = if state.mqtt_service.is_connected().await {
        "ok".to_string()
    } else {
        "disconnected".to_string()
    };

    let overall_status = if ble_status == "ok" && mqtt_status == "ok" {
        "ok".to_string()
    } else {
        "degraded".to_string()
    };

    Json(HealthResponse {
        status: overall_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
        services: ServiceHealth {
            ble: ble_status,
            mqtt: mqtt_status,
        },
    })
}
