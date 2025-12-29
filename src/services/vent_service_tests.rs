use super::{VentService, VentStatus, STATUS_UUID};
use crate::error::AppError;
use crate::services::ble_service::BleService;
use uuid::Uuid;

async fn create_test_vent_service() -> VentService {
    let ble_service = BleService::new(5)
        .await
        .expect("Failed to create BleService");
    VentService::new(ble_service, 10)
}

#[tokio::test]
async fn test_vent_service_creation() {
    let service = create_test_vent_service().await;

    // Verify initial status is Disconnected
    let status = service.get_vent_status().await;
    match status {
        VentStatus::Disconnected => assert!(true),
        _ => panic!("Initial status should be Disconnected"),
    }
}

#[tokio::test]
async fn test_vent_initial_status_is_disconnected() {
    let service = create_test_vent_service().await;
    let status = service.get_vent_status().await;

    assert_eq!(
        format!("{:?}", status),
        "Disconnected",
        "Initial status should be Disconnected"
    );
}

#[tokio::test]
async fn test_parse_device_status_open() {
    let service = create_test_vent_service().await;

    let status = service.parse_device_status_byte(0x01);
    assert_eq!(
        format!("{:?}", status),
        "Open",
        "Should parse 0x01 as Open status"
    );
}

#[tokio::test]
async fn test_parse_device_status_closed() {
    let service = create_test_vent_service().await;

    let status = service.parse_device_status_byte(0x02);
    assert_eq!(
        format!("{:?}", status),
        "Closed",
        "Should parse 0x02 as Closed status"
    );
}

#[tokio::test]
async fn test_parse_device_status_connected() {
    let service = create_test_vent_service().await;

    let status = service.parse_device_status_byte(0x03);
    assert_eq!(
        format!("{:?}", status),
        "Disconnected",
        "Should default to Disconnected for unknown byte values"
    );
}

#[tokio::test]
async fn test_parse_device_status_unknown() {
    let service = create_test_vent_service().await;

    let status = service.parse_device_status_byte(0xFF);
    assert_eq!(
        format!("{:?}", status),
        "Disconnected",
        "Should default to Disconnected for unknown status"
    );
}

#[tokio::test]
async fn test_connect_without_adapter_fails() {
    let service = create_test_vent_service().await;

    // This will fail because no adapters are available in test environment
    let result = service.connect("nonexistent_device", 1).await;

    match result {
        Err(AppError::NoAdapter) | Err(AppError::Ble(_)) => {
            // Expected to fail in test environment
            assert!(true);
        }
        Ok(_) => {
            // Should not succeed without real BLE adapter
            panic!("Should not connect without BLE adapter");
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn test_disconnect_without_connection_succeeds() {
    let service = create_test_vent_service().await;

    // Disconnect without ever connecting should succeed gracefully
    let result = service.disconnect().await;

    assert!(
        result.is_ok(),
        "Disconnect should succeed even without active connection"
    );
}

#[tokio::test]
async fn test_open_vent_without_connection_fails() {
    let service = create_test_vent_service().await;

    let result = service.open_vent().await;

    match result {
        Err(AppError::NotConnected) => assert!(true),
        _ => panic!("Should return NotConnected error when not connected"),
    }
}

#[tokio::test]
async fn test_close_vent_without_connection_fails() {
    let service = create_test_vent_service().await;

    let result = service.close_vent().await;

    match result {
        Err(AppError::NotConnected) => assert!(true),
        _ => panic!("Should return NotConnected error when not connected"),
    }
}

#[tokio::test]
async fn test_status_uuid_is_valid() {
    // Verify STATUS_UUID is a valid UUID
    assert!(
        Uuid::parse_str(STATUS_UUID).is_ok(),
        "STATUS_UUID '{}' should be a valid UUID",
        STATUS_UUID
    );
}
