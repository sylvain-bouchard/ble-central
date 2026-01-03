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

pub trait BleBackend: Send + Sync + 'static {
    fn list_adapters(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<AdapterInfo>, BleError>> + Send;
    fn start_scan(
        &self,
    ) -> impl std::future::Future<
        Output = Result<(mpsc::Receiver<DeviceInfo>, mpsc::Receiver<ManufacturerData>), BleError>,
    > + Send;
    fn stop_scan(&self) -> impl std::future::Future<Output = Result<(), BleError>> + Send;

    fn connect(
        &self,
        device_id: &str,
        timeout_secs: u64,
    ) -> impl std::future::Future<Output = Result<(), BleError>> + Send;
    fn disconnect(
        &self,
        device_id: &str,
    ) -> impl std::future::Future<Output = Result<(), BleError>> + Send;
    fn read_characteristic(
        &self,
        device_id: &str,
        uuid: Uuid,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, BleError>> + Send;
    fn write_characteristic(
        &self,
        device_id: &str,
        uuid: Uuid,
        data: &[u8],
    ) -> impl std::future::Future<Output = Result<(), BleError>> + Send;
    fn subscribe_to_characteristic(
        &self,
        device_id: &str,
        uuid: Uuid,
    ) -> impl std::future::Future<Output = Result<(), BleError>> + Send;
    fn get_notifications(
        &self,
        device_id: &str,
    ) -> impl std::future::Future<
        Output = Result<mpsc::Receiver<CharacteristicNotification>, BleError>,
    > + Send;
}
