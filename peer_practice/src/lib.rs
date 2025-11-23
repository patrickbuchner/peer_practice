use axum::Router;
use axum::http::{HeaderValue, Method};
use axum::routing::{get, post};
use eyre::{Context, Result};
use std::net::SocketAddr;
use std::path::Path;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::input::config::current::Config;
use app_state::AppState;
use handler::login;
use handler::websocket;

mod app_state;
mod handler;
pub mod input;
mod services;

async fn run(config: Config) -> Result<()> {
    // Ensure data directory exists and initialize logging to a file within it
    std::fs::create_dir_all(&config.server.data_dir).with_context(|| {
        format!(
            "Failed to create data directory at {}",
            config.server.data_dir.display()
        )
    })?;

    let _logging_guard = init_file_logging(&config.server.data_dir)?;
    let state = AppState::new(config.clone());

    tokio::spawn(services::run_expired_posts_reaper(
        state.clone(),
        chrono::Duration::hours(1),
    ));

    info!(
        "Serving static files from: {}",
        &config.server.webroot.clone().unwrap().display()
    );

    let serve_dir =
        ServeDir::new(config.clone().server.webroot.unwrap()).not_found_service(ServeFile::new(
            format!("{}/index.html", config.server.webroot.unwrap().display()),
        ));
    let cors_origin = config
        .server
        .cors_allowed_origins
        .iter()
        .map(HeaderValue::try_from)
        .map(|i| i.expect("Invalid CORS origin"))
        .collect::<Vec<_>>();
    info!("CORS allowed origins: {:?}", cors_origin);

    let app = Router::new()
        .fallback_service(serve_dir)
        .route("/v1/pin", post(login::pin_handler))
        .route("/v1/login", post(login::login_handler))
        .route("/v1/ws", get(websocket::ws_handler))
        .with_state(state)
        .layer(CorsLayer::new().allow_origin(cors_origin).allow_methods([
            Method::POST,
            Method::GET,
            Method::OPTIONS,
        ]));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    info!("Server listening on https://{addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .await
        .context("Failed to start server")?;
    Ok(())
}

fn init_file_logging(data_dir: &Path) -> Result<WorkerGuard> {
    // Respect RUST_LOG if set, otherwise default to info
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Create a daily rolling file appender in the data directory
    let file_appender = tracing_appender::rolling::daily(data_dir, "server.log");
    let (nb_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Build a JSON formatting layer that writes to the file
    let json_file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .json()
        .with_writer(nb_writer);

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(json_file_layer);

    #[cfg(debug_assertions)]
    let subscriber = {
        let stdout_layer = tracing_subscriber::fmt::layer();
        subscriber.with(stdout_layer)
    };

    subscriber.init();
    Ok(guard)
}
