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
    pub topic: String,
    pub manufacturer_id: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleConfig {
    pub enable_scan: bool,
    pub scan_auto_start: Option<bool>,
    pub connection_timeout_secs: u64,
    pub observer_timeout_secs: u64,
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

        let settings: Settings = builder
            // Load from environment variables with APP_ prefix
            // Supports nested keys like APP_MQTT__BROKER
            .add_source(
                Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("__"),
            )
            .build()?
            .try_deserialize()?;

        // Validate the loaded configuration
        settings.validate()?;

        Ok(settings)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate API configuration
        if self.api.port == 0 {
            return Err(ConfigError::Message(
                "API port must be greater than 0".to_string(),
            ));
        }

        if self.api.host.is_empty() {
            return Err(ConfigError::Message("API host cannot be empty".to_string()));
        }

        // Validate log level
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.api.log_level.to_lowercase().as_str()) {
            return Err(ConfigError::Message(format!(
                "Invalid log level '{}'. Must be one of: trace, debug, info, warn, error",
                self.api.log_level
            )));
        }

        // Validate MQTT configuration
        if self.mqtt.broker.is_empty() {
            return Err(ConfigError::Message(
                "MQTT broker cannot be empty".to_string(),
            ));
        }

        if self.mqtt.port == 0 {
            return Err(ConfigError::Message(
                "MQTT port must be greater than 0".to_string(),
            ));
        }

        if self.mqtt.client_id.is_empty() {
            return Err(ConfigError::Message(
                "MQTT client_id cannot be empty".to_string(),
            ));
        }

        if self.mqtt.topic.is_empty() {
            return Err(ConfigError::Message(
                "MQTT topic cannot be empty".to_string(),
            ));
        }

        if self.mqtt.keep_alive_secs == 0 {
            return Err(ConfigError::Message(
                "MQTT keep_alive_secs must be greater than 0".to_string(),
            ));
        }

        if self.mqtt.keep_alive_secs > 65535 {
            return Err(ConfigError::Message(
                "MQTT keep_alive_secs must be less than or equal to 65535".to_string(),
            ));
        }

        // manufacturer_id is u16, so it's already validated by type (0-65535)
        // No additional validation needed unless we want to restrict specific values

        // Validate BLE configuration
        if self.ble.connection_timeout_secs == 0 {
            return Err(ConfigError::Message(
                "BLE connection_timeout_secs must be greater than 0".to_string(),
            ));
        }

        if self.ble.connection_timeout_secs > 300 {
            return Err(ConfigError::Message(
                "BLE connection_timeout_secs should not exceed 300 seconds (5 minutes)".to_string(),
            ));
        }

        if self.ble.observer_timeout_secs == 0 {
            return Err(ConfigError::Message(
                "BLE observer_timeout_secs must be greater than 0".to_string(),
            ));
        }

        if self.ble.observer_timeout_secs > 60 {
            return Err(ConfigError::Message(
                "BLE observer_timeout_secs should not exceed 60 seconds".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the full listen address (host:port)
    pub fn listen_address(&self) -> String {
        format!("{}:{}", self.api.host, self.api.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_settings() -> Settings {
        Settings {
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
                topic: "sensors/vent".to_string(),
                manufacturer_id: 0xFFFF,
            },
            ble: BleConfig {
                enable_scan: true,
                scan_auto_start: Some(true),
                connection_timeout_secs: 10,
                observer_timeout_secs: 5,
            },
        }
    }

    #[test]
    fn test_listen_address_format() {
        let settings = create_valid_settings();
        assert_eq!(settings.listen_address(), "127.0.0.1:8080");
    }

    #[test]
    fn test_validation_valid_config() {
        let settings = create_valid_settings();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validation_api_port_zero() {
        let mut settings = create_valid_settings();
        settings.api.port = 0;
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("port must be greater than 0"));
    }

    #[test]
    fn test_validation_api_host_empty() {
        let mut settings = create_valid_settings();
        settings.api.host = String::new();
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("host cannot be empty"));
    }

    #[test]
    fn test_validation_log_level_invalid() {
        let mut settings = create_valid_settings();
        settings.api.log_level = "invalid".to_string();
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Must be one of"));
    }

    #[test]
    fn test_validation_log_level_case_insensitive() {
        let mut settings = create_valid_settings();
        settings.api.log_level = "INFO".to_string();
        assert!(settings.validate().is_ok());

        settings.api.log_level = "Error".to_string();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validation_mqtt_broker_empty() {
        let mut settings = create_valid_settings();
        settings.mqtt.broker = String::new();
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("broker cannot be empty"));
    }

    #[test]
    fn test_validation_mqtt_port_zero() {
        let mut settings = create_valid_settings();
        settings.mqtt.port = 0;
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("port must be greater than 0"));
    }

    #[test]
    fn test_validation_mqtt_client_id_empty() {
        let mut settings = create_valid_settings();
        settings.mqtt.client_id = String::new();
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("client_id cannot be empty"));
    }

    #[test]
    fn test_validation_mqtt_topic_empty() {
        let mut settings = create_valid_settings();
        settings.mqtt.topic = String::new();
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("topic cannot be empty"));
    }

    #[test]
    fn test_validation_mqtt_keep_alive_zero() {
        let mut settings = create_valid_settings();
        settings.mqtt.keep_alive_secs = 0;
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("keep_alive_secs must be greater than 0"));
    }

    #[test]
    fn test_validation_mqtt_keep_alive_too_large() {
        let mut settings = create_valid_settings();
        settings.mqtt.keep_alive_secs = 70000;
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("keep_alive_secs must be less than or equal to 65535"));
    }

    #[test]
    fn test_validation_ble_connection_timeout_zero() {
        let mut settings = create_valid_settings();
        settings.ble.connection_timeout_secs = 0;
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("connection_timeout_secs must be greater than 0"));
    }

    #[test]
    fn test_validation_ble_connection_timeout_too_large() {
        let mut settings = create_valid_settings();
        settings.ble.connection_timeout_secs = 400;
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("should not exceed 300 seconds"));
    }

    #[test]
    fn test_validation_ble_observer_timeout_zero() {
        let mut settings = create_valid_settings();
        settings.ble.observer_timeout_secs = 0;
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("observer_timeout_secs must be greater than 0"));
    }

    #[test]
    fn test_validation_ble_observer_timeout_too_large() {
        let mut settings = create_valid_settings();
        settings.ble.observer_timeout_secs = 100;
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("should not exceed 60 seconds"));
    }
}
