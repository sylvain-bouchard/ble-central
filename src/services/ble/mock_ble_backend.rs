use async_trait::async_trait;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::ble_backend::{
    AdapterInfo, BleBackend, CharacteristicNotification, DeviceInfo, ManufacturerData,
};
use super::error::BleError;

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MockBleBackend;

#[async_trait]
impl BleBackend for MockBleBackend {
    async fn list_adapters(&self) -> Result<Vec<AdapterInfo>, BleError> {
        Ok(vec![])
    }

    async fn start_scan(
        &self,
    ) -> Result<(mpsc::Receiver<DeviceInfo>, mpsc::Receiver<ManufacturerData>), BleError> {
        let (_tx1, rx1) = mpsc::channel(1);
        let (_tx2, rx2) = mpsc::channel(1);
        Ok((rx1, rx2))
    }

    async fn stop_scan(&self) -> Result<(), BleError> {
        Ok(())
    }

    async fn connect(&self, _device_id: &str, _timeout_secs: u64) -> Result<(), BleError> {
        Ok(())
    }

    async fn disconnect(&self, _device_id: &str) -> Result<(), BleError> {
        Ok(())
    }

    async fn read_characteristic(
        &self,
        _device_id: &str,
        _uuid: Uuid,
    ) -> Result<Vec<u8>, BleError> {
        Ok(vec![])
    }

    async fn write_characteristic(
        &self,
        _device_id: &str,
        _uuid: Uuid,
        _data: &[u8],
    ) -> Result<(), BleError> {
        Ok(())
    }

    async fn subscribe_to_characteristic(
        &self,
        _device_id: &str,
        _uuid: Uuid,
    ) -> Result<(), BleError> {
        Ok(())
    }

    async fn get_notifications(
        &self,
        _device_id: &str,
    ) -> Result<mpsc::Receiver<CharacteristicNotification>, BleError> {
        let (_tx, rx) = mpsc::channel(1);
        Ok(rx)
    }
}
