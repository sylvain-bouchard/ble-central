# Configuration Framework

This application uses a hierarchical configuration system based on `config-rs` with environment-specific overrides.

## Configuration Priority (Highest to Lowest)

1. **Environment Variables** - Prefixed with `APP_` and using `__` as separator for nested keys
2. **Environment-specific Config Files** - Loaded from `configuration/{APP_ENV}.toml`
3. **Default Configuration** - Loaded from `configuration/default.toml`

## Configuration Structure

The application is configured through the following sections:

### API Configuration

```toml
[api]
host = "0.0.0.0"      # Listen address
port = 8080           # Listen port
log_level = "info"    # Tracing log level (debug, info, warn, error)
```

### MQTT Configuration

```toml
[mqtt]
broker = "localhost"           # MQTT broker hostname
port = 1883                    # MQTT broker port
client_id = "ble_to_mqtt_bridge"  # MQTT client identifier
keep_alive_secs = 5            # Keep-alive interval in seconds
```

### BLE Configuration

```toml
[ble]
enable_scan = true    # Enable BLE scanning on startup
```

## Configuration Files

### `configuration/default.toml`

Default values for all configuration options. Always loaded.

### `configuration/development.toml`

Development environment overrides. Loaded when `APP_ENV=development`.

### `configuration/staging.toml` (optional)

Staging environment overrides. Loaded when `APP_ENV=staging`.

### `configuration/production.toml` (optional)

Production environment overrides. Loaded when `APP_ENV=production`.

## Environment Variables

Override configuration using environment variables with the `APP_` prefix:

```bash
# Single-level keys
export APP_API__PORT=9000

# Nested keys using double underscores
export APP_MQTT__BROKER=mqtt.example.com
export APP_API__LOG_LEVEL=debug

# Set environment
export APP_ENV=production
```

### Common Environment Variables

| Variable                    | Type    | Example       | Description             |
| --------------------------- | ------- | ------------- | ----------------------- |
| `APP_ENV`                   | string  | `development` | Active environment name |
| `APP_API__HOST`             | string  | `0.0.0.0`     | API bind address        |
| `APP_API__PORT`             | integer | `8080`        | API port                |
| `APP_API__LOG_LEVEL`        | string  | `info`        | Log verbosity level     |
| `APP_MQTT__BROKER`          | string  | `localhost`   | MQTT broker host        |
| `APP_MQTT__PORT`            | integer | `1883`        | MQTT broker port        |
| `APP_MQTT__CLIENT_ID`       | string  | `bridge-1`    | MQTT client ID          |
| `APP_MQTT__KEEP_ALIVE_SECS` | integer | `5`           | MQTT keep-alive seconds |
| `APP_BLE__ENABLE_SCAN`      | boolean | `true`        | Enable BLE scanning     |

## Usage Examples

### Development Setup

```bash
# Uses configuration/development.toml
cargo run
```

### Production Setup

```bash
# Uses configuration/production.toml and environment overrides
APP_ENV=production APP_MQTT__BROKER=mqtt.prod.example.com cargo run
```

### Custom Configuration

```bash
# Override specific values via environment variables
APP_API__PORT=9000 \
APP_API__LOG_LEVEL=debug \
APP_MQTT__BROKER=mqtt.custom.com \
cargo run
```

## Adding New Configuration Values

1. Add the new field to the appropriate struct in `src/configuration.rs`
2. Add the default value to `configuration/default.toml`
3. Optionally override in environment-specific files
4. Access in code via the `Settings` struct

Example:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
    pub timeout_secs: u64,  // New field
}
```

Then in `configuration/default.toml`:

```toml
[api]
host = "0.0.0.0"
port = 8080
log_level = "info"
timeout_secs = 30  # New default
```

And use it:

```rust
let config = Settings::from_env()?;
let timeout = Duration::from_secs(config.api.timeout_secs);
```

## Validation

Configuration is validated at startup. If invalid values are provided (wrong types, missing required fields), the application will fail with a clear error message.

```bash
$ cargo run
thread 'main' panicked at 'Failed to load configuration: missing field `port`'
```
