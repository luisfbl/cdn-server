use crate::aws_client::AwsClients;
use crate::models::{DocumentListItem};
use crate::utils::calculate_hash;
use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::Response,
};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;

pub async fn upload_document(
    State(aws_clients): State<Arc<AwsClients>>,
    mut multipart: Multipart,
) -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut file_data = Vec::new();
    let mut description = String::new();
    let mut content_type = String::from("application/octet-stream");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
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
    let size = file_data.len();

    let file_id = hash.clone();

    let existing = aws_clients
        .get_file_metadata(&file_id)
        .await
        .map_err(|e| {
            eprintln!("DynamoDB get error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let document_id = if existing.is_some() {
        file_id.clone()
    } else {
        let bucket = "ingestor-processed";
        let file_key = format!("documents/{}", &hash);

        let etag = aws_clients
            .upload_to_s3(bucket, &file_key, Bytes::from(file_data), &content_type)
            .await
            .map_err(|e| {
                eprintln!("S3 upload error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        aws_clients
            .put_file_metadata_with_description(
                &file_id,
                &file_key,
                bucket,
                size as i64,
                &etag,
                &hash,
                &content_type,
                "PROCESSED",
                if description.is_empty() { None } else { Some(&description) },
            )
            .await
            .map_err(|e| {
                eprintln!("DynamoDB put error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        file_id
    };

    let mut response = HashMap::new();
    response.insert("id".to_string(), document_id.clone());
    response.insert("hash".to_string(), hash);

    Ok(Json(response))
}

pub async fn get_document(
    State(aws_clients): State<Arc<AwsClients>>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let metadata = aws_clients
        .get_file_metadata(&id)
        .await
        .map_err(|e| {
            eprintln!("DynamoDB get error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let bucket = metadata
        .get("bucket")
        .and_then(|v| v.as_s().ok())
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let key = metadata
        .get("key")
        .and_then(|v| v.as_s().ok())
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let mime_type = metadata
        .get("contentType")
        .and_then(|v| v.as_s().ok())
        .map(|s| s.as_str())
        .unwrap_or("application/octet-stream");

    let file_data = aws_clients
        .get_from_s3(bucket, key)
        .await
        .map_err(|e| {
            eprintln!("S3 get error: {}", e);
            StatusCode::NOT_FOUND
        })?;

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime_type)
        .header("Cache-Control", "public, max-age=31536000")
        .body(file_data.to_vec().into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn list_documents(
    State(aws_clients): State<Arc<AwsClients>>,
) -> Result<Json<Vec<DocumentListItem>>, StatusCode> {
    let items = aws_clients
        .scan_files(Some(100))
        .await
        .map_err(|e| {
            eprintln!("DynamoDB scan error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let documents: Vec<DocumentListItem> = items
        .iter()
        .filter_map(|item| {
            Some(DocumentListItem {
                id: item.get("pk").and_then(|v| v.as_s().ok())?.to_string(),
                hash: item.get("checksum").and_then(|v| v.as_s().ok())?.to_string(),
                file_size: item
                    .get("size")
                    .and_then(|v| v.as_n().ok())
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0),
                mime_type: item
                    .get("contentType")
                    .and_then(|v| v.as_s().ok())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string()),
                created_at: item
                    .get("processedAt")
                    .and_then(|v| v.as_s().ok())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(chrono::Utc::now),
                description: item
                    .get("description")
                    .and_then(|v| v.as_s().ok())
                    .map(|s| s.to_string()),
            })
        })
        .collect();

    Ok(Json(documents))
}
