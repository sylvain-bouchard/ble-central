use crate::api::vent::vent_controller::{ConnectRequest, ScanRequest, VentApiController};
use crate::services::ble_service::BleService;
use crate::services::vent_service::VentService;
use crate::state::ApplicationState;
use std::sync::Arc;

async fn create_test_app_state() -> ApplicationState {
    let ble_service = BleService::new().await;
    let vent_service = VentService::new(ble_service);

    ApplicationState {
        vent_service: Arc::new(vent_service),
        vent_api_controller: Arc::new(VentApiController::new()),
    }
}

async fn create_test_controller() -> VentApiController {
    VentApiController::new()
}

#[tokio::test]
async fn test_controller_creation() {
    let _controller = create_test_controller().await;
    // If we get here, controller was created successfully
    assert!(true);
}

#[tokio::test]
async fn test_status_endpoint_without_connection() {
    let controller = create_test_controller().await;
    let app_state = create_test_app_state().await;

    let status_response = controller.status(&app_state).await;

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
    let controller = create_test_controller().await;
    let app_state = create_test_app_state().await;

    // Disconnect without ever connecting should succeed gracefully
    let result = controller.disconnect(&app_state).await;

    assert!(
        result.is_ok(),
        "Disconnect should succeed without active connection"
    );
}

#[tokio::test]
async fn test_stop_scan_endpoint() {
    let controller = create_test_controller().await;
    let app_state = create_test_app_state().await;

    // Stop scan should succeed even if no scan is running
    let result = controller.stop_scan(&app_state).await;

    assert!(result.is_ok(), "stop_scan should always succeed");
}

#[tokio::test]
async fn test_open_without_connection() {
    let controller = create_test_controller().await;
    let app_state = create_test_app_state().await;

    // Trying to open without connection should fail
    let result = controller.open(&app_state).await;

    assert!(
        result.is_err(),
        "open() should fail without active connection"
    );
}

#[tokio::test]
async fn test_close_without_connection() {
    let controller = create_test_controller().await;
    let app_state = create_test_app_state().await;

    // Trying to close without connection should fail
    let result = controller.close(&app_state).await;

    assert!(
        result.is_err(),
        "close() should fail without active connection"
    );
}

#[tokio::test]
async fn test_connect_invalid_device() {
    let controller = create_test_controller().await;
    let app_state = create_test_app_state().await;

    // Try to connect to non-existent device
    let request = ConnectRequest {
        device_id: "invalid_device_id".to_string(),
        timeout_secs: Some(1),
    };

    let result = controller
        .connect(&app_state, "invalid_device_id".to_string(), request)
        .await;

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
    let controller = create_test_controller().await;
    let app_state = create_test_app_state().await;

    // Scan should always succeed (starts BLE scanning)
    let request = ScanRequest {
        timeout_secs: Some(1),
    };

    let result = controller.scan(&app_state, request).await;

    assert!(result.is_ok(), "scan() should succeed");
}

#[tokio::test]
async fn test_multiple_disconnect_calls() {
    let controller = create_test_controller().await;
    let app_state = create_test_app_state().await;

    // First disconnect
    let result1 = controller.disconnect(&app_state).await;
    assert!(result1.is_ok());

    // Second disconnect (no active connection)
    let result2 = controller.disconnect(&app_state).await;
    assert!(result2.is_ok());
}
