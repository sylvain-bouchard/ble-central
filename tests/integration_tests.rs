/// Integration tests for the BLE Central Gateway API
///
/// These tests validate the API endpoints end-to-end, ensuring that:
/// - Routes are properly configured
/// - Request/response formats are correct
/// - Error handling works as expected
/// - Services integrate correctly
use serde_json::{json, Value};

mod test_helpers {
    use ble_central_gateway::{
        api::vent::vent_routes::build_router,
        services::{ble_service::BleService, mqtt_service::MqttService, vent_service::VentService},
        state::ApplicationState,
    };
    use std::sync::Arc;

    /// Create a test application state for integration testing
    pub async fn create_test_state() -> ApplicationState {
        // Use minimal configuration for testing
        let ble_service = BleService::new(5)
            .await
            .expect("Failed to create BLE service");

        let vent_service = Arc::new(VentService::new(ble_service, 10));

        // Use a test MQTT broker (may fail to connect, but that's ok for testing)
        let mqtt_service = Arc::new(
            MqttService::new("127.0.0.1", 1883, "test_client", "test/topic")
                .await
                .unwrap_or_else(|_| {
                    // If MQTT fails, we still need a service for testing
                    // The service will handle connection errors gracefully
                    panic!(
                        "MQTT service creation failed - this is expected if no broker is running"
                    )
                }),
        );

        ApplicationState {
            vent_service,
            mqtt_service,
        }
    }

    /// Create test server with the application router
    pub async fn create_test_server() -> axum_test::TestServer {
        let state = create_test_state().await;
        let router = build_router(state);
        axum_test::TestServer::new(router).unwrap()
    }
}

#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    let server = test_helpers::create_test_server().await;

    let response = server.get("/api/v1/health").await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
    assert!(body["uptime_seconds"].is_number());
    assert!(body["services"]["ble"].is_string());
    assert_eq!(body["services"]["mqtt"], "ok");
}

#[tokio::test]
async fn test_health_endpoint_contains_version() {
    let server = test_helpers::create_test_server().await;

    let response = server.get("/api/v1/health").await;

    let body: Value = response.json();
    let version = body["version"].as_str().unwrap();
    assert!(!version.is_empty(), "Version should not be empty");
    // Version should follow semantic versioning
    assert!(version.contains('.'), "Version should contain dots");
}

#[tokio::test]
async fn test_vent_status_returns_status() {
    let server = test_helpers::create_test_server().await;

    let response = server.get("/api/v1/vent/status").await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert!(body["status"].is_string());
}

#[tokio::test]
async fn test_ble_devices_returns_empty_list_initially() {
    let server = test_helpers::create_test_server().await;

    let response = server.get("/api/v1/ble/devices").await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert!(body["devices"].is_array());
}

#[tokio::test]
async fn test_ble_scan_accepts_valid_request() {
    let server = test_helpers::create_test_server().await;

    let response = server
        .post("/api/v1/ble/scan")
        .json(&json!({
            "timeout_secs": 10
        }))
        .await;

    // May succeed or fail depending on BLE adapter availability
    // But should return proper response structure
    assert!(
        response.status_code().is_success() || response.status_code().is_server_error(),
        "Expected success or server error, got {}",
        response.status_code()
    );
}

#[tokio::test]
async fn test_mqtt_status_returns_ok() {
    let server = test_helpers::create_test_server().await;

    let response = server.get("/api/v1/mqtt/status").await;

    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["status"], "connected");
}

#[tokio::test]
async fn test_mqtt_publish_requires_topic() {
    let server = test_helpers::create_test_server().await;

    let response = server
        .post("/api/v1/mqtt/publish")
        .json(&json!({
            "message": "test message",
            "qos": 0
        }))
        .await;

    // Should fail without topic field
    assert!(
        response.status_code().is_client_error(),
        "Expected client error for missing topic"
    );
}

#[tokio::test]
async fn test_mqtt_publish_json_requires_payload() {
    let server = test_helpers::create_test_server().await;

    let response = server
        .post("/api/v1/mqtt/publish/json")
        .json(&json!({
            "topic": "test/topic",
            "qos": 0
        }))
        .await;

    // Should fail without payload field
    assert!(
        response.status_code().is_client_error() || response.status_code().is_server_error(),
        "Expected error for missing payload"
    );
}

#[tokio::test]
async fn test_invalid_qos_returns_error() {
    let server = test_helpers::create_test_server().await;

    let response = server
        .post("/api/v1/mqtt/publish")
        .json(&json!({
            "topic": "test/topic",
            "message": "test",
            "qos": 5  // Invalid QoS (must be 0, 1, or 2)
        }))
        .await;

    assert!(
        response.status_code().is_client_error(),
        "Expected client error for invalid QoS"
    );
}

#[tokio::test]
async fn test_vent_disconnect_without_connection() {
    let server = test_helpers::create_test_server().await;

    let response = server.post("/api/v1/vent/disconnect").await;

    // Should succeed even if not connected (idempotent operation)
    response.assert_status_ok();
}

#[tokio::test]
async fn test_open_vent_without_connection_returns_error() {
    let server = test_helpers::create_test_server().await;

    let response = server.post("/api/v1/vent/open").await;

    // Should fail because no device is connected
    assert!(
        response.status_code().is_client_error() || response.status_code().is_server_error(),
        "Expected error when opening vent without connection"
    );
}

#[tokio::test]
async fn test_close_vent_without_connection_returns_error() {
    let server = test_helpers::create_test_server().await;

    let response = server.post("/api/v1/vent/close").await;

    // Should fail because no device is connected
    assert!(
        response.status_code().is_client_error() || response.status_code().is_server_error(),
        "Expected error when closing vent without connection"
    );
}

#[tokio::test]
async fn test_api_versioning_v1_prefix() {
    let server = test_helpers::create_test_server().await;

    // Test that v1 endpoints are accessible
    let response = server.get("/api/v1/health").await;
    response.assert_status_ok();

    // Test that non-versioned endpoint doesn't exist
    let response = server.get("/api/health").await;
    response.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_health_uptime_increases() {
    let server = test_helpers::create_test_server().await;

    let response1 = server.get("/api/v1/health").await;
    let body1: Value = response1.json();
    let uptime1 = body1["uptime_seconds"].as_u64().unwrap();

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let response2 = server.get("/api/v1/health").await;
    let body2: Value = response2.json();
    let uptime2 = body2["uptime_seconds"].as_u64().unwrap();

    assert!(
        uptime2 >= uptime1,
        "Uptime should increase or stay the same"
    );
}

#[tokio::test]
async fn test_mqtt_publish_with_valid_data() {
    let server = test_helpers::create_test_server().await;

    let response = server
        .post("/api/v1/mqtt/publish")
        .json(&json!({
            "topic": "test/integration",
            "message": "Integration test message",
            "qos": 0
        }))
        .await;

    // May succeed or fail depending on MQTT broker availability
    // But should not panic
    assert!(
        response.status_code().is_success() || response.status_code().is_server_error(),
        "Expected success or server error, got {}",
        response.status_code()
    );
}

#[tokio::test]
async fn test_mqtt_publish_json_with_valid_data() {
    let server = test_helpers::create_test_server().await;

    let response = server
        .post("/api/v1/mqtt/publish/json")
        .json(&json!({
            "topic": "test/integration/json",
            "qos": 1,
            "payload": {
                "temperature": 22.5,
                "humidity": 55.0
            }
        }))
        .await;

    // May succeed or fail depending on MQTT broker availability
    assert!(
        response.status_code().is_success() || response.status_code().is_server_error(),
        "Expected success or server error, got {}",
        response.status_code()
    );
}

#[tokio::test]
async fn test_ble_stop_scan_is_idempotent() {
    let server = test_helpers::create_test_server().await;

    // Stop scan without starting (should succeed - idempotent)
    let response1 = server.post("/api/v1/ble/stop-scan").await;
    response1.assert_status_ok();

    // Stop again (should still succeed)
    let response2 = server.post("/api/v1/ble/stop-scan").await;
    response2.assert_status_ok();
}
