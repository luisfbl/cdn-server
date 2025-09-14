use crate::models::{DocumentListItem, DocumentRecord};
use crate::utils::calculate_hash;
use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::Response,
};
use sqlx::PgPool;
use std::collections::HashMap;
use std::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn upload_document(
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut file_data = Vec::new();
    let mut description = String::new();
    let mut filename = String::new();
    let mut content_type = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                filename = field.file_name().unwrap_or("unknown").to_string();
                content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                file_data = field
                    .bytes()
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?
                    .to_vec();
            }
            "description" => {
                description = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            _ => {}
        }
    }

    if file_data.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let hash = calculate_hash(&file_data);

    let existing_document = sqlx::query_as::<_, DocumentRecord>(
        "SELECT id, hash, file_path, file_size, mime_type FROM documents WHERE hash = $1",
    )
    .bind(hash.clone())
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let document_id = if let Some(existing) = existing_document {
        existing.id
    } else {
        let storage_dir = "/var/cdn/storage";
        fs::create_dir_all(storage_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let file_extension = std::path::Path::new(&filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");

        let file_path = format!("{storage_dir}/{hash}.{file_extension}");

        let mut file = File::create(&file_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.write_all(&file_data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        sqlx::query_scalar("INSERT INTO documents (hash, file_path, file_size, mime_type) VALUES ($1, $2, $3, $4) RETURNING id")
            .bind(hash.clone())
            .bind(file_path)
            .bind(file_data.len() as i32)
            .bind(content_type)
            .fetch_one(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    if !description.is_empty() {
        sqlx::query("INSERT INTO document_descriptions (document_id, description) VALUES ($1, $2)")
            .bind(document_id)
            .bind(description)
            .execute(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let mut response = HashMap::new();
    response.insert("id".to_string(), document_id.to_string());
    response.insert("hash".to_string(), hash);

    Ok(Json(response))
}

pub async fn get_document(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let document_id = uuid::Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let document = sqlx::query_as::<_, DocumentRecord>(
        "SELECT id, hash, file_path, file_size, mime_type FROM documents WHERE id = $1",
    )
    .bind(document_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let file_data = tokio::fs::read(&document.file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", document.mime_type)
        .header("Cache-Control", "public, max-age=31536000")
        .body(file_data.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn list_documents(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<DocumentListItem>>, StatusCode> {
    let documents = sqlx::query_as::<_, DocumentListItem>(
        r#"
        SELECT
            d.id,
            d.hash,
            d.file_size,
            d.mime_type,
            COALESCE(dd.created_at, d.created_at) as created_at,
            dd.description
        FROM documents d
        LEFT JOIN document_descriptions dd ON d.id = dd.document_id
        ORDER BY COALESCE(dd.created_at, d.created_at) DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(documents))
}
