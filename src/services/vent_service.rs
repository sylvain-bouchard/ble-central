use crate::domain::vent::vent::VentStatus;
use crate::services::ble_service::{BleDataObserver, BleService};
use btleplug::platform::{Adapter, Peripheral};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

// BLE Characteristic UUIDs
const STATUS_UUID: &str = "0000180b-0000-1000-8000-00805f9b34fb"; // Status characteristic

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

pub struct VentService {
    ble_service: BleService,
    scanning_adapter: Arc<Mutex<Option<Adapter>>>,
    connected_device: Arc<Mutex<Option<Peripheral>>>,
    vent_status: Arc<Mutex<VentStatus>>,
    discovered_devices: Arc<Mutex<Vec<DiscoveredDevice>>>,
}

#[derive(thiserror::Error, Debug)]
pub enum VentError {
    #[error("No Bluetooth adapter found")]
    NoAdapter,
    #[error("Device not connected")]
    NotConnected,
    #[error("BLE error: {0}")]
    Ble(#[from] btleplug::Error),
}

impl VentService {
    pub fn new(ble_service: BleService) -> Self {
        VentService {
            ble_service,
            scanning_adapter: Arc::new(Mutex::new(None)),
            connected_device: Arc::new(Mutex::new(None)),
            vent_status: Arc::new(Mutex::new(VentStatus::Disconnected)),
            discovered_devices: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a BLE data observer with the internal BLE service
    pub async fn register_ble_observer(&self, observer: Arc<dyn BleDataObserver>) {
        self.ble_service.register_observer(observer).await;
    }

    pub async fn initialize(&self) -> Result<(), VentError> {
        info!("Initializing BLE scan");
        let adapter = self
            .ble_service
            .get_default_adapter()
            .ok_or(VentError::NoAdapter)?;

        info!("Starting BLE scan with adapter");
        let rx = self.ble_service.start_scan(&adapter).await?;
        info!("BLE scan started, got receiver channel");

        *self.scanning_adapter.lock().await = Some(adapter);

        // Spawn a task to collect discovered devices with names from advertisement
        let discovered_devices = Arc::clone(&self.discovered_devices);
        tokio::spawn(async move {
            info!("Device collector task started");
            let mut rx = rx;
            let mut device_count = 0;
            while let Some(device_info) = rx.recv().await {
                device_count += 1;
                debug!(
                    "Received device info #{}: id={}, name={:?}, addr={:?}",
                    device_count, device_info.id, device_info.name, device_info.address
                );

                let mut devices = discovered_devices.lock().await;

                // Check if device already exists
                if !devices.iter().any(|d| d.id == device_info.id) {
                    // Device info already includes name and address from advertisement
                    devices.push(DiscoveredDevice {
                        id: device_info.id.clone(),
                        address: device_info.address.clone(),
                        name: device_info.name.clone(),
                    });
                    info!("Added device to list (total: {})", devices.len());
                } else {
                    debug!("Device {} already in list, skipping", device_info.id);
                }
            }
            info!(
                "Device collector task ended (received {} devices total)",
                device_count
            );
        });

        info!("Initialize completed");
        Ok(())
    }

    /// Connect to a vent device by waiting for discovery
    pub async fn connect(&self, device_id: &str, timeout_secs: u64) -> Result<(), VentError> {
        let peripheral = self
            .ble_service
            .connect_to_device(device_id, timeout_secs)
            .await?;

        // Subscribe to status characteristic notifications
        self.ble_service
            .subscribe_to_characteristic(&peripheral, STATUS_UUID)
            .await?;

        // Spawn background task to listen for status notifications
        let vent_status_clone = Arc::clone(&self.vent_status);
        let peripheral_clone = peripheral.clone();
        let ble_service_clone = self.ble_service.clone();
        let status_uuid = STATUS_UUID.to_string();

        tokio::spawn(async move {
            if let Ok(mut notifications) =
                ble_service_clone.get_notifications(&peripheral_clone).await
            {
                while let Some(notification) = notifications.next().await {
                    // Check if this notification is from the status characteristic
                    if notification.uuid.to_string() == status_uuid {
                        if !notification.value.is_empty() {
                            let status_byte = notification.value[0];
                            let new_status = match status_byte {
                                0x01 => VentStatus::Open,
                                0x02 => VentStatus::Closed,
                                _ => VentStatus::Disconnected,
                            };
                            *vent_status_clone.lock().await = new_status;
                        }
                    }
                }
            }
        });

        // Stop scanning after successfully connecting
        if let Some(adapter) = self.scanning_adapter.lock().await.take() {
            let _ = self.ble_service.stop_scan(&adapter).await;
        }

        *self.connected_device.lock().await = Some(peripheral);
        *self.vent_status.lock().await = VentStatus::Connected;

        Ok(())
    }

    /// Open the vent by sending a command to the BLE device
    pub async fn open_vent(&self) -> Result<(), VentError> {
        let device = self.connected_device.lock().await;

        let peripheral = device.as_ref().ok_or(VentError::NotConnected)?;

        // Write "open" status as single byte: 0x01 to the status characteristic
        // Device will receive this and update its state
        // Status notifications will come back via the subscribed characteristic
        self.ble_service
            .write_characteristic(peripheral, STATUS_UUID, &[0x01])
            .await?;

        Ok(())
    }

    /// Close the vent by sending a command to the BLE device
    pub async fn close_vent(&self) -> Result<(), VentError> {
        let device = self.connected_device.lock().await;

        let peripheral = device.as_ref().ok_or(VentError::NotConnected)?;

        // Write "close" status as single byte: 0x02 to the status characteristic
        // Device will receive this and update its state
        // Status notifications will come back via the subscribed characteristic
        self.ble_service
            .write_characteristic(peripheral, STATUS_UUID, &[0x02])
            .await?;

        Ok(())
    }

    /// Get the current vent status (from cached notifications)
    pub async fn get_vent_status(&self) -> VentStatus {
        // Return the cached status, which is updated by notifications
        self.vent_status.lock().await.clone()
    }

    /// Disconnect from the device
    pub async fn disconnect(&self) -> Result<(), VentError> {
        let mut device = self.connected_device.lock().await;

        if let Some(peripheral) = device.take() {
            self.ble_service.disconnect_device(&peripheral).await?;
        }

        // Also stop scanning if it's still active
        if let Some(adapter) = self.scanning_adapter.lock().await.take() {
            let _ = self.ble_service.stop_scan(&adapter).await;
        }

        *self.vent_status.lock().await = VentStatus::Disconnected;

        Ok(())
    }

    /// Stop scanning (can be called manually to clean up resources)
    pub async fn stop_scanning(&self) -> Result<(), VentError> {
        if let Some(adapter) = self.scanning_adapter.lock().await.take() {
            self.ble_service.stop_scan(&adapter).await?;
        }
        Ok(())
    }

    /// Read the actual status from the device's status characteristic
    #[allow(dead_code)]
    async fn read_device_status_from_characteristic(
        &self,
        peripheral: &Peripheral,
    ) -> Result<u8, VentError> {
        let data = self
            .ble_service
            .read_characteristic(peripheral, STATUS_UUID)
            .await?;

        // Device responds with a single byte: 0x01 = open, 0x02 = closed
        if data.is_empty() {
            return Err(VentError::Ble(btleplug::Error::Other(Box::new(
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
        let devices = self.discovered_devices.lock().await.clone();
        info!(
            "get_discovered_devices called, returning {} devices",
            devices.len()
        );
        for device in &devices {
            debug!(
                "Device: id={}, name={:?}, address={:?}",
                device.id, device.name, device.address
            );
        }
        devices
    }
}

#[cfg(test)]
mod vent_service_tests {
    include!("vent_service_tests.rs");
}
