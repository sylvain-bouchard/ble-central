use std::error::Error;

use api::vent::vent_routes::build_router;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod api;
mod application;
mod configuration;
mod domain;
mod services;
mod state;

/// BLE Central Gateway - Converts BLE sensor data to MQTT messages
#[derive(Parser, Debug)]
#[command(name = "ble-central-gateway")]
#[command(about = "BLE to MQTT Bridge Gateway", long_about = None)]
struct Args {
    /// Path to configuration file (overrides default locations)
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    // Load configuration from files and environment variables
    let configuration = configuration::Settings::from_env(args.config.as_deref())?;

    // Initialize tracing/logging with configured log level
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&configuration.api.log_level));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    tracing::info!("Configuration loaded: {:?}", configuration);
    tracing::info!(
        "Starting application with environment: {}",
        std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string())
    );

    // Create application with configuration
    let application = application::Application::new(
        &configuration.mqtt.broker,
        configuration.mqtt.port,
        &configuration.mqtt.client_id,
        &configuration.listen_address(),
    )
    .await?;

    let address: std::net::SocketAddr = application.listen_address().parse().unwrap();

    let listener = tokio::net::TcpListener::bind(address).await?;
    let local_address = listener.local_addr()?;

    println!("\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    println!("🚀 Vent API running on {local_address}");
    println!("API docs are accessible at {local_address}/docs");
    println!("Log level: {}", configuration.api.log_level);
    println!("MQTT broker: {}:{}", configuration.mqtt.broker, configuration.mqtt.port);
    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n");

    axum::serve(listener, build_router(application.state().clone())).await?;

    Ok(())
}
