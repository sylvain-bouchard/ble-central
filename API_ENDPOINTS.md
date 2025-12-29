# BLE Central Gateway - API Endpoints

## Overview

The BLE Central Gateway provides REST API endpoints for scanning, connecting to, and controlling BLE (Bluetooth Low Energy) devices, specifically vents.

## Base URL

```
http://localhost:8080
```

## API Versioning

All endpoints are versioned under `/api/v1`. This allows for backward compatibility when introducing breaking changes in future versions.

**Current Version:** v1

---

## Health Check

### Health Status

**Endpoint:** `GET /api/v1/health`

**Description:** Returns the health status of the application and its services. Useful for monitoring, load balancers, and orchestration systems.

**Response (Success - 200):**

```json
{
  "status": "ok",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "services": {
    "ble": "ok",
    "mqtt": "ok"
  }
}
```

**Service Status Values:**

- `ble`: `"ok"` (adapter available) or `"no_adapter"` (no BLE adapter found)
- `mqtt`: `"ok"` (service running and auto-reconnecting)

---

## BLE Operations

### 1. Start BLE Scan

**Endpoint:** `POST /api/v1/ble/scan`

**Description:** Initializes BLE scanning to discover nearby devices.

**Request Body:**

```json
{
  "timeout_secs": null // Optional: timeout duration (defaults to adapter timeout)
}
```

**Response (Success - 200):**

```json
{
  "status": "scanning",
  "message": "BLE scan started"
}
```

**Response (Error - 500):**

```json
{
  "error": "Failed to initialize scan: [error details]"
}
```

---

### 2. Connect to Device

**Endpoint:** `POST /api/v1/ble/connect/:device_id`

**Description:** Connects to a discovered BLE device by its ID.

**Path Parameters:**

- `device_id` (string, required): The unique identifier of the device to connect to

**Request Body:**

```json
{
  "device_id": "device_id_value",
  "timeout_secs": 10 // Optional: timeout for connection (defaults to 10 seconds)
}
```

**Response (Success - 200):**

```json
{
  "status": "connected",
  "device_id": "device_id_value",
  "message": "Successfully connected to device"
}
```

**Response (Error - 500):**

```json
{
  "error": "Failed to connect to device: [error details]"
}
```

---

## Vent Control Operations

### 3. Open Vent

**Endpoint:** `POST /api/v1/vent/open`

**Description:** Sends a command to the connected BLE device to open the vent.

**Request Body:** None

**Response (Success - 200):**

```json
{
  "status": "open",
  "message": "Vent opened successfully"
}
```

**Response (Error - 500):**

```json
{
  "error": "Failed to open vent: [error details]"
}
```

---

### 4. Close Vent

**Endpoint:** `POST /api/v1/vent/close`

**Description:** Sends a command to the connected BLE device to close the vent.

**Request Body:** None

**Response (Success - 200):**

```json
{
  "status": "closed",
  "message": "Vent closed successfully"
}
```

**Response (Error - 500):**

```json
{
  "error": "Failed to close vent: [error details]"
}
```

---

### 5. Get Vent Status

**Endpoint:** `GET /api/v1/vent/status`

**Description:** Retrieves the current status of the vent (Connected/Disconnected/Open/Closed).

**Response (Success - 200):**

```json
{
  "status": "Connected" // or "Disconnected", "Open", "Closed"
}
```

---

### 6. Disconnect from Device

**Endpoint:** `POST /api/v1/vent/disconnect`

**Description:** Disconnects from the currently connected BLE device.

**Request Body:** None

**Response (Success - 200):**

```json
{
  "status": "disconnected",
  "message": "Disconnected from device"
}
```

**Response (Error - 500):**

```json
{
  "error": "Failed to disconnect: [error details]"
}
```

---

## Workflow Example

### Typical usage flow:

1. **Start scanning for devices**

   ```bash
   curl -X POST http://localhost:8080/api/ble/scan \
     -H "Content-Type: application/json" \
     -d '{"timeout_secs": null}'
   ```

2. **Connect to a discovered device**

   ```bash
   curl -X POST http://localhost:8080/api/ble/connect/device_id_here \
     -H "Content-Type: application/json" \
     -d '{"device_id": "device_id_here", "timeout_secs": 10}'
   ```

3. **Check device status**

   ```bash
   curl http://localhost:8080/api/vent/status
   ```

4. **Open the vent**

   ```bash
   curl -X POST http://localhost:8080/api/vent/open
   ```

5. **Close the vent**

   ```bash
   curl -X POST http://localhost:8080/api/vent/close
   ```

6. **Disconnect from device**
   ```bash
   curl -X POST http://localhost:8080/api/vent/disconnect
   ```

---

## MQTT Integration

The gateway automatically publishes sensor data from BLE devices to an MQTT broker. Additionally, it provides API endpoints for MQTT operations.

### 7. Get MQTT Status

**Endpoint:** `GET /api/v1/mqtt/status`

**Description:** Check the status of the MQTT connection and service.

**Response (Success - 200):**

```json
{
  "status": "connected",
  "message": "MQTT service is active and monitoring sensor data"
}
```

---

### 8. Manually Publish Message

**Endpoint:** `POST /api/v1/mqtt/publish`

**Description:** Manually publish a text message to a specific MQTT topic.

**Request Body:**

```json
{
  "topic": "test/topic",
  "message": "Hello MQTT",
  "qos": 0
}
```

**QoS Levels:**

- `0`: At most once (fire and forget)
- `1`: At least once (acknowledged delivery)
- `2`: Exactly once (assured delivery)

**Response (Success - 200):**

```json
{
  "status": "published",
  "message": "Message published to topic 'test/topic'",
  "topic": "test/topic",
  "qos": 0
}
```

**Example:**

```bash
curl -X POST http://localhost:8080/api/mqtt/publish \
  -H "Content-Type: application/json" \
  -d '{"topic": "test/topic", "message": "Hello World", "qos": 1}'
```

---

### 9. Publish JSON Data

**Endpoint:** `POST /api/v1/mqtt/publish/json`

**Description:** Publish JSON-formatted data to a specific MQTT topic.

**Request Body:**

```json
{
  "topic": "sensor/data",
  "qos": 1,
  "payload": {
    "temperature": 25.5,
    "humidity": 60.0,
    "timestamp": "2025-12-27T12:00:00Z"
  }
}
```

**Response (Success - 200):**

```json
{
  "status": "published",
  "message": "JSON published to topic 'sensor/data'",
  "topic": "sensor/data",
  "qos": 1
}
```

**Example:**

```bash
curl -X POST http://localhost:8080/api/mqtt/publish/json \
  -H "Content-Type: application/json" \
  -d '{
    "topic": "living_room/air_quality/data",
    "qos": 1,
    "payload": {
      "temperature": 22.5,
      "humidity": 55.0,
      "co2": 450,
      "pm2_5": 12,
      "pm10": 18,
      "tvoc": 125
    }
  }'
```

---

## Automatic Sensor Data Publishing

The gateway automatically publishes sensor data from BLE devices with manufacturer ID `0xFFFF` to the MQTT topic `living_room/air_quality/data`.

**Published Data Format:**

```json
{
  "temperature": 25.8,
  "humidity": 77.2,
  "co2": 1286,
  "pm2_5": 1800,
  "pm10": 2314,
  "tvoc": 2828
}
```

This data is published whenever the gateway receives manufacturer data advertisements from connected BLE devices.

---

## Error Handling

All error responses follow this format:

```json
{
  "error": "Descriptive error message"
}
```

Common error scenarios:

- **No Bluetooth adapter found**: Ensure Bluetooth is enabled on your system
- **Device not found**: Verify the device ID is correct and within range
- **Device not connected**: Connect to a device before attempting control operations
- **Connection timeout**: The device took too long to respond; try again
- **BLE communication error**: Check device compatibility and characteristic UUIDs

---

## Notes

- The API uses **async/await** patterns for non-blocking operations
- All BLE operations have configurable timeouts (default 10 seconds)
- Device IDs are platform-specific and may vary across macOS, Linux, and Windows
- Characteristic UUIDs are currently placeholders; customize for your specific BLE device
- The scanning process runs in the background after initialization
- Multiple clients can query status simultaneously, but only one active connection is maintained

---

## Future Enhancements

- [x] MQTT service integration with automatic sensor data publishing
- [x] MQTT status and manual publish endpoints
- [ ] Subscribe to device discovery events via WebSocket
- [ ] Batch control operations on multiple devices
- [ ] Device pairing and security features
- [ ] RSSI (signal strength) monitoring
- [ ] Automatic reconnection on disconnection
- [ ] MQTT subscription endpoints (receive messages from topics)
- [ ] Configurable MQTT topics via API
