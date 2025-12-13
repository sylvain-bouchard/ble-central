use std::error::Error;

use api::vent::vent_routes::build_router;
use tracing_subscriber::EnvFilter;

mod api;
mod application;
mod configuration;
mod domain;
mod services;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration from files and environment variables
    let config = configuration::Settings::from_env()?;

    // Initialize tracing/logging with configured log level
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.api.log_level));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    tracing::info!("Configuration loaded: {:?}", config);
    tracing::info!(
        "Starting application with environment: {}",
        std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string())
    );

    // Create application with configuration
    let application = application::Application::new(
        &config.mqtt.broker,
        config.mqtt.port,
        &config.mqtt.client_id,
        &config.listen_address(),
    )
    .await?;

    let address: std::net::SocketAddr = application.listen_address().parse().unwrap();

    let listener = tokio::net::TcpListener::bind(address).await?;
    let local_address = listener.local_addr()?;

    println!("\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    println!("🚀 Vent API running on {local_address}");
    println!("API docs are accessible at {local_address}/docs");
    println!("Log level: {}", config.api.log_level);
    println!("MQTT broker: {}:{}", config.mqtt.broker, config.mqtt.port);
    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n");

    axum::serve(listener, build_router(application.state().clone())).await?;

    Ok(())
}
