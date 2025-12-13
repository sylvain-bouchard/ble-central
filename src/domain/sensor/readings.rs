#[derive(Clone, Debug, serde::Serialize)]
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
    /// Expects exactly 14 bytes of data starting at offset 2 (indices 0-11 in this slice):
    ///
    /// Manufacturer data format (little endian):
    /// - bytes 0-1: co2 (i16, raw ppm value)
    /// - bytes 2-3: temperature (i16, divide by 200.0 to get °C)
    /// - bytes 4-5: humidity (i16, divide by 100.0 to get %)
    /// - bytes 6-7: voc_index (i16, divide by 10 to get index)
    /// - bytes 8-9: pm1p0 (u16, divide by 10.0 to get µg/m³)
    /// - bytes 10-11: pm2p5 (u16, divide by 10.0 to get µg/m³)
    pub fn from_manufacturer_data(data: &[u8]) -> Option<Self> {
        // Must be exactly 12 bytes (indices 0-11 from stripped manufacturer ID header)
        if data.len() != 12 {
            return None;
        }

        let co2 = i16::from_le_bytes([data[0], data[1]]);
        let temperature = i16::from_le_bytes([data[2], data[3]]) as f32 / 200.0;
        let humidity = i16::from_le_bytes([data[4], data[5]]) as f32 / 100.0;
        let voc_index = i16::from_le_bytes([data[6], data[7]]) / 10;
        let pm1p0 = u16::from_le_bytes([data[8], data[9]]) as f32 / 10.0;
        let pm2p5 = u16::from_le_bytes([data[10], data[11]]) as f32 / 10.0;

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
