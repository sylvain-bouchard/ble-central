use api::vent::vent_routes::build_router;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod api;
mod application;
mod configuration;
mod domain;
mod error;
mod services;
mod state;

use error::AppError;

/// BLE Central Gateway - Converts BLE sensor data to MQTT messages
///
/// ## API Versioning
///
/// All API endpoints are versioned under `/api/v1` to allow for backward
/// compatibility when introducing breaking changes in future versions.
/// This follows REST API best practices and enables smooth migrations.
#[derive(Parser, Debug)]
#[command(name = "ble-central-gateway")]
#[command(about = "BLE to MQTT Bridge Gateway", long_about = None)]
struct Args {
    /// Path to configuration file (overrides default locations)
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = Args::parse();
    let configuration = configuration::Settings::from_env(args.config.as_deref())?;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&configuration.api.log_level));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    tracing::info!("Configuration loaded: {:?}", configuration);
    tracing::info!(
        "Starting application with environment: {}",
        std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string())
    );

    // Initialize health check start time
    api::health::health_controller::init_start_time();

    let application =
        application::Application::new(&configuration, &configuration.listen_address()).await?;

    if configuration.ble.scan_auto_start.unwrap_or(false) {
        tracing::info!("Starting BLE scan as per configuration");
        application.state.vent_service.initialize().await?;
    } else {
        tracing::info!("BLE scan auto-start is disabled in configuration");
    }

    let address: std::net::SocketAddr = application
        .listen_address()
        .parse()
        .map_err(|e| format!("Invalid listen address: {}", e))?;

    let listener = tokio::net::TcpListener::bind(address).await?;
    let local_address = listener.local_addr()?;

    println!("\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    println!("🚀 Vent API running on {local_address}");
    println!("API docs are accessible at {local_address}/docs");
    println!("Log level: {}", configuration.api.log_level);
    println!(
        "MQTT broker: {}:{} Topic: {}",
        configuration.mqtt.broker, configuration.mqtt.port, configuration.mqtt.topic
    );

    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n");

    // Create shutdown signal handler
    let shutdown_signal = async {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!("Received Ctrl+C signal");
            },
            _ = terminate => {
                tracing::info!("Received SIGTERM signal");
            },
        }
    };

    // Run the server with graceful shutdown
    let server = axum::serve(listener, build_router(application.state().clone()))
        .with_graceful_shutdown(shutdown_signal);

    tracing::info!("Server is running, press Ctrl+C to stop");

    if let Err(e) = server.await {
        tracing::error!("Server error: {}", e);
    }

    // Perform application cleanup
    application.shutdown().await;

    tracing::info!("Application stopped");

    Ok(())
}
