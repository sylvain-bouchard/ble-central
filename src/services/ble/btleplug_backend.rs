use async_trait::async_trait;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use futures::stream::StreamExt;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout, Duration};
use tracing::{debug, error, info};
use uuid::Uuid;

use super::ble_backend::{AdapterInfo, BleBackend, CharacteristicNotification, ManufacturerData};
use super::error::BleError;
use crate::services::ble::ble_backend::DeviceInfo;

pub struct BtleplugBackend {
    #[allow(dead_code)]
    manager: Manager,
    adapters: Vec<Adapter>,
}

impl BtleplugBackend {
    pub async fn new() -> Result<Self, BleError> {
        let manager = Manager::new().await.map_err(BleError::from)?;
        let adapters = manager.adapters().await.map_err(BleError::from)?;
        if adapters.is_empty() {
            return Err(BleError::NoAdapter);
        }
        Ok(Self { manager, adapters })
    }

    fn get_default_adapter(&self) -> Result<Adapter, BleError> {
        self.adapters.first().cloned().ok_or(BleError::NoAdapter)
    }

    async fn get_peripheral(&self, device_id: &str) -> Result<Peripheral, BleError> {
        let adapter = self.get_default_adapter()?;
        let peripherals = adapter.peripherals().await.map_err(BleError::from)?;
        peripherals
            .into_iter()
            .find(|p| p.id().to_string() == device_id)
            .ok_or(BleError::DeviceNotFound)
    }

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
}

#[async_trait]
impl BleBackend for BtleplugBackend {
    async fn list_adapters(&self) -> Result<Vec<AdapterInfo>, BleError> {
        let mut adapter_infos = Vec::new();

        for adapter in &self.adapters {
            let info = adapter.adapter_info().await.map_err(BleError::from)?;
            adapter_infos.push(AdapterInfo {
                id: info.to_string(),
                name: info.to_string(),
            });
        }

        Ok(adapter_infos)
    }

    async fn start_scan(
        &self,
    ) -> Result<(mpsc::Receiver<DeviceInfo>, mpsc::Receiver<ManufacturerData>), BleError> {
        let adapter = self.get_default_adapter()?;

        info!("Starting scan on {}...", adapter.adapter_info().await?);

        adapter
            .start_scan(ScanFilter::default())
            .await
            .map_err(BleError::from)?;

        let (tx_devices, rx_devices) = mpsc::channel(32);
        let (tx_manufacturer, rx_manufacturer) = mpsc::channel(32);
        let mut events = adapter.events().await.map_err(BleError::from)?;

        tokio::spawn(async move {
            while let Some(event) = events.next().await {
                match event {
                    btleplug::api::CentralEvent::DeviceDiscovered(id) => {
                        if let Some((name, address)) = get_device_info(&adapter, &id).await {
                            let info = DeviceInfo {
                                id: id.to_string(),
                                name,
                                address,
                            };
                            if tx_devices.send(info).await.is_err() {
                                error!("Receiver dropped; stopping scan task");
                                return;
                            }
                        }
                    }

                    btleplug::api::CentralEvent::ManufacturerDataAdvertisement {
                        id,
                        manufacturer_data,
                    } => {
                        debug!("Manufacturer data from {id}: data={:?}", manufacturer_data);
                        for (manufacturer_id, data) in manufacturer_data.iter() {
                            let msg = ManufacturerData {
                                device_id: id.to_string(),
                                manufacturer_id: *manufacturer_id,
                                data: data.clone(),
                            };
                            let _ = tx_manufacturer.send(msg).await;
                        }
                    }
                    _ => {}
                }
            }

            let _ = adapter.stop_scan().await;
        });

        Ok((rx_devices, rx_manufacturer))
    }

    async fn stop_scan(&self) -> Result<(), BleError> {
        let adapter = self.get_default_adapter()?;
        adapter.stop_scan().await.map_err(BleError::from)?;
        Ok(())
    }

    async fn connect(&self, device_id: &str, timeout_secs: u64) -> Result<(), BleError> {
        let timeout_dur = Duration::from_secs(timeout_secs);
        let _discovered = self.wait_for_device(device_id, timeout_dur).await?;

        let peripheral = self.get_peripheral(device_id).await?;
        peripheral.connect().await.map_err(BleError::from)?;
        peripheral
            .discover_services()
            .await
            .map_err(BleError::from)?;
        Ok(())
    }

    async fn disconnect(&self, device_id: &str) -> Result<(), BleError> {
        let peripheral = self.get_peripheral(device_id).await?;
        peripheral.disconnect().await.map_err(BleError::from)?;
        Ok(())
    }

    async fn read_characteristic(&self, device_id: &str, uuid: Uuid) -> Result<Vec<u8>, BleError> {
        let peripheral = self.get_peripheral(device_id).await?;
        let characteristic = find_characteristic(&peripheral, uuid)?;
        peripheral
            .read(&characteristic)
            .await
            .map_err(BleError::from)
    }

    async fn write_characteristic(
        &self,
        device_id: &str,
        uuid: Uuid,
        data: &[u8],
    ) -> Result<(), BleError> {
        let peripheral = self.get_peripheral(device_id).await?;
        let characteristic = find_characteristic(&peripheral, uuid)?;
        peripheral
            .write(
                &characteristic,
                data,
                btleplug::api::WriteType::WithResponse,
            )
            .await
            .map_err(BleError::from)?;
        Ok(())
    }

    async fn subscribe_to_characteristic(
        &self,
        device_id: &str,
        uuid: Uuid,
    ) -> Result<(), BleError> {
        let peripheral = self.get_peripheral(device_id).await?;
        let characteristic = find_characteristic(&peripheral, uuid)?;
        peripheral
            .subscribe(&characteristic)
            .await
            .map_err(BleError::from)?;
        Ok(())
    }

    async fn get_notifications(
        &self,
        device_id: &str,
    ) -> Result<mpsc::Receiver<CharacteristicNotification>, BleError> {
        let peripheral = self.get_peripheral(device_id).await?;
        let mut notifications = peripheral.notifications().await.map_err(BleError::from)?;

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(notification) = notifications.next().await {
                let notif = CharacteristicNotification {
                    uuid: notification.uuid,
                    value: notification.value,
                };
                if tx.send(notif).await.is_err() {
                    debug!("Notification receiver dropped");
                    break;
                }
            }
        });

        Ok(rx)
    }
}

impl From<btleplug::Error> for BleError {
    fn from(err: btleplug::Error) -> Self {
        use btleplug::Error::*;
        match err {
            DeviceNotFound => BleError::DeviceNotFound,
            TimedOut(..) => BleError::Timeout,
            _ => BleError::Backend(err.to_string()),
        }
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

fn find_characteristic(
    peripheral: &Peripheral,
    uuid: Uuid,
) -> Result<btleplug::api::Characteristic, BleError> {
    peripheral
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == uuid)
        .ok_or(BleError::CharacteristicNotFound)
}
