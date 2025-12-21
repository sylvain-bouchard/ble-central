use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

/// Application configuration loaded from files and environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub api: ApiConfig,
    pub mqtt: MqttConfig,
    pub ble: BleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker: String,
    pub port: u16,
    pub client_id: String,
    pub keep_alive_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleConfig {
    pub enable_scan: bool,
    pub scan_auto_start: Option<bool>,
}

impl Settings {
    /// Load configuration from files and environment variables
    ///
    /// Priority order (highest to lowest):
    /// 1. Environment variables (with APP_ prefix)
    /// 2. Custom config file (if provided via config_path)
    /// 3. Environment-specific config file (e.g., development.toml)
    /// 4. Default configuration file (default.toml)
    pub fn from_env(config_path: Option<&str>) -> Result<Self, ConfigError> {
        let env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let config_dir = "configuration";

        let mut builder = Config::builder()
            // Load default configuration
            .add_source(File::with_name(&format!("{}/default", config_dir)))
            // Load environment-specific config if it exists
            .add_source(File::with_name(&format!("{}/{}", config_dir, env)).required(false));

        // Load custom config file if provided
        if let Some(path) = config_path {
            let file_content = std::fs::read_to_string(path).map_err(|e| {
                ConfigError::Message(format!("Failed to read config file '{}': {}", path, e))
            })?;
            builder = builder
                .add_source(File::from_str(&file_content, config::FileFormat::Toml).required(true));
        }

        let settings = builder
            // Load from environment variables with APP_ prefix
            // Supports nested keys like APP_MQTT__BROKER
            .add_source(
                Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("__"),
            )
            .build()?
            .try_deserialize()?;

        Ok(settings)
    }

    /// Get the full listen address (host:port)
    pub fn listen_address(&self) -> String {
        format!("{}:{}", self.api.host, self.api.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listen_address_format() {
        let settings = Settings {
            api: ApiConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                log_level: "debug".to_string(),
            },
            mqtt: MqttConfig {
                broker: "localhost".to_string(),
                port: 1883,
                client_id: "test".to_string(),
                keep_alive_secs: 5,
            },
            ble: BleConfig { enable_scan: true, scan_auto_start: Some(true)},
        };

        assert_eq!(settings.listen_address(), "127.0.0.1:8080");
    }
}
