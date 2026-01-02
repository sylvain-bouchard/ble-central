use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use super::vent_controller::{
    self, ConnectRequest, ConnectResponse, DiscoveredDevicesResponse, MessageResponse, ScanRequest,
    VentStatusResponse,
};
use crate::api::health::health_controller;
use crate::api::mqtt::mqtt_routes;
use crate::services::ble::BleBackend;
use crate::state::ApplicationState;

pub fn build_router<B: BleBackend + 'static>(state: ApplicationState<B>) -> Router {
    // Version 1 API routes
    let v1_routes = Router::new()
        // Health check
        .route("/health", get(health_check::<B>))
        // BLE operations
        .route("/ble/scan", post(ble_scan::<B>))
        .route("/ble/stop-scan", post(ble_stop_scan::<B>))
        .route("/ble/devices", get(ble_devices::<B>))
        .route("/ble/connect/{device_id}", post(ble_connect::<B>))
        // Vent operations
        .route("/vent/open", post(open_vent::<B>))
        .route("/vent/close", post(close_vent::<B>))
        .route("/vent/status", get(vent_status::<B>))
        .route("/vent/disconnect", post(vent_disconnect::<B>))
        // MQTT operations
        .nest("/mqtt", mqtt_routes::mqtt_routes());

    Router::new().nest("/api/v1", v1_routes).with_state(state)
}

/// Health check endpoint
async fn health_check<B: BleBackend>(State(state): State<ApplicationState<B>>) -> impl axum::response::IntoResponse {
    health_controller::health_check(&state).await
}

/// Initialize BLE scanning
async fn ble_scan<B: BleBackend>(
    State(state): State<ApplicationState<B>>,
    Json(payload): Json<ScanRequest>,
) -> Result<Json<MessageResponse>, impl axum::response::IntoResponse> {
    vent_controller::scan(&state, payload).await.map(Json)
}

/// Stop BLE scanning
async fn ble_stop_scan<B: BleBackend>(
    State(state): State<ApplicationState<B>>,
) -> Result<Json<MessageResponse>, impl axum::response::IntoResponse> {
    vent_controller::stop_scan(&state).await.map(Json)
}

/// Connect to a discovered BLE device
async fn ble_connect<B: BleBackend>(
    State(state): State<ApplicationState<B>>,
    Path(device_id): Path<String>,
    Json(payload): Json<ConnectRequest>,
) -> Result<Json<ConnectResponse>, impl axum::response::IntoResponse> {
    vent_controller::connect(&state, device_id, payload)
        .await
        .map(Json)
}

/// Open the vent via BLE
async fn open_vent<B: BleBackend>(
    State(state): State<ApplicationState<B>>,
) -> Result<Json<MessageResponse>, impl axum::response::IntoResponse> {
    vent_controller::open(&state).await.map(Json)
}

/// Close the vent via BLE
async fn close_vent<B: BleBackend>(
    State(state): State<ApplicationState<B>>,
) -> Result<Json<MessageResponse>, impl axum::response::IntoResponse> {
    vent_controller::close(&state).await.map(Json)
}

/// Get current vent status
async fn vent_status<B: BleBackend>(State(state): State<ApplicationState<B>>) -> Json<VentStatusResponse> {
    Json(vent_controller::status(&state).await)
}

/// Disconnect from the device
async fn vent_disconnect<B: BleBackend>(
    State(state): State<ApplicationState<B>>,
) -> Result<Json<MessageResponse>, impl axum::response::IntoResponse> {
    vent_controller::disconnect(&state).await.map(Json)
}

/// Get list of discovered BLE devices
async fn ble_devices<B: BleBackend>(State(state): State<ApplicationState<B>>) -> Json<DiscoveredDevicesResponse> {
    Json(vent_controller::list_discovered_devices(&state).await)
}
