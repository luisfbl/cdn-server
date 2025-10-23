mod handlers;
mod models;
mod utils;
mod aws_client;

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde_json::json;
use std::sync::Arc;

async fn health_check() -> Json<serde_json::Value> {
    let container_id = std::fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    Json(json!({
        "status": "healthy",
        "container_id": container_id,
        "service": "backend"
    }))
}

#[tokio::main]
async fn main() {
    let aws_clients = Arc::new(aws_client::AwsClients::new().await);

    let app = Router::new()
        .route("/api/documents", post(handlers::upload_document))
        .route("/api/documents", get(handlers::list_documents))
        .route("/api/documents/{id}", get(handlers::get_document))
        .route("/api/health", get(health_check))
        .with_state(aws_clients);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(listener) => {
            println!("CDN server running on http://0.0.0.0:3000");
            listener
        }
        Err(e) => {
            eprintln!("Failed to bind to port 3000: {}", e);
            std::process::exit(1);
        }
    };

    println!("Backend is ready to accept connections!");

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("❌ Server error: {}", e);
        std::process::exit(1);
    }
}
