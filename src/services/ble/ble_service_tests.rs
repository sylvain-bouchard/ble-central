use super::mock_ble_backend::MockBleBackend;
use super::BleService;
use std::sync::Arc;

#[tokio::test]
async fn test_ble_service_creation() {
    let backend = Arc::new(MockBleBackend);
    let service = BleService::new(backend, 5);

    // Verify service was created
    let adapters = service
        .list_adapters()
        .await
        .expect("Failed to list adapters");
    assert_eq!(adapters.len(), 0); // Mock returns empty list
}

#[tokio::test]
async fn test_list_adapters() {
    let backend = Arc::new(MockBleBackend);
    let service = BleService::new(backend, 5);

    let adapters = service
        .list_adapters()
        .await
        .expect("Failed to list adapters");
    assert_eq!(adapters.len(), 0); // Mock returns empty list
}

#[tokio::test]
async fn test_ble_service_multiple_instances() {
    let backend1 = Arc::new(MockBleBackend);
    let service1 = BleService::new(backend1, 5);

    let backend2 = Arc::new(MockBleBackend);
    let service2 = BleService::new(backend2, 5);

    let adapters1 = service1
        .list_adapters()
        .await
        .expect("Failed to list adapters");
    let adapters2 = service2
        .list_adapters()
        .await
        .expect("Failed to list adapters");

    // Both should have same adapter count (0 for mock)
    assert_eq!(adapters1.len(), adapters2.len());
}

#[tokio::test]
async fn test_start_and_stop_scan() {
    let backend = Arc::new(MockBleBackend);
    let service = BleService::new(backend, 5);

    // Start scan
    let (_devices_rx, _manufacturer_rx) = service.start_scan().await.expect("Failed to start scan");

    // Stop scan
    service.stop_scan().await.expect("Failed to stop scan");
}

#[tokio::test]
async fn test_connect_and_disconnect() {
    let backend = Arc::new(MockBleBackend);
    let service = BleService::new(backend, 5);

    // Connect to device
    let device_id = service
        .connect_to_device("test-device-id", 5)
        .await
        .expect("Failed to connect");
    assert_eq!(device_id, "test-device-id");

    // Disconnect from device
    service
        .disconnect_device(&device_id)
        .await
        .expect("Failed to disconnect");
}

#[tokio::test]
async fn test_read_write_characteristic() {
    use uuid::Uuid;

    let backend = Arc::new(MockBleBackend);
    let service = BleService::new(backend, 5);

    let device_id = "test-device-id";
    let uuid = Uuid::parse_str("0000180b-0000-1000-8000-00805f9b34fb").unwrap();

    // Read characteristic
    let data = service
        .read_characteristic(device_id, uuid)
        .await
        .expect("Failed to read");
    assert_eq!(data.len(), 0); // Mock returns empty vec

    // Write characteristic
    service
        .write_characteristic(device_id, uuid, &[0x01, 0x02])
        .await
        .expect("Failed to write");
}

#[tokio::test]
async fn test_subscribe_and_get_notifications() {
    use uuid::Uuid;

    let backend = Arc::new(MockBleBackend);
    let service = BleService::new(backend, 5);

    let device_id = "test-device-id";
    let uuid = Uuid::parse_str("0000180b-0000-1000-8000-00805f9b34fb").unwrap();

    // Subscribe to characteristic
    service
        .subscribe_to_characteristic(device_id, uuid)
        .await
        .expect("Failed to subscribe");

    // Get notifications receiver
    let _notifications_rx = service
        .get_notifications(device_id)
        .await
        .expect("Failed to get notifications");
}

#[tokio::test]
async fn test_uuid_validation() {
    use uuid::Uuid;

    // Valid UUID should parse
    let valid_uuid = "0000180b-0000-1000-8000-00805f9b34fb";
    assert!(Uuid::parse_str(valid_uuid).is_ok());

    // Invalid UUIDs should fail
    let invalid_uuids = vec!["not-a-uuid", "0000180b", "0000180b-0000-1000-8000", ""];

    for invalid in invalid_uuids {
        assert!(
            Uuid::parse_str(invalid).is_err(),
            "Should reject invalid UUID: {}",
            invalid
        );
    }
}
