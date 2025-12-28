use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use super::vent_controller::{
    self, ConnectRequest, ConnectResponse, DiscoveredDevicesResponse, ErrorResponse,
    MessageResponse, ScanRequest, VentStatusResponse,
};
use crate::state::ApplicationState;

pub fn build_router(state: ApplicationState) -> Router {
    Router::new()
        // BLE operations
        .route("/api/ble/scan", post(ble_scan))
        .route("/api/ble/stop-scan", post(ble_stop_scan))
        .route("/api/ble/devices", get(ble_devices))
        .route("/api/ble/connect/{device_id}", post(ble_connect))
        // Vent operations
        .route("/api/vent/open", post(open_vent))
        .route("/api/vent/close", post(close_vent))
        .route("/api/vent/status", get(vent_status))
        .route("/api/vent/disconnect", post(vent_disconnect))
        .with_state(state)
}

/// Initialize BLE scanning
async fn ble_scan(
    State(state): State<ApplicationState>,
    Json(payload): Json<ScanRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    vent_controller::scan(&state, payload)
        .await
        .map(Json)
        .map_err(|(code, err)| (code, Json(err)))
}

/// Stop BLE scanning
async fn ble_stop_scan(
    State(state): State<ApplicationState>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    vent_controller::stop_scan(&state)
        .await
        .map(Json)
        .map_err(|(code, err)| (code, Json(err)))
}

/// Connect to a discovered BLE device
async fn ble_connect(
    State(state): State<ApplicationState>,
    Path(device_id): Path<String>,
    Json(payload): Json<ConnectRequest>,
) -> Result<Json<ConnectResponse>, (StatusCode, Json<ErrorResponse>)> {
    vent_controller::connect(&state, device_id, payload)
        .await
        .map(Json)
        .map_err(|(code, err)| (code, Json(err)))
}

/// Open the vent via BLE
async fn open_vent(
    State(state): State<ApplicationState>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    vent_controller::open(&state)
        .await
        .map(Json)
        .map_err(|(code, err)| (code, Json(err)))
}

/// Close the vent via BLE
async fn close_vent(
    State(state): State<ApplicationState>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    vent_controller::close(&state)
        .await
        .map(Json)
        .map_err(|(code, err)| (code, Json(err)))
}

/// Get current vent status
async fn vent_status(State(state): State<ApplicationState>) -> Json<VentStatusResponse> {
    Json(vent_controller::status(&state).await)
}

/// Disconnect from the device
async fn vent_disconnect(
    State(state): State<ApplicationState>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    vent_controller::disconnect(&state)
        .await
        .map(Json)
        .map_err(|(code, err)| (code, Json(err)))
}

/// Get list of discovered BLE devices
async fn ble_devices(State(state): State<ApplicationState>) -> Json<DiscoveredDevicesResponse> {
    Json(vent_controller::list_discovered_devices(&state).await)
}
