use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct DocumentRecord {
    pub id: uuid::Uuid,
    pub hash: String,
    pub file_path: String,
    pub file_size: i32,
    pub mime_type: String,
}

#[derive(Debug, FromRow)]
pub struct DocumentDescriptionRecord {
    pub id: uuid::Uuid,
    pub document_id: uuid::Uuid,
    pub description: String,
}

#[derive(Debug)]
pub struct DocumentWithDescriptions {
    pub id: uuid::Uuid,
    pub hash: String,
    pub file_size: i32,
    pub mime_type: String,
    pub descriptions: Vec<String>,
}