use std::error::Error;
use std::sync::Arc;

use api::vent::vent_controller::VentApiController;
use api::vent::vent_routes::build_router;
use services::ble_service::BleService;
use services::vent_service::VentService;
use state::ApplicationState;
use tracing_subscriber;

mod api;
mod domain;
mod services;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing/logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let ble_service = BleService::new().await;
    let vent_service = Arc::new(VentService::new(ble_service));
    let vent_api_controller = Arc::new(VentApiController::new());

    let application_state = ApplicationState {
        vent_service,
        vent_api_controller,
    };

    let address: std::net::SocketAddr = "0.0.0.0:8080".parse().unwrap();

    let listener = tokio::net::TcpListener::bind(address).await?;
    let local_address = listener.local_addr()?;

    println!("\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    println!("🚀 Vent API running on {local_address}");
    println!("API docs are accessible at {local_address}/docs");

    axum::serve(listener, build_router(application_state)).await?;

    Ok(())
}
