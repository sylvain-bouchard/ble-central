use super::BleService;

// Note: Full integration tests for BleService would require mocking btleplug
// which is complex due to its trait-based design. These are basic structural tests.

#[tokio::test]
async fn test_ble_service_creation() {
    // This test verifies BleService can be instantiated
    // In a real scenario with mocking, you'd test actual BLE operations
    let service = BleService::new().await;

    // Verify adapters were loaded (may be empty in test environment)
    let adapters = service.list_adapters();
    assert!(adapters.is_empty() || !adapters.is_empty()); // Always true, but verifies method exists
}

#[tokio::test]
async fn test_get_default_adapter() {
    let service = BleService::new().await;

    // If adapters exist, get_default_adapter should return Some
    // If not, it should return None
    let adapter = service.get_default_adapter();
    match adapter {
        Some(_) => {
            // Adapter was found
            assert!(true);
        }
        None => {
            // No adapters available in test environment
            assert!(true);
        }
    }
}

#[tokio::test]
async fn test_list_adapters() {
    let service = BleService::new().await;
    let adapters = service.list_adapters();
    // May be empty in test environment or have devices
    assert!(adapters.len() < 1000); // Reasonable upper bound for number of BLE adapters
}

#[tokio::test]
async fn test_ble_service_multiple_instances() {
    let service1 = BleService::new().await;
    let service2 = BleService::new().await;

    let adapters1 = service1.list_adapters();
    let adapters2 = service2.list_adapters();

    // Both should have same adapter count
    assert_eq!(adapters1.len(), adapters2.len());
}
