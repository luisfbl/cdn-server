use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_s3::Client as S3Client;
use bytes::Bytes;
use std::env;
use std::collections::HashMap;

pub struct AwsClients {
    pub s3: S3Client,
    pub dynamodb: DynamoDbClient,
}

impl AwsClients {
    pub async fn new() -> Self {
        let endpoint_url = env::var("AWS_ENDPOINT_URL")
            .unwrap_or_else(|_| "http://localstack:4566".to_string());
        let config = aws_config::defaults(BehaviorVersion::latest())
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(&endpoint_url)
            .force_path_style(true)  // Important for LocalStack
            .build();
        let s3 = S3Client::from_conf(s3_config);

        let dynamodb_config = aws_sdk_dynamodb::config::Builder::from(&config)
            .endpoint_url(&endpoint_url)
            .build();
        let dynamodb = DynamoDbClient::from_conf(dynamodb_config);

        Self { s3, dynamodb }
    }

    pub async fn upload_to_s3(
        &self,
        bucket: &str,
        key: &str,
        data: Bytes,
        content_type: &str,
    ) -> Result<String, String> {
        let result = self.s3
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(data.into())
            .content_type(content_type)
            .send()
            .await;

        match result {
            Ok(output) => {
                let etag = output
                    .e_tag()
                    .unwrap_or("unknown")
                    .trim_matches('"')
                    .to_string();
                Ok(etag)
            }
            Err(e) => {
                eprintln!("❌ S3 upload error details: {:?}", e);
                Err(format!("S3 upload error: {}", e))
            }
        }
    }

    pub async fn get_from_s3(&self, bucket: &str, key: &str) -> Result<Bytes, String> {
        let output = self
            .s3
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                eprintln!("S3 get error details: bucket={}, key={}, error={}", bucket, key, e);
                format!("S3 get error: {}", e)
            })?;

        let data = output
            .body
            .collect()
            .await
            .map_err(|e| format!("Failed to read S3 body: {}", e))?
            .into_bytes();

        Ok(data)
    }

    pub async fn put_file_metadata_with_description(
        &self,
        pk: &str,
        file_key: &str,
        bucket: &str,
        size: i64,
        etag: &str,
        checksum: &str,
        content_type: &str,
        status: &str,
        description: Option<&str>,
    ) -> Result<(), String> {
        let timestamp = chrono::Utc::now().to_rfc3339();

        let mut request = self.dynamodb
            .put_item()
            .table_name("files")
            .item("pk", AttributeValue::S(pk.to_string()))
            .item("file", AttributeValue::S(file_key.to_string()))
            .item("bucket", AttributeValue::S(bucket.to_string()))
            .item("key", AttributeValue::S(file_key.to_string()))
            .item("size", AttributeValue::N(size.to_string()))
            .item("ETag", AttributeValue::S(etag.to_string()))
            .item("checksum", AttributeValue::S(checksum.to_string()))
            .item("contentType", AttributeValue::S(content_type.to_string()))
            .item("status", AttributeValue::S(status.to_string()))
            .item("processedAt", AttributeValue::S(timestamp));

        if let Some(desc) = description {
            request = request.item("description", AttributeValue::S(desc.to_string()));
        }

        request
            .send()
            .await
            .map_err(|e| format!("DynamoDB put error: {}", e))?;

        Ok(())
    }

    pub async fn get_file_metadata(&self, pk: &str) -> Result<Option<HashMap<String, AttributeValue>>, String> {
        let result = self
            .dynamodb
            .get_item()
            .table_name("files")
            .key("pk", AttributeValue::S(pk.to_string()))
            .send()
            .await
            .map_err(|e| format!("DynamoDB get error: {}", e))?;

        Ok(result.item)
    }

    pub async fn scan_files(&self, limit: Option<i32>) -> Result<Vec<HashMap<String, AttributeValue>>, String> {
        let mut request = self.dynamodb.scan().table_name("files");

        if let Some(l) = limit {
            request = request.limit(l);
        }

        let result = request
            .send()
            .await
            .map_err(|e| format!("DynamoDB scan error: {}", e))?;

        Ok(result.items.unwrap_or_default())
    }
}
