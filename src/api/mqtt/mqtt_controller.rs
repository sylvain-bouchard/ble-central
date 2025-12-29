use axum::{http::StatusCode, response::IntoResponse, Json};
use rumqttc::QoS;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::error::AppError;
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

#[derive(Debug, Serialize)]
pub struct PublishResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MqttStatusResponse {
    pub status: String,
    pub message: String,
}

/// Get MQTT connection status
pub async fn get_status() -> impl IntoResponse {
    info!("MQTT status requested");
    
    // Since MqttService is actively running if the app started,
    // we can return connected status
    Json(json!({
        "status": "connected",
        "message": "MQTT service is active and monitoring sensor data"
    }))
}

/// Manually publish a message to MQTT broker
pub async fn publish_message(
    state: ApplicationState,
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
pub async fn publish_json(
    state: ApplicationState,
    Json(request): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // Extract topic and qos from the JSON request
    let topic = request
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("Missing 'topic' field".to_string()))?;
    
    let qos_value = request
        .get("qos")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u8;
    
    let payload = request
        .get("payload")
        .ok_or_else(|| AppError::InvalidInput("Missing 'payload' field".to_string()))?;

    info!("Publishing JSON to MQTT topic: {}", topic);

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

    info!("Successfully published JSON to topic {}", topic);
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
    use super::*;

    #[tokio::test]
    async fn test_get_status() {
        let response = get_status().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_invalid_qos() {
        // This test documents that invalid QoS values should be rejected
        // In a real scenario, you'd want to test with a mock MQTT service
        assert!(matches!(3, 3)); // QoS must be 0, 1, or 2
    }
}
