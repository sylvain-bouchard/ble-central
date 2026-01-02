use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use super::vent_controller::{
    self, ConnectRequest, ConnectResponse, DiscoveredDevicesResponse, MessageResponse, ScanRequest,
    VentStatusResponse,
};
use crate::api::{health::health_controller, DefaultApplicationState};
use crate::api::mqtt::mqtt_routes;

pub fn build_router(state: DefaultApplicationState) -> Router {
    // Version 1 API routes
    let v1_routes = Router::new()
        // Health check
        .route("/health", get(health_check))
        // BLE operations
        .route("/ble/scan", post(ble_scan))
        .route("/ble/stop-scan", post(ble_stop_scan))
        .route("/ble/devices", get(ble_devices))
        .route("/ble/connect/{device_id}", post(ble_connect))
        // Vent operations
        .route("/vent/open", post(open_vent))
        .route("/vent/close", post(close_vent))
        .route("/vent/status", get(vent_status))
        .route("/vent/disconnect", post(vent_disconnect))
        // MQTT operations
        .nest("/mqtt", mqtt_routes::mqtt_routes());

    Router::new().nest("/api/v1", v1_routes).with_state(state)
}

/// Health check endpoint
async fn health_check(State(state): State<DefaultApplicationState>) -> impl axum::response::IntoResponse {
    health_controller::health_check(&state).await
}

/// Initialize BLE scanning
async fn ble_scan(
    State(state): State<DefaultApplicationState>,
    Json(payload): Json<ScanRequest>,
) -> Result<Json<MessageResponse>, impl axum::response::IntoResponse> {
    vent_controller::scan(&state, payload).await.map(Json)
}

/// Stop BLE scanning
async fn ble_stop_scan(
    State(state): State<DefaultApplicationState>,
) -> Result<Json<MessageResponse>, impl axum::response::IntoResponse> {
    vent_controller::stop_scan(&state).await.map(Json)
}

/// Connect to a discovered BLE device
async fn ble_connect(
    State(state): State<DefaultApplicationState>,
    Path(device_id): Path<String>,
    Json(payload): Json<ConnectRequest>,
) -> Result<Json<ConnectResponse>, impl axum::response::IntoResponse> {
    vent_controller::connect(&state, device_id, payload)
        .await
        .map(Json)
}

/// Open the vent via BLE
async fn open_vent(
    State(state): State<DefaultApplicationState>,
) -> Result<Json<MessageResponse>, impl axum::response::IntoResponse> {
    vent_controller::open(&state).await.map(Json)
}

/// Close the vent via BLE
async fn close_vent(
    State(state): State<DefaultApplicationState>,
) -> Result<Json<MessageResponse>, impl axum::response::IntoResponse> {
    vent_controller::close(&state).await.map(Json)
}

/// Get current vent status
async fn vent_status(State(state): State<DefaultApplicationState>) -> Json<VentStatusResponse> {
    Json(vent_controller::status(&state).await)
}

/// Disconnect from the device
async fn vent_disconnect(
    State(state): State<DefaultApplicationState>,
) -> Result<Json<MessageResponse>, impl axum::response::IntoResponse> {
    vent_controller::disconnect(&state).await.map(Json)
}

/// Get list of discovered BLE devices
async fn ble_devices(State(state): State<DefaultApplicationState>) -> Json<DiscoveredDevicesResponse> {
    Json(vent_controller::list_discovered_devices(&state).await)
}
