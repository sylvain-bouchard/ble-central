# BLE Central Gateway - API Endpoints

## Overview

The BLE Central Gateway provides REST API endpoints for scanning, connecting to, and controlling BLE (Bluetooth Low Energy) devices, specifically vents.

## Base URL

```
http://localhost:8080
```

---

## BLE Operations

### 1. Start BLE Scan

**Endpoint:** `POST /api/ble/scan`

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

**Endpoint:** `POST /api/ble/connect/:device_id`

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

**Endpoint:** `POST /api/vent/open`

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

**Endpoint:** `POST /api/vent/close`

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

**Endpoint:** `GET /api/vent/status`

**Description:** Retrieves the current status of the vent (Connected/Disconnected/Open/Closed).

**Response (Success - 200):**

```json
{
  "status": "Connected" // or "Disconnected", "Open", "Closed"
}
```

---

### 6. Disconnect from Device

**Endpoint:** `POST /api/vent/disconnect`

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

- [ ] Discover devices endpoint to list available devices during scan
- [ ] Subscribe to device discovery events via WebSocket
- [ ] Batch control operations on multiple devices
- [ ] Device pairing and security features
- [ ] RSSI (signal strength) monitoring
- [ ] Automatic reconnection on disconnection
