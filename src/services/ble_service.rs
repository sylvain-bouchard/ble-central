use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

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
        info!("Starting scan on {}...", adapter.adapter_info().await?);

        let (tx, rx) = mpsc::channel::<DeviceInfo>(32);
        let adapter = adapter.clone();

        tokio::spawn(async move {
            let mut events = match adapter.events().await {
                Ok(events) => events,
                Err(error) => {
                    error!("Failed to get adapter events: {:?}", error);
                    return;
                }
            };

            if let Err(error) = adapter.start_scan(ScanFilter::default()).await {
                error!("Failed to start BLE scan: {:?}", error);
                return;
            }

            while let Some(event) = events.next().await {
                match event {
                    btleplug::api::CentralEvent::DeviceDiscovered(id) => {
                        // Device discovered → extract info
                        if let Some((name, address)) = get_device_info(&adapter, &id).await {
                            let info = DeviceInfo {
                                id: id.to_string(),
                                name,
                                address,
                            };

                            if tx.send(info).await.is_err() {
                                error!("Receiver dropped; stopping scan task");
                                return;
                            }
                        }
                    }

                    btleplug::api::CentralEvent::ManufacturerDataAdvertisement {
                        id,
                        manufacturer_data,
                    } => {
                        info!("Manufacturer data from {id}: data={:?}", manufacturer_data);
                    }
                    _ => {}
                }
            }
        });

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

                    if let Err(e) = device.discover_services().await {
                        error!("Failed to discover GATT services: {:?}", e);
                        return Err(e);
                    }
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

async fn get_device_info(
    adapter: &Adapter,
    id: &PeripheralId,
) -> Option<(Option<String>, Option<String>)> {
    let peripherals = adapter.peripherals().await.ok()?;

    let peripheral = peripherals.iter().find(|p| p.id() == *id)?;
    let props = peripheral.properties().await.ok()??;

    let name = props.local_name;
    let addr = props.address.to_string();

    // macOS returns 00:00:00:00:00:00 — ignore that
    let address = if addr == "00:00:00:00:00:00" {
        None
    } else {
        Some(addr)
    };

    Some((name, address))
}
#[cfg(test)]
mod ble_service_tests {
    include!("ble_service_tests.rs");
}
