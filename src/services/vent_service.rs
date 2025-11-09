use crate::domain::vent::vent::VentStatus;
use crate::services::ble_service::BleService;
use btleplug::platform::{Adapter, Peripheral};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// BLE Characteristic UUIDs
const CONTROL_UUID: &str = "0000180a-0000-1000-8000-00805f9b34fb"; // Control characteristic
const STATUS_UUID: &str = "0000180b-0000-1000-8000-00805f9b34fb"; // Status characteristic (read responses)

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub id: String,
    pub address: Option<String>,
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

    pub async fn initialize(&self) -> Result<(), VentError> {
        let adapter = self
            .ble_service
            .get_default_adapter()
            .ok_or(VentError::NoAdapter)?;

        let rx = self.ble_service.start_scan(&adapter).await?;

        *self.scanning_adapter.lock().await = Some(adapter);

        // Spawn a task to collect discovered devices with names from advertisement
        let discovered_devices = Arc::clone(&self.discovered_devices);
        tokio::spawn(async move {
            let mut rx = rx;
            while let Some(device_info) = rx.recv().await {
                let mut devices = discovered_devices.lock().await;

                // Check if device already exists
                if !devices.iter().any(|d| d.id == device_info.id) {
                    // Device info already includes name and address from advertisement
                    devices.push(DiscoveredDevice {
                        id: device_info.id,
                        address: device_info.address,
                        name: device_info.name,
                    });
                }
            }
        });

        Ok(())
    }

    /// Connect to a vent device by waiting for discovery
    pub async fn connect(&self, device_id: &str, timeout_secs: u64) -> Result<(), VentError> {
        let peripheral = self
            .ble_service
            .connect_to_device(device_id, timeout_secs)
            .await?;

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

        if device.is_none() {
            return Err(VentError::NotConnected);
        }

        let peripheral = device.as_ref().unwrap();

        // Send "open" command
        self.ble_service
            .write_characteristic(peripheral, CONTROL_UUID, b"open")
            .await?;

        // Verify by reading device status
        let status = self
            .read_device_status_from_characteristic(peripheral)
            .await?;

        // Update local status based on device response
        if status.to_lowercase().contains("open") {
            *self.vent_status.lock().await = VentStatus::Open;
            Ok(())
        } else {
            Err(VentError::Ble(btleplug::Error::Other(Box::new(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Device did not confirm open status. Response: {}", status),
                ),
            ))))
        }
    }

    /// Close the vent by sending a command to the BLE device
    pub async fn close_vent(&self) -> Result<(), VentError> {
        let device = self.connected_device.lock().await;

        if device.is_none() {
            return Err(VentError::NotConnected);
        }

        let peripheral = device.as_ref().unwrap();

        // Send "close" command
        self.ble_service
            .write_characteristic(peripheral, CONTROL_UUID, b"close")
            .await?;

        // Verify by reading device status
        let status = self
            .read_device_status_from_characteristic(peripheral)
            .await?;

        // Update local status based on device response
        if status.to_lowercase().contains("closed") {
            *self.vent_status.lock().await = VentStatus::Closed;
            Ok(())
        } else {
            Err(VentError::Ble(btleplug::Error::Other(Box::new(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Device did not confirm closed status. Response: {}", status),
                ),
            ))))
        }
    }

    /// Get the current vent status (reads from device if connected)
    pub async fn get_vent_status(&self) -> VentStatus {
        // If device is connected, try to read the actual status
        if let Some(device) = self.connected_device.lock().await.as_ref() {
            if let Ok(status) = self.read_device_status_from_characteristic(device).await {
                return self.parse_device_status(&status);
            }
        }
        // Fall back to local status if reading fails
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
    async fn read_device_status_from_characteristic(
        &self,
        peripheral: &Peripheral,
    ) -> Result<String, VentError> {
        let data = self
            .ble_service
            .read_characteristic(peripheral, STATUS_UUID)
            .await?;

        Ok(String::from_utf8_lossy(&data).to_string())
    }

    /// Parse device status response and convert to VentStatus
    pub fn parse_device_status(&self, response: &str) -> VentStatus {
        let lower = response.to_lowercase();
        if lower.contains("open") {
            VentStatus::Open
        } else if lower.contains("closed") || lower.contains("close") {
            VentStatus::Closed
        } else if lower.contains("connected") {
            VentStatus::Connected
        } else {
            VentStatus::Disconnected
        }
    }

    /// Get list of discovered devices
    pub async fn get_discovered_devices(&self) -> Vec<DiscoveredDevice> {
        self.discovered_devices.lock().await.clone()
    }
}

#[cfg(test)]
mod vent_service_tests {
    include!("vent_service_tests.rs");
}
