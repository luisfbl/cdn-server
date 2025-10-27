import json
import boto3
import hashlib
from datetime import datetime

s3 = boto3.client("s3", endpoint_url="http://localstack:4566")
dynamodb = boto3.client("dynamodb", endpoint_url="http://localstack:4566")


def lambda_handler(event, context):
    print(f"Event received: {json.dumps(event)}")

    for record in event["Records"]:
        bucket = record["s3"]["bucket"]["name"]
        key = record["s3"]["object"]["key"]
        size = record["s3"]["object"]["size"]

        print(f"Processing file: bucket={bucket}, key={key}, size={size}")

        try:
            response = s3.get_object(Bucket=bucket, Key=key)
            file_data = response["Body"].read()
            content_type = response.get("ContentType", "application/octet-stream")
            etag = response["ETag"].strip('"')

            checksum = hashlib.sha256(file_data).hexdigest()
            raw_timestamp = datetime.utcnow().isoformat() + "Z"

            dynamodb.put_item(
                TableName="files",
                Item={
                    "pk": {"S": checksum},
                    "file": {"S": key},
                    "bucket": {"S": bucket},
                    "key": {"S": key},
                    "size": {"N": str(size)},
                    "etag": {"S": etag},
                    "checksum": {"S": checksum},
                    "contentType": {"S": content_type},
                    "status": {"S": "RAW"},
                    "processedAt": {"S": raw_timestamp},
                },
            )

            dest_bucket = "ingestor-processed"
            dest_key = f"processed/{checksum}"

            s3.put_object(
                Bucket=dest_bucket,
                Key=dest_key,
                Body=file_data,
                ContentType=content_type,
            )

            head_response = s3.head_object(Bucket=dest_bucket, Key=dest_key)
            processed_etag = head_response["ETag"].strip('"')

            processed_timestamp = datetime.utcnow().isoformat() + "Z"

            dynamodb.put_item(
                TableName="files",
                Item={
                    "pk": {"S": checksum},
                    "file": {"S": dest_key},
                    "bucket": {"S": dest_bucket},
                    "key": {"S": dest_key},
                    "size": {"N": str(size)},
                    "etag": {"S": processed_etag},
                    "checksum": {"S": checksum},
                    "contentType": {"S": content_type},
                    "status": {"S": "PROCESSED"},
                    "processedAt": {"S": processed_timestamp},
                },
            )
            s3.delete_object(Bucket=bucket, Key=key)

            return {
                "statusCode": 200,
                "body": json.dumps(
                    {
                        "message": "File processed successfully",
                        "checksum": checksum,
                        "key": dest_key,
                        "status": "PROCESSED",
                    }
                ),
            }

        except Exception as e:
            print(f"Error processing file: {str(e)}")

            try:
                error_time = datetime.utcnow().isoformat() + "Z"
                dynamodb.put_item(
                    TableName="files",
                    Item={
                        "pk": {"S": f"error-{key}-{error_time}"},
                        "file": {"S": key},
                        "bucket": {"S": bucket},
                        "key": {"S": key},
                        "size": {"N": str(size)},
                        "status": {"S": "RAW"},
                        "processedAt": {"S": error_time},
                        "error": {"S": str(e)},
                    },
                )
            except Exception as db_error:
                print(f"Failed to store error in DynamoDB: {str(db_error)}")

            raise e
