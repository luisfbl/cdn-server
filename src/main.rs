mod handlers;
mod models;
mod utils;

use axum::{
    Router,
    response::Html,
    routing::{get, post},
};
use std::env;

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

    println!("Connecting to database...");
    let pool = match sqlx::PgPool::connect(&database_url).await {
        Ok(pool) => {
            println!("Database connection successful");
            pool
        }
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    println!("Running migrations...");
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Failed to run migrations: {}", e);
        std::process::exit(1);
    };
    println!("Migrations completed successfully");

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/documents", post(handlers::upload_document))
        .route("/documents", get(handlers::list_documents))
        .route("/documents/{id}", get(handlers::get_document))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");

    println!("CDN server running on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn serve_html() -> Html<String> {
    match tokio::fs::read_to_string("./static/index.html").await {
        Ok(html) => Html(html),
        Err(_) => Html("<h1>Erro ao carregar página</h1>".to_string())
    }
}
