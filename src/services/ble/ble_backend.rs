use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::error::BleError;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AdapterInfo {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ManufacturerData {
    pub device_id: String,
    pub manufacturer_id: u16,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct CharacteristicNotification {
    pub uuid: Uuid,
    pub value: Vec<u8>,
}

#[async_trait]
pub trait BleBackend: Send + Sync + 'static {
    async fn list_adapters(&self) -> Result<Vec<AdapterInfo>, BleError>;
    async fn start_scan(
        &self,
    ) -> Result<(mpsc::Receiver<DeviceInfo>, mpsc::Receiver<ManufacturerData>), BleError>;
    async fn stop_scan(&self) -> Result<(), BleError>;

    async fn connect(&self, device_id: &str, timeout_secs: u64) -> Result<(), BleError>;
    async fn disconnect(&self, device_id: &str) -> Result<(), BleError>;
    async fn read_characteristic(&self, device_id: &str, uuid: Uuid) -> Result<Vec<u8>, BleError>;
    async fn write_characteristic(
        &self,
        device_id: &str,
        uuid: Uuid,
        data: &[u8],
    ) -> Result<(), BleError>;
    async fn subscribe_to_characteristic(
        &self,
        device_id: &str,
        uuid: Uuid,
    ) -> Result<(), BleError>;
    async fn get_notifications(
        &self,
        device_id: &str,
    ) -> Result<mpsc::Receiver<CharacteristicNotification>, BleError>;
}
