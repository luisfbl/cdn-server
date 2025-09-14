mod handlers;
mod models;
mod utils;

use axum::{
    Json, Router,
    response::Html,
    routing::{get, post},
};
use serde_json::json;
use std::env;
use std::time::Duration;

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
    println!("Starting CDN backend...");

    let db_name = env::var("POSTGRES_DB").unwrap_or_else(|_| "cdn".to_string());
    let db_user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let db_password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
    let db_host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "postgres".to_string());

    let database_url = format!(
        "postgresql://{}:{}@{}:5432/{}",
        db_user, db_password, db_host, db_name
    );

    println!("Connecting to database at {}", database_url);
    let pool = loop {
        match sqlx::PgPool::connect(&database_url).await {
            Ok(pool) => {
                println!("✅ Database connection successful");
                break pool;
            }
            Err(e) => {
                eprintln!("❌ Failed to connect to database: {}", e);
                eprintln!("🔄 Retrying in 5 seconds...");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    };

    println!("🔄 Running database migrations...");
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => println!("✅ Migrations completed successfully"),
        Err(e) => {
            eprintln!("❌ Failed to run migrations: {}", e);
            eprintln!("💀 Exiting...");
            std::process::exit(1);
        }
    };

    let app = Router::new()
        .route("/api/documents", post(handlers::upload_document))
        .route("/api/documents", get(handlers::list_documents))
        .route("/api/documents/{id}", get(handlers::get_document))
        .route("/api/health", get(health_check))
        .with_state(pool);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(listener) => {
            println!("🚀 CDN server running on http://0.0.0.0:3000");
            listener
        }
        Err(e) => {
            eprintln!("❌ Failed to bind to port 3000: {}", e);
            std::process::exit(1);
        }
    };

    println!("🌟 Backend is ready to accept connections!");

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("❌ Server error: {}", e);
        std::process::exit(1);
    }
}
