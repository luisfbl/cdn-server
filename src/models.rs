use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, sqlx::FromRow)]
pub struct DocumentRecord {
    pub id: uuid::Uuid,
    pub hash: String,
    pub file_path: String,
    pub file_size: i32,
    pub mime_type: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DocumentListItem {
    pub id: uuid::Uuid,
    pub hash: String,
    pub file_size: i32,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
}