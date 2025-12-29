# Integration Tests

This directory contains integration tests for the BLE Central Gateway API.

## Overview

Integration tests validate the API endpoints end-to-end, ensuring that:

- Routes are properly configured and accessible
- Request/response formats are correct
- Error handling works as expected
- Services integrate correctly
- API versioning is enforced

## Running Tests

### Run all tests (unit + integration)

```bash
cargo test
```

### Run only integration tests

```bash
cargo test --test integration_tests
```

### Run a specific integration test

```bash
cargo test --test integration_tests test_health_endpoint_returns_ok
```

### Run with output

```bash
cargo test --test integration_tests -- --nocapture
```

## Test Coverage

### Health Check (2 tests)

- ✅ Health endpoint returns OK status
- ✅ Health endpoint contains version information
- ✅ Health endpoint tracks uptime correctly

### BLE Operations (4 tests)

- ✅ BLE devices endpoint returns empty list initially
- ✅ BLE scan accepts valid requests
- ✅ BLE stop-scan is idempotent
- ✅ Discover devices endpoint format validation

### Vent Operations (4 tests)

- ✅ Vent status returns current status
- ✅ Vent disconnect without connection succeeds (idempotent)
- ✅ Opening vent without connection returns error
- ✅ Closing vent without connection returns error

### MQTT Operations (5 tests)

- ✅ MQTT status returns connected state
- ✅ MQTT publish requires topic field
- ✅ MQTT publish JSON requires payload field
- ✅ Invalid QoS value returns client error
- ✅ MQTT publish with valid data processes correctly

### API Versioning (1 test)

- ✅ v1 prefix is required for all endpoints
- ✅ Non-versioned endpoints return 404

## Test Environment

Integration tests create a test server using `axum_test::TestServer` with:

- Minimal BLE service configuration
- Mock MQTT service (handles connection failures gracefully)
- Test application state with proper service initialization

## Dependencies

The integration tests use:

- `axum-test`: HTTP testing framework for Axum applications
- `serde_json`: JSON serialization/deserialization for API testing
- `tokio`: Async runtime for test execution

## Notes

- Tests are designed to work without external dependencies (BLE adapters, MQTT brokers)
- MQTT connection failures are handled gracefully
- BLE adapter availability is checked but not required for tests to pass
- Tests validate both success and error scenarios
- All tests are isolated and can run in parallel

## Test Count

**Total**: 52 tests (35 unit + 17 integration)

- Unit tests: 35 (services + controllers)
- Integration tests: 17 (API endpoints)
- Doc tests: 0

## CI/CD Integration

These tests are suitable for:

- Continuous Integration pipelines
- Pre-commit hooks
- Automated deployment verification
- API contract testing
