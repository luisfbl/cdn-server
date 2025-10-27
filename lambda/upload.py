import json
import boto3
import base64
import hashlib
from datetime import datetime

s3 = boto3.client("s3", endpoint_url="http://localstack:4566")
dynamodb = boto3.client("dynamodb", endpoint_url="http://localstack:4566")


def lambda_handler(event, context):
    print(f"Event: {json.dumps(event)}")

    try:
        body = event.get("body", "")
        is_base64 = event.get("isBase64Encoded", False)

        if is_base64:
            body = base64.b64decode(body)
        else:
            body = body.encode("utf-8") if isinstance(body, str) else body

        headers = event.get("headers", {})
        content_type = headers.get("content-type", "application/octet-stream")
        checksum = hashlib.sha256(body).hexdigest()

        try:
            response = dynamodb.get_item(TableName="files", Key={"pk": {"S": checksum}})

            if "Item" in response:
                return {
                    "statusCode": 200,
                    "headers": {
                        "Content-Type": "application/json",
                        "Access-Control-Allow-Origin": "*",
                    },
                    "body": json.dumps(
                        {
                            "id": checksum,
                            "hash": checksum,
                            "message": "File already exists",
                        }
                    ),
                }
        except Exception as e:
            print(f"Error checking existing file: {e}")

        timestamp = datetime.utcnow().strftime("%Y%m%d%H%M%S%f")
        file_key = f"raw/{timestamp}-{checksum}"

        s3.put_object(
            Bucket="ingestor-raw", Key=file_key, Body=body, ContentType=content_type
        )

        return {
            "statusCode": 200,
            "headers": {
                "Content-Type": "application/json",
                "Access-Control-Allow-Origin": "*",
            },
            "body": json.dumps(
                {
                    "id": checksum,
                    "hash": checksum,
                    "status": "uploaded",
                    "message": "File uploaded successfully, processing...",
                }
            ),
        }

    except Exception as e:
        print(f"Error: {str(e)}")
        return {
            "statusCode": 500,
            "headers": {
                "Content-Type": "application/json",
                "Access-Control-Allow-Origin": "*",
            },
            "body": json.dumps({"error": str(e)}),
        }
