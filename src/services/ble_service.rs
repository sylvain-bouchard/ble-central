use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use serde::{Deserialize, Serialize};

use futures::stream::StreamExt;

use tokio::sync::mpsc;
use tokio::time::{sleep, timeout, Duration};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub address: Option<String>,
    pub name: Option<String>,
}

pub struct BleService {
    #[allow(dead_code)]
    manager: Manager,
    adapters: Vec<Adapter>,
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
        let (tx, rx) = mpsc::channel::<DeviceInfo>(32);
        let adapter_clone = adapter.clone();

        tokio::spawn(async move {
            if let Ok(mut events) = adapter_clone.events().await {
                adapter_clone.start_scan(ScanFilter::default()).await.ok();
                while let Some(event) = events.next().await {
                    match event {
                        btleplug::api::CentralEvent::DeviceDiscovered(id) => {
                            // Try to get device name and address from peripherals
                            let (device_name, device_address) = if let Ok(peripherals) =
                                adapter_clone.peripherals().await
                            {
                                let device_data = peripherals
                                    .iter()
                                    .find(|p| p.id().to_string() == id.to_string())
                                    .and_then(|p| {
                                        // Try to get properties (may be cached or from advertisement)
                                        if let Ok(Some(props)) =
                                            futures::executor::block_on(p.properties())
                                        {
                                            Some((props.local_name, props.address))
                                        } else {
                                            None
                                        }
                                    });

                                match device_data {
                                    Some((name, addr)) => {
                                        let addr_string = addr.to_string();
                                        // On macOS, address may be 00:00:00:00:00:00 (unavailable)
                                        // In that case, use the device ID as address since it's unique
                                        let final_address = if addr_string == "00:00:00:00:00:00" {
                                            None
                                        } else {
                                            Some(addr_string)
                                        };
                                        (name, final_address)
                                    }
                                    None => (None, None),
                                }
                            } else {
                                (None, None)
                            };

                            let device_info = DeviceInfo {
                                id: id.to_string(),
                                address: device_address,
                                name: device_name,
                            };

                            let _ = tx.send(device_info).await;
                        }
                        _ => {}
                    }
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
        timeout_dur: Duration,
    ) -> Result<Peripheral, btleplug::Error> {
        let poll = async {
            loop {
                for adapter in &self.adapters {
                    let peripherals = adapter.peripherals().await?;
                    for per in peripherals {
                        if per.id().to_string() == device_id {
                            return Ok::<Peripheral, btleplug::Error>(per);
                        }
                    }
                }
                sleep(Duration::from_millis(200)).await;
            }
        };

        match timeout(timeout_dur, poll).await {
            Ok(Ok(peripheral)) => Ok(peripheral),
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

        // Wait for device to be discovered
        let _discovered = self.wait_for_device(device_id, timeout_dur).await?;

        // Use connect_device to perform the actual connection
        self.connect_device(device_id).await
    }

    pub async fn disconnect_device(&self, device: &Peripheral) -> Result<(), btleplug::Error> {
        device.disconnect().await?;

        Ok(())
    }

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
        char_uuid: &str,
        data: &[u8],
    ) -> Result<(), btleplug::Error> {
        // Match characteristic by UUID string
        let chars = device.characteristics();

        for ch in chars {
            if ch.uuid.to_string() == char_uuid {
                device.write(&ch, data, WriteType::WithResponse).await?;
                return Ok(());
            }
        }
        Err(btleplug::Error::Other(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Characteristic not found",
        ))))
    }
}

#[cfg(test)]
mod ble_service_tests {
    include!("ble_service_tests.rs");
}
