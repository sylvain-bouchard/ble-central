use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use futures::stream::StreamExt;

use tokio::sync::mpsc;
use tokio::time::{sleep, timeout, Duration};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

pub struct BleService {
    #[allow(dead_code)]
    manager: Manager,
    adapters: Vec<Adapter>,
}

impl Clone for BleService {
    fn clone(&self) -> Self {
        BleService {
            manager: self.manager.clone(),
            adapters: self.adapters.clone(),
        }
    }
}

impl BleService {
    pub async fn new() -> Self {
        let manager = Manager::new().await.unwrap();
        let adapters = manager.adapters().await.unwrap();

        BleService { manager, adapters }
    }

    #[allow(dead_code)]
    pub fn list_adapters(&self) -> Vec<Adapter> {
        self.adapters.iter().cloned().collect()
    }

    pub fn get_default_adapter(&self) -> Option<Adapter> {
        self.adapters.first().cloned()
    }

    pub async fn start_scan(
        &self,
        adapter: &Adapter,
    ) -> Result<mpsc::Receiver<DeviceInfo>, btleplug::Error> {
        info!("Starting BLE scan");
        let (tx, rx) = mpsc::channel::<DeviceInfo>(32);
        let adapter_clone = adapter.clone();

        tokio::spawn(async move {
            debug!("Spawn task: Getting adapter events");
            match adapter_clone.events().await {
                Ok(mut events) => {
                    info!("Successfully created event stream");

                    // Start the scan
                    match adapter_clone.start_scan(ScanFilter::default()).await {
                        Ok(_) => info!("BLE scan started successfully"),
                        Err(e) => {
                            error!("Failed to start BLE scan: {:?}", e);
                            return;
                        }
                    }

                    debug!("Entering event loop");
                    while let Some(event) = events.next().await {
                        match event {
                            btleplug::api::CentralEvent::DeviceDiscovered(id) => {
                                debug!("Device discovered event: {}", id);

                                // Try to get device name and address from peripherals
                                let (device_name, device_address) = match adapter_clone
                                    .peripherals()
                                    .await
                                {
                                    Ok(peripherals) => {
                                        debug!("Retrieved {} peripherals", peripherals.len());
                                        let device_data = peripherals
                                            .iter()
                                            .find(|p| p.id().to_string() == id.to_string())
                                            .map(|p| p.clone());

                                        // If we found the peripheral, try to get its properties
                                        if let Some(peripheral) = device_data {
                                            debug!("Found matching peripheral for device {}", id);
                                            match peripheral.properties().await {
                                                Ok(Some(props)) => {
                                                    debug!("Retrieved properties for {}: name={:?}, addr={:?}", 
                                                           id, props.local_name, props.address);
                                                    Some((props.local_name, props.address))
                                                }
                                                Ok(None) => {
                                                    warn!("Properties returned None for device {}", id);
                                                    None
                                                }
                                                Err(e) => {
                                                    warn!("Failed to get properties for device {}: {:?}", id, e);
                                                    None
                                                }
                                            }
                                        } else {
                                            debug!("No matching peripheral found for device {}", id);
                                            None
                                        }
                                        .map(|(name, addr)| {
                                            let addr_string = addr.to_string();
                                            debug!(
                                                "Device {}: name={:?}, raw_addr={}",
                                                id, name, addr_string
                                            );

                                            // On macOS, address may be 00:00:00:00:00:00 (unavailable)
                                            let final_address = if addr_string
                                                == "00:00:00:00:00:00"
                                            {
                                                debug!("Address is invalid (00:00:00:00:00:00), setting to None");
                                                None
                                            } else {
                                                Some(addr_string)
                                            };
                                            (name, final_address)
                                        })
                                        .unwrap_or((None, None))
                                    }
                                    Err(e) => {
                                        error!("Failed to get peripherals: {:?}", e);
                                        (None, None)
                                    }
                                };

                                let device_info = DeviceInfo {
                                    id: id.to_string(),
                                    address: device_address.clone(),
                                    name: device_name.clone(),
                                };

                                info!(
                                    "Sending device info: id={}, name={:?}, address={:?}",
                                    device_info.id, device_info.name, device_info.address
                                );

                                if let Err(e) = tx.send(device_info).await {
                                    error!("Failed to send device info through channel: {:?}", e);
                                }
                            }
                            event => {
                                debug!("Received non-discovery event: {:?}", event);
                            }
                        }
                    }
                    info!("Event stream ended");
                }
                Err(e) => {
                    error!("Failed to get adapter events: {:?}", e);
                }
            }
        });

        debug!("scan_start returning receiver channel");
        Ok(rx)
    }

    pub async fn stop_scan(&self, adapter: &Adapter) -> Result<(), btleplug::Error> {
        adapter.stop_scan().await?;

        Ok(())
    }

    /// Wait for a device to be discovered with a timeout
    pub async fn wait_for_device(
        &self,
        device_id: &str,
        timeout_seconds: Duration,
    ) -> Result<Peripheral, btleplug::Error> {
        let device_id = device_id.to_string();

        let poll = async {
            loop {
                for adapter in &self.adapters {
                    let peripherals = adapter.peripherals().await?;

                    if let Some(per) = peripherals
                        .into_iter()
                        .find(|p| p.id().to_string() == device_id)
                    {
                        return Ok(per);
                    }
                }

                sleep(Duration::from_millis(200)).await;
            }
        };

        match timeout(timeout_seconds, poll).await {
            Ok(Ok(p)) => Ok(p),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(btleplug::Error::DeviceNotFound),
        }
    }

    pub async fn connect_device(&self, device_id: &str) -> Result<Peripheral, btleplug::Error> {
        for adapter in &self.adapters {
            let devices = adapter.peripherals().await?;
            for device in devices {
                if device.id().to_string() == device_id {
                    device.connect().await?;
                    return Ok(device);
                }
            }
        }
        Err(btleplug::Error::DeviceNotFound)
    }

    /// Wait for device discovery and connect with timeout
    pub async fn connect_to_device(
        &self,
        device_id: &str,
        timeout_secs: u64,
    ) -> Result<Peripheral, btleplug::Error> {
        let timeout_dur = Duration::from_secs(timeout_secs);

        let _discovered = self.wait_for_device(device_id, timeout_dur).await?;

        self.connect_device(device_id).await
    }

    pub async fn disconnect_device(&self, device: &Peripheral) -> Result<(), btleplug::Error> {
        device.disconnect().await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn read_characteristic(
        &self,
        device: &Peripheral,
        char_uuid: &str,
    ) -> Result<Vec<u8>, btleplug::Error> {
        let characteristics = device.characteristics();
        for characteristic in characteristics {
            if characteristic.uuid.to_string() == char_uuid {
                return device.read(&characteristic).await;
            }
        }
        Err(btleplug::Error::Other(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Characteristic not found",
        ))))
    }

    /// Write data to a characteristic by UUID string
    pub async fn write_characteristic(
        &self,
        device: &Peripheral,
        characteristic_uuid: &str,
        data: &[u8],
    ) -> Result<(), btleplug::Error> {
        // Match characteristic by UUID string
        let characteristics = device.characteristics();

        for characteristic in characteristics {
            if characteristic.uuid.to_string() == characteristic_uuid {
                device
                    .write(&characteristic, data, WriteType::WithResponse)
                    .await?;
                return Ok(());
            }
        }
        Err(btleplug::Error::Other(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Characteristic not found",
        ))))
    }

    /// Subscribe to notifications on a characteristic by UUID string
    pub async fn subscribe_to_characteristic(
        &self,
        device: &Peripheral,
        characteristic_uuid: &str,
    ) -> Result<(), btleplug::Error> {
        let characteristics = device.characteristics();

        for characteristic in characteristics {
            if characteristic.uuid.to_string() == characteristic_uuid {
                device.subscribe(&characteristic).await?;
                return Ok(());
            }
        }
        Err(btleplug::Error::Other(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Characteristic not found",
        ))))
    }

    /// Get the notification stream from a device
    pub async fn get_notifications(
        &self,
        device: &Peripheral,
    ) -> Result<futures::stream::BoxStream<'_, btleplug::api::ValueNotification>, btleplug::Error>
    {
        Ok(Box::pin(device.notifications().await?))
    }
}

#[cfg(test)]
mod ble_service_tests {
    include!("ble_service_tests.rs");
}
