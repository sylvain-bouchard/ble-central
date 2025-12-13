use std::error::Error;

use api::vent::vent_routes::build_router;
use tracing_subscriber;

mod api;
mod application;
mod domain;
mod services;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing/logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let application =
        application::Application::new("192.168.0.36", 1883, "ble_to_mqtt_bridge", "0.0.0.0:8080")
            .await?;

    let address: std::net::SocketAddr = application.listen_address().parse().unwrap();

    let listener = tokio::net::TcpListener::bind(address).await?;
    let local_address = listener.local_addr()?;

    println!("\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    println!("🚀 Vent API running on {local_address}");
    println!("API docs are accessible at {local_address}/docs");

    axum::serve(listener, build_router(application.state().clone())).await?;

    Ok(())
}