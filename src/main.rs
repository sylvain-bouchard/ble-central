use std::error::Error;
use std::sync::Arc;

use api::vent::vent_routes::build_router;
use domain::vent::controller::VentController;
use state::ApplicationState;

mod api;
mod domain;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let vent = Arc::new(VentController::new());
    let application_state = ApplicationState { vent };

    // give the compiler an explicit type for FromStr
    let address: std::net::SocketAddr = "0.0.0.0:8080".parse().unwrap();

    let listener = tokio::net::TcpListener::bind(address).await?;
    let local_address = listener.local_addr()?;

    println!("\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    println!("🚀 Vent API running on {local_address}");
    println!("API docs are accessible at {local_address}/docs");

    axum::serve(listener, build_router(application_state)).await?;

    Ok(())
}
