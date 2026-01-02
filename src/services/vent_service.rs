use crate::domain::vent::vent::VentStatus;
use crate::error::AppError;
use crate::services::ble::{BleBackend, BleDataObserver, BleService};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};
use uuid::Uuid;

// BLE Characteristic UUIDs
const STATUS_UUID: &str = "0000180b-0000-1000-8000-00805f9b34fb"; // Status characteristic

// Vent command bytes
const VENT_CMD_OPEN: u8 = 0x01;
const VENT_CMD_CLOSE: u8 = 0x02;

// Compile-time validation that STATUS_UUID is a valid UUID
#[allow(dead_code)]
const fn validate_status_uuid() {
    // This will fail at compile time if STATUS_UUID has invalid format
    // Note: Uuid::parse_str is not const, so we validate at runtime in tests
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

pub struct VentService<B: BleBackend + 'static> {
    ble_service: BleService<B>,
    connected_device_id: Arc<Mutex<Option<String>>>,
    vent_status: Arc<RwLock<VentStatus>>,
    discovered_devices: Arc<Mutex<HashMap<String, DiscoveredDevice>>>,
    default_connection_timeout_secs: u64,
    status_uuid: Uuid,
}

impl<B: BleBackend + 'static> VentService<B> {
    pub fn new(ble_service: BleService<B>, default_connection_timeout_secs: u64) -> Self {
        let status_uuid = Uuid::parse_str(STATUS_UUID).expect("STATUS_UUID must be a valid UUID");

        VentService {
            ble_service,
            connected_device_id: Arc::new(Mutex::new(None)),
            vent_status: Arc::new(RwLock::new(VentStatus::Disconnected)),
            discovered_devices: Arc::new(Mutex::new(HashMap::new())),
            default_connection_timeout_secs,
            status_uuid,
        }
    }

    /// Register a BLE data observer with the internal BLE service
    pub async fn register_ble_observer(&self, observer: Arc<dyn BleDataObserver>) {
        self.ble_service.register_observer(observer).await;
    }

    /// Check if a BLE adapter is available
    pub async fn has_adapter(&self) -> bool {
        self.ble_service
            .list_adapters()
            .await
            .map(|adapters| !adapters.is_empty())
            .unwrap_or(false)
    }

    pub async fn initialize(&self) -> Result<(), AppError> {
        debug!("Initializing BLE scan");

        if !self.has_adapter().await {
            return Err(AppError::NoAdapter);
        }

        debug!("Starting BLE scan");
        let (mut device_rx, _manufacturer_rx) = self.ble_service.start_scan().await?;
        debug!("BLE scan started, got receiver channels");

        // Spawn a task to collect discovered devices
        let discovered_devices = Arc::clone(&self.discovered_devices);
        tokio::spawn(async move {
            debug!("Device collector task started");
            let mut device_count = 0;
            while let Some(device_info) = device_rx.recv().await {
                device_count += 1;
                debug!(
                    "Received device info #{}: id={}, name={:?}, addr={:?}",
                    device_count, device_info.id, device_info.name, device_info.address
                );

                let device_id = device_info.id.clone();

                let (was_added, total_count) = {
                    let mut devices = discovered_devices.lock().await;
                    let was_added = if !devices.contains_key(&device_id) {
                        devices.insert(
                            device_id.clone(),
                            DiscoveredDevice {
                                id: device_info.id.clone(),
                                address: device_info.address.clone(),
                                name: device_info.name.clone(),
                            },
                        );
                        true
                    } else {
                        false
                    };
                    (was_added, devices.len())
                };

                if was_added {
                    debug!("Added device to list (total: {})", total_count);
                } else {
                    debug!("Device {} already in list, skipping", device_id);
                }
            }
            debug!(
                "Device collector task ended (received {} devices total)",
                device_count
            );
        });

        debug!("Initialize completed");
        Ok(())
    }

    /// Connect to a vent device by waiting for discovery
    pub async fn connect(&self, device_id: &str, timeout_secs: u64) -> Result<(), AppError> {
        let connected_device_id = self
            .ble_service
            .connect_to_device(device_id, timeout_secs)
            .await?;

        // Subscribe to status characteristic notifications
        self.ble_service
            .subscribe_to_characteristic(&connected_device_id, self.status_uuid)
            .await?;

        // Spawn background task to listen for status notifications
        let vent_status_clone = Arc::clone(&self.vent_status);
        let device_id_clone = connected_device_id.clone();
        let ble_service_clone = self.ble_service.clone();
        let status_uuid = self.status_uuid;

        tokio::spawn(async move {
            if let Ok(mut notifications) =
                ble_service_clone.get_notifications(&device_id_clone).await
            {
                while let Some(notification) = notifications.recv().await {
                    // Check if this notification is from the status characteristic
                    if notification.uuid == status_uuid {
                        if !notification.value.is_empty() {
                            let status_byte = notification.value[0];
                            let new_status = match status_byte {
                                VENT_CMD_OPEN => VentStatus::Open,
                                VENT_CMD_CLOSE => VentStatus::Closed,
                                _ => VentStatus::Disconnected,
                            };
                            *vent_status_clone.write().await = new_status;
                        }
                    }
                }
            }
        });

        // Stop scanning after successfully connecting
        let _ = self.ble_service.stop_scan().await;

        *self.connected_device_id.lock().await = Some(connected_device_id);
        *self.vent_status.write().await = VentStatus::Connected;

        Ok(())
    }

    /// Open the vent by sending a command to the BLE device
    pub async fn open_vent(&self) -> Result<(), AppError> {
        let device_id = {
            let device = self.connected_device_id.lock().await;
            device.as_ref().ok_or(AppError::NotConnected)?.clone()
        };

        // Write "open" status as single byte: 0x01 to the status characteristic
        // Device will receive this and update its state
        // Status notifications will come back via the subscribed characteristic
        self.ble_service
            .write_characteristic(&device_id, self.status_uuid, &[VENT_CMD_OPEN])
            .await?;

        Ok(())
    }

    /// Close the vent by sending a command to the BLE device
    pub async fn close_vent(&self) -> Result<(), AppError> {
        let device_id = {
            let device = self.connected_device_id.lock().await;
            device.as_ref().ok_or(AppError::NotConnected)?.clone()
        };

        // Write "close" status as single byte: 0x02 to the status characteristic
        // Device will receive this and update its state
        // Status notifications will come back via the subscribed characteristic
        self.ble_service
            .write_characteristic(&device_id, self.status_uuid, &[VENT_CMD_CLOSE])
            .await?;

        Ok(())
    }

    /// Get the current vent status (from cached notifications)
    pub async fn get_vent_status(&self) -> VentStatus {
        // Return the cached status, which is updated by notifications
        *self.vent_status.read().await
    }

    /// Get the default connection timeout in seconds
    pub fn get_default_connection_timeout(&self) -> u64 {
        self.default_connection_timeout_secs
    }

    /// Disconnect from the device
    pub async fn disconnect(&self) -> Result<(), AppError> {
        let device_id_opt = self.connected_device_id.lock().await.take();

        if let Some(device_id) = device_id_opt {
            self.ble_service.disconnect_device(&device_id).await?;
        }

        // Also stop scanning if it's still active
        let _ = self.ble_service.stop_scan().await;

        *self.vent_status.write().await = VentStatus::Disconnected;

        Ok(())
    }

    /// Stop scanning (can be called manually to clean up resources)
    pub async fn stop_scanning(&self) -> Result<(), AppError> {
        self.ble_service.stop_scan().await?;
        Ok(())
    }

    /// Read the actual status from the device's status characteristic
    #[allow(dead_code)]
    async fn read_device_status_from_characteristic(
        &self,
        device_id: &str,
    ) -> Result<u8, AppError> {
        let data = self
            .ble_service
            .read_characteristic(device_id, self.status_uuid)
            .await?;

        // Device responds with a single byte: 0x01 = open, 0x02 = closed
        if data.is_empty() {
            return Err(AppError::Ble(btleplug::Error::Other(Box::new(
                std::io::Error::new(std::io::ErrorKind::Other, "Device returned empty response"),
            ))));
        }

        Ok(data[0])
    }

    /// Parse device status response (binary byte) and convert to VentStatus
    #[allow(dead_code)]
    pub fn parse_device_status_byte(&self, status_byte: u8) -> VentStatus {
        match status_byte {
            0x01 => VentStatus::Open,
            0x02 => VentStatus::Closed,
            _ => VentStatus::Disconnected,
        }
    }

    /// Get list of discovered devices
    pub async fn get_discovered_devices(&self) -> Vec<DiscoveredDevice> {
        let device_list: Vec<DiscoveredDevice> = {
            let devices = self.discovered_devices.lock().await;
            devices.values().cloned().collect()
        };

        info!(
            "get_discovered_devices called, returning {} devices",
            device_list.len()
        );
        for device in &device_list {
            debug!(
                "Device: id={}, name={:?}, address={:?}",
                device.id, device.name, device.address
            );
        }
        device_list
    }
}

#[cfg(test)]
mod vent_service_tests {
    include!("vent_service_tests.rs");
}
