use axum::{
    extract::State,
    routing::{get, post},
    Router,
};

use crate::api::mqtt::mqtt_controller;
use crate::services::ble::BleBackend;
use crate::state::ApplicationState;

pub fn mqtt_routes<B: BleBackend + 'static>() -> Router<ApplicationState<B>> {
    Router::new()
        .route("/status", get(mqtt_status_handler::<B>))
        .route("/publish", post(publish_message_handler::<B>))
        .route("/publish/json", post(publish_json_handler::<B>))
}

async fn mqtt_status_handler<B: BleBackend>(State(state): State<ApplicationState<B>>) -> impl axum::response::IntoResponse {
    mqtt_controller::get_status(state).await
}

async fn publish_message_handler<B: BleBackend>(
    State(state): State<ApplicationState<B>>,
    payload: axum::Json<mqtt_controller::PublishRequest>,
) -> Result<impl axum::response::IntoResponse, impl axum::response::IntoResponse> {
    mqtt_controller::publish_message(state, payload).await
}

async fn publish_json_handler<B: BleBackend>(
    State(state): State<ApplicationState<B>>,
    payload: axum::Json<serde_json::Value>,
) -> Result<impl axum::response::IntoResponse, impl axum::response::IntoResponse> {
    mqtt_controller::publish_json(state, payload).await
}
