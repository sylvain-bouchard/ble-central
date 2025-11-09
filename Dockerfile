# Build stage
FROM rust:1.91-slim-bullseye as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libdbus-1-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release


# Production stage
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    libdbus-1-3 \
    dbus \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/ble-central-gateway .

EXPOSE 8080

# Note: To run this container with Bluetooth access, use:
# docker run --privileged --network host \
#   -v /run/dbus:/run/dbus \
#   -v /var/run/dbus:/var/run/dbus \
#   ble-central-gateway

CMD ["./ble-central-gateway"]