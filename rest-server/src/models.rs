use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DocumentListItem {
    pub id: String,
    pub hash: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
}