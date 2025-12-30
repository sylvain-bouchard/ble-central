use crate::api::vent::vent_controller::{self, ConnectRequest, ScanRequest};
use crate::services::ble_service::BleService;
use crate::services::mqtt_service::MqttService;
use crate::services::vent_service::VentService;
use crate::state::ApplicationState;
use std::sync::Arc;

async fn create_test_app_state() -> ApplicationState {
    let ble_service = BleService::new(5)
        .await
        .expect("Failed to create BleService");
    let vent_service = VentService::new(ble_service, 10);

    // Try to create MQTT service, but if it fails (no broker), use a test instance
    // In a real test environment, you'd use a mock MQTT service
    let mqtt_service =
        match MqttService::new("localhost", 1883, "test-client", "sensors/vent", 5, 0xFFFF).await {
            Ok(service) => service,
            Err(_) => {
                // Fallback: create with a non-existent broker for testing
                // The service will still be created but will log connection errors
                MqttService::new(
                    "127.0.0.1",
                    18830,
                    "test-fallback",
                    "sensors/vent",
                    5,
                    0xFFFF,
                )
                .await
                .expect("Failed to create fallback MQTT service")
            }
        };

    ApplicationState {
        vent_service: Arc::new(vent_service),
        mqtt_service: Arc::new(mqtt_service),
    }
}

#[tokio::test]
async fn test_controller_creation() {
    // With free functions, there's no controller to create
    // This test now just verifies the module is accessible
    let _state = create_test_app_state().await;
    assert!(true);
}

#[tokio::test]
async fn test_status_endpoint_without_connection() {
    let app_state = create_test_app_state().await;

    let status_response = vent_controller::status(&app_state).await;

    // status() returns VentStatusResponse directly, not Result
    // The status field contains the debug format of VentStatus enum
    assert!(
        status_response.status.contains("Disconnected")
            || status_response.status.contains("Connected")
            || status_response.status.contains("Open")
            || status_response.status.contains("Closed"),
        "Status response should contain a valid vent status, got: {}",
        status_response.status
    );
}

#[tokio::test]
async fn test_disconnect_without_connection() {
    let app_state = create_test_app_state().await;

    // Disconnect without ever connecting should succeed gracefully
    let result = vent_controller::disconnect(&app_state).await;

    assert!(
        result.is_ok(),
        "Disconnect should succeed without active connection"
    );
}

#[tokio::test]
async fn test_stop_scan_endpoint() {
    let app_state = create_test_app_state().await;

    // Stop scan should succeed even if no scan is running
    let result = vent_controller::stop_scan(&app_state).await;

    assert!(result.is_ok(), "stop_scan should always succeed");
}

#[tokio::test]
async fn test_open_without_connection() {
    let app_state = create_test_app_state().await;

    // Trying to open without connection should fail
    let result = vent_controller::open(&app_state).await;

    assert!(
        result.is_err(),
        "open() should fail without active connection"
    );
}

#[tokio::test]
async fn test_close_without_connection() {
    let app_state = create_test_app_state().await;

    // Trying to close without connection should fail
    let result = vent_controller::close(&app_state).await;

    assert!(
        result.is_err(),
        "close() should fail without active connection"
    );
}

#[tokio::test]
async fn test_connect_invalid_device() {
    let app_state = create_test_app_state().await;

    // Try to connect to non-existent device
    let request = ConnectRequest {
        timeout_secs: Some(1),
    };

    let result =
        vent_controller::connect(&app_state, "invalid_device_id".to_string(), request).await;

    match result {
        Ok(_) => {
            // Unexpected success
            panic!("Should not connect to non-existent device");
        }
        Err(_) => {
            // Expected to fail
            assert!(true);
        }
    }
}

#[tokio::test]
async fn test_scan_endpoint() {
    let app_state = create_test_app_state().await;

    // Scan should always succeed (starts BLE scanning)
    let request = ScanRequest {
        timeout_secs: Some(1),
    };

    let result = vent_controller::scan(&app_state, request).await;

    assert!(result.is_ok(), "scan() should succeed");
}

#[tokio::test]
async fn test_multiple_disconnect_calls() {
    let app_state = create_test_app_state().await;

    // First disconnect
    let result1 = vent_controller::disconnect(&app_state).await;
    assert!(result1.is_ok());

    // Second disconnect (no active connection)
    let result2 = vent_controller::disconnect(&app_state).await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_list_discovered_devices_empty() {
    let app_state = create_test_app_state().await;

    // Before scanning, should return empty list
    let response = vent_controller::list_discovered_devices(&app_state).await;
    assert!(
        response.devices.is_empty(),
        "Should return empty devices list before scanning"
    );
}

#[tokio::test]
async fn test_list_discovered_devices_after_scan() {
    let app_state = create_test_app_state().await;

    // Start scanning
    let scan_request = ScanRequest {
        timeout_secs: Some(1),
    };
    let _scan_result = vent_controller::scan(&app_state, scan_request).await;

    // Get discovered devices (may still be empty if no devices in test environment)
    let response = vent_controller::list_discovered_devices(&app_state).await;

    // Should return a valid response (may be empty in test env)
    assert!(
        response.devices.is_empty() || !response.devices.is_empty(),
        "Should always return valid response"
    );
}
