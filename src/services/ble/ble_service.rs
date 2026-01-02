use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use tokio::sync::mpsc;
use tokio::sync::RwLock;

use super::error::BleError;
use super::{BleBackend, CharacteristicNotification, DeviceInfo, ManufacturerData};

#[async_trait]
pub trait BleDataObserver: Send + Sync {
    #[allow(dead_code)]
    async fn on_sensor_data(&self, id: String, manufacturer_id: u16, data: Arc<[u8]>);
}

pub struct BleService<B: BleBackend> {
    backend: Arc<B>,
    observers: Arc<RwLock<Vec<Arc<dyn BleDataObserver>>>>,
    observer_timeout_secs: u64,
}

impl<B: BleBackend> Clone for BleService<B> {
    fn clone(&self) -> Self {
        Self {
            backend: Arc::clone(&self.backend),
            observers: Arc::clone(&self.observers),
            observer_timeout_secs: self.observer_timeout_secs,
        }
    }
}

impl<B: BleBackend> BleService<B> {
    pub fn new(backend: Arc<B>, observer_timeout_secs: u64) -> Self {
        Self {
            backend,
            observers: Arc::new(RwLock::new(Vec::new())),
            observer_timeout_secs,
        }
    }

    pub async fn list_adapters(&self) -> Result<Vec<String>, BleError> {
        let adapters = self.backend.list_adapters().await?;
        Ok(adapters.into_iter().map(|a| a.name).collect())
    }

    pub async fn start_scan(
        &self,
    ) -> Result<(mpsc::Receiver<DeviceInfo>, mpsc::Receiver<ManufacturerData>), BleError> {
        self.backend.start_scan().await
    }

    pub async fn stop_scan(&self) -> Result<(), BleError> {
        self.backend.stop_scan().await
    }

    pub async fn connect_to_device(
        &self,
        device_id: &str,
        timeout_secs: u64,
    ) -> Result<String, BleError> {
        self.backend.connect(device_id, timeout_secs).await?;
        Ok(device_id.to_string())
    }

    pub async fn disconnect_device(&self, device_id: &str) -> Result<(), BleError> {
        self.backend.disconnect(device_id).await
    }

    pub async fn read_characteristic(
        &self,
        device_id: &str,
        uuid: Uuid,
    ) -> Result<Vec<u8>, BleError> {
        self.backend.read_characteristic(device_id, uuid).await
    }

    pub async fn write_characteristic(
        &self,
        device_id: &str,
        uuid: Uuid,
        data: &[u8],
    ) -> Result<(), BleError> {
        self.backend
            .write_characteristic(device_id, uuid, data)
            .await
    }

    pub async fn subscribe_to_characteristic(
        &self,
        device_id: &str,
        uuid: Uuid,
    ) -> Result<(), BleError> {
        self.backend
            .subscribe_to_characteristic(device_id, uuid)
            .await
    }

    pub async fn get_notifications(
        &self,
        device_id: &str,
    ) -> Result<mpsc::Receiver<CharacteristicNotification>, BleError> {
        self.backend.get_notifications(device_id).await
    }

    /// Register a data observer
    pub async fn register_observer(&self, observer: Arc<dyn BleDataObserver>) {
        self.observers.write().await.push(observer);
    }
}
