use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use serde_json::json;
use shared::models::Message; // Add the Message struct
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting unscrolled backend server");

    // Create a CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build our application with a route
    let app = Router::new()
        // Health check route
        .route("/health", get(health_check))
        // New route to handle message submissions
        .route("/api/messages", post(receive_message))
        .layer(cors);

    // Define the address to run the server on
    // Allow port to be set via environment variable or use default
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Starting server on {}", addr);

    // Run the server
    match TcpListener::bind(addr).await {
        Ok(listener) => {
            tracing::info!("Server started, listening on {}", addr);
            axum::serve(listener, app).await.unwrap();
        }
        Err(e) => {
            tracing::error!("Failed to bind to address {}: {}", addr, e);
            if e.kind() == std::io::ErrorKind::AddrInUse {
                tracing::error!(
                    "The port {} is already in use. Try setting a different port using the PORT environment variable.",
                    port
                );
            }
            std::process::exit(1);
        }
    }
}

// Health check handler
async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "message": "Unscrolled API is running",
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}

// New handler to receive messages from the frontend
async fn receive_message(Json(message): Json<Message>) -> StatusCode {
    tracing::info!("Received message: {}", message.content);
    StatusCode::OK
}
