mod models;
mod handlers;
mod utils;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() {
    let db_name = env::var("POSTGRES_DB").unwrap_or_else(|_| "cdn".to_string());
    let db_user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let db_password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
    
    let database_url = format!("postgresql://{}:{}@postgres:5432/{}", db_user, db_password, db_name);
    
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app = Router::new()
        .route("/upload", post(handlers::upload_document))
        .route("/documents/:id", get(handlers::get_document))
        .route("/documents", get(handlers::list_documents))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");

    println!("CDN server running on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}