use axum::{http::StatusCode, response::IntoResponse, Json};
use rumqttc::QoS;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info};

use crate::error::AppError;
use crate::services::ble::BleBackend;
use crate::state::ApplicationState;

#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    pub topic: String,
    pub message: String,
    #[serde(default = "default_qos")]
    pub qos: u8, // 0 = AtMostOnce, 1 = AtLeastOnce, 2 = ExactlyOnce
}

fn default_qos() -> u8 {
    0
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct PublishResponse {
    pub status: String,
    pub message: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct MqttStatusResponse {
    pub status: String,
    pub message: String,
}

/// Get MQTT connection status
pub async fn get_status<B: BleBackend>(state: ApplicationState<B>) -> impl IntoResponse {
    info!("MQTT status requested");

    let is_connected = state.mqtt_service.is_connected().await;

    if is_connected {
        Json(json!({
            "status": "connected",
            "message": "MQTT service is connected and monitoring sensor data"
        }))
    } else {
        Json(json!({
            "status": "disconnected",
            "message": "MQTT service is disconnected or attempting to reconnect"
        }))
    }
}

/// Manually publish a message to MQTT broker
pub async fn publish_message<B: BleBackend>(
    state: ApplicationState<B>,
    Json(request): Json<PublishRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Publishing message to MQTT topic: {}", request.topic);

    let qos = match request.qos {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => {
            return Err(AppError::InvalidInput(format!(
                "Invalid QoS value: {}. Must be 0, 1, or 2",
                request.qos
            )));
        }
    };

    state
        .mqtt_service
        .send_string_message(&request.topic, &request.message, qos)
        .await?;

    info!(
        "Successfully published to topic {}: {}",
        request.topic, request.message
    );
    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "published",
            "message": format!("Message published to topic '{}'", request.topic),
            "topic": request.topic,
            "qos": request.qos
        })),
    ))
}

/// Publish JSON data to MQTT broker
pub async fn publish_json<B: BleBackend>(
    state: ApplicationState<B>,
    Json(request): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // Extract topic and qos from the JSON request
    let topic = request
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("Missing 'topic' field".to_string()))?;

    let qos_value = request.get("qos").and_then(|v| v.as_u64()).unwrap_or(0) as u8;

    let payload = request
        .get("payload")
        .ok_or_else(|| AppError::InvalidInput("Missing 'payload' field".to_string()))?;

    debug!("Publishing JSON to MQTT topic: {}", topic);

    let qos = match qos_value {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => {
            return Err(AppError::InvalidInput(format!(
                "Invalid QoS value: {}. Must be 0, 1, or 2",
                qos_value
            )));
        }
    };

    state
        .mqtt_service
        .send_json_message(topic, payload, qos)
        .await?;

    debug!("Successfully published JSON to topic {}", topic);
    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "published",
            "message": format!("JSON published to topic '{}'", topic),
            "topic": topic,
            "qos": qos_value
        })),
    ))
}

#[cfg(test)]
mod mqtt_controller_tests {
    include!("mqtt_controller_tests.rs");
}
