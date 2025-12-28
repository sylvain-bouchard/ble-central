use axum::{
    extract::State,
    routing::{get, post},
    Router,
};

use crate::api::mqtt::mqtt_controller;
use crate::state::ApplicationState;

pub fn mqtt_routes() -> Router<ApplicationState> {
    Router::new()
        .route("/status", get(mqtt_status_handler))
        .route("/publish", post(publish_message_handler))
        .route("/publish/json", post(publish_json_handler))
}

async fn mqtt_status_handler() -> impl axum::response::IntoResponse {
    mqtt_controller::get_status().await
}

async fn publish_message_handler(
    State(state): State<ApplicationState>,
    payload: axum::Json<mqtt_controller::PublishRequest>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    mqtt_controller::publish_message(state, payload).await
}

async fn publish_json_handler(
    State(state): State<ApplicationState>,
    payload: axum::Json<serde_json::Value>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    mqtt_controller::publish_json(state, payload).await
}
