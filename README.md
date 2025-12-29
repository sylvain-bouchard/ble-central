# BLE Central Gateway

A Rust-based BLE (Bluetooth Low Energy) central gateway that provides a REST API for discovering, connecting to, and controlling BLE devices.

## Features

- BLE device discovery and scanning
- Device connection and management
- RESTful API for BLE operations
- Real-time status notifications via BLE characteristic subscriptions
- Single status characteristic for control and state management

## API Endpoints

- `POST /api/ble/scan` - Start BLE device scanning
- `GET /api/ble/devices` - List discovered devices
- `POST /api/ble/connect/{device_id}` - Connect to a device
- `POST /api/ble/stop-scan` - Stop scanning
- `GET /api/vent/status` - Get current status
- `POST /api/vent/open` - Send open command
- `POST /api/vent/close` - Send close command
- `POST /api/vent/disconnect` - Disconnect from device

## Building and Running

### Local Development

```bash
cargo build --release
cargo run
```

The API will be available at `http://localhost:8080`

### Docker

This application requires direct access to Bluetooth hardware via DBus. Docker support is available but requires:

1. The host must have DBus and Bluetooth support
2. The container must be run with `--privileged` and `--network host`
3. DBus sockets must be mounted from the host

#### Using Docker Compose (Recommended)

```bash
docker-compose up
```

#### Manual Docker Run

```bash
docker run --privileged --network host \
  -v /run/dbus:/run/dbus \
  -v /var/run/dbus:/var/run/dbus \
  -v /sys/class/bluetooth:/sys/class/bluetooth \
  ble-central-gateway
```

## Architecture

- **BleService**: Low-level BLE operations using btleplug
- **VentService**: Device-specific logic and state management
- **VentApiController**: REST API business logic
- **Notification Listener**: Background task that processes BLE notifications and updates status

## Protocol

The application uses a single BLE status characteristic (UUID: `0000180b-0000-1000-8000-00805f9b34fb`):

- Write `0x01` to open
- Write `0x02` to close
- Device sends notifications with current state

## Status Values

- `0x01` = Open
- `0x02` = Closed
- Other = Disconnected

## Testing

The project includes comprehensive test coverage:

### Run all tests

```bash
cargo test
```

### Test breakdown

- **Unit tests**: 35 tests (services + controllers)
- **Integration tests**: 17 tests (API endpoints)
- **Total**: 52 tests

### Integration tests

Integration tests validate API endpoints end-to-end using `axum-test`. They test:

- Health check endpoint
- BLE operations (scan, devices, stop-scan)
- Vent operations (status, open, close, disconnect)
- MQTT operations (status, publish, publish JSON)
- API versioning enforcement
- Error handling and validation

See [`tests/README.md`](tests/README.md) for detailed test documentation.

### Run only integration tests

```bash
cargo test --test integration_tests
```
