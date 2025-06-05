use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::{Value, json};
use shared::api::{ApiEndpoints, get_api_base_url};
use shared::models::Message;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

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
    let api = ApiEndpoints::new(get_api_base_url());
    let messages_path = api.messages_endpoint().replace(get_api_base_url(), "");

    let app = Router::new()
        .route("/health", get(health_check))
        .route(&messages_path, post(receive_message))
        .layer(cors);

    // Define the address to run the server on
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap_or(8000);
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

// Handler to receive messages from the frontend and send to Anthropic
async fn receive_message(
    Json(message): Json<Message>,
) -> Result<Json<Message>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("Received message: {}", message.content);

    match send_to_anthropic(&message.content).await {
        Ok(response) => {
            tracing::info!("Anthropic response: {}", response);

            // Create a response message with Anthropic's reply
            let response_message = Message {
                content: response,
                timestamp: get_current_time(),
            };

            Ok(Json(response_message))
        }
        Err(e) => {
            tracing::error!("Failed to get response from Anthropic: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get response from Anthropic",
                    "details": e.to_string()
                })),
            ))
        }
    }
}

// Function to send message to Anthropic API and get response
async fn send_to_anthropic(user_message: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;

    tracing::info!(
        "API key loaded: {}...",
        &api_key[..std::cmp::min(10, api_key.len())]
    );

    let client = reqwest::Client::new();

    // Prepare headers
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("x-api-key", HeaderValue::from_str(&api_key)?);
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

    // Prepare the request body
    let request_body = json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": 1000,
        "messages": [
            {
                "role": "user",
                "content": user_message
            }
        ]
    });

    // Send request to Anthropic API
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        tracing::error!(
            "Anthropic API error - Status: {}, Body: {}",
            status,
            error_text
        );
        return Err(format!("Anthropic API error ({}): {}", status, error_text).into());
    }

    // Parse the response
    let response_json: Value = response.json().await?;

    // Extract the message content from the response
    let content = response_json
        .get("content")
        .and_then(|content| content.as_array())
        .and_then(|array| array.first())
        .and_then(|first| first.get("text"))
        .and_then(|text| text.as_str())
        .ok_or("Failed to extract message content from Anthropic response")?;

    Ok(content.to_string())
}

// Helper function to get current time (same as frontend)
fn get_current_time() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let total_seconds = duration.as_secs();

    let hours = (total_seconds / 3600) % 24;
    let minutes = (total_seconds / 60) % 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
