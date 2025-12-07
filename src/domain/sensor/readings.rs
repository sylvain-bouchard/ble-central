#[derive(Debug, serde::Serialize)]
pub struct SensorReadings {
    pub co2: i16,
    pub temperature: f32,
    pub humidity: f32,
    pub voc_index: i16,
    pub pm1p0: f32,
    pub pm2p5: f32,
}

impl SensorReadings {
    /// Parse a manufacturer data payload from the ESP32-H2 BLE peripheral
    pub fn from_manufacturer_data(data: &[u8]) -> Option<Self> {
        // Ensure payload has at least 14 bytes (bytes 2..13)
        if data.len() < 14 {
            return None;
        }

        let co2 = i16::from_le_bytes([data[2], data[3]]);
        let temperature = i16::from_le_bytes([data[4], data[5]]) as f32 / 200.0;
        let humidity = i16::from_le_bytes([data[6], data[7]]) as f32 / 100.0;
        let voc_index = i16::from_le_bytes([data[8], data[9]]) / 10;
        let pm1p0 = u16::from_le_bytes([data[10], data[11]]) as f32 / 10.0;
        let pm2p5 = u16::from_le_bytes([data[12], data[13]]) as f32 / 10.0;

        Some(SensorReadings {
            co2,
            temperature,
            humidity,
            voc_index,
            pm1p0,
            pm2p5,
        })
    }
}
