import json
import boto3
import base64

s3 = boto3.client("s3", endpoint_url="http://localstack:4566")
dynamodb = boto3.client("dynamodb", endpoint_url="http://localstack:4566")


def lambda_handler(event, context):
    print(f"Event: {json.dumps(event)}")

    try:
        path_params = event.get("pathParameters", {}) or {}
        file_id = path_params.get("id")

        if not file_id:
            return {
                "statusCode": 400,
                "headers": {
                    "Content-Type": "application/json",
                    "Access-Control-Allow-Origin": "*",
                },
                "body": json.dumps({"error": "Missing file ID"}),
            }

        response = dynamodb.get_item(TableName="files", Key={"pk": {"S": file_id}})

        if "Item" not in response:
            return {
                "statusCode": 404,
                "headers": {
                    "Content-Type": "application/json",
                    "Access-Control-Allow-Origin": "*",
                },
                "body": json.dumps({"error": "File not found"}),
            }

        item = response["Item"]
        bucket = item.get("bucket", {}).get("S", "")
        key = item.get("key", {}).get("S", "")
        content_type = item.get("contentType", {}).get("S", "application/octet-stream")

        s3_response = s3.get_object(Bucket=bucket, Key=key)
        file_data = s3_response["Body"].read()

        return {
            "statusCode": 200,
            "headers": {
                "Content-Type": content_type,
                "Access-Control-Allow-Origin": "*",
                "Cache-Control": "public, max-age=31536000",
            },
            "body": base64.b64encode(file_data).decode("utf-8"),
            "isBase64Encoded": True,
        }

    except s3.exceptions.NoSuchKey:
        return {
            "statusCode": 404,
            "headers": {
                "Content-Type": "application/json",
                "Access-Control-Allow-Origin": "*",
            },
            "body": json.dumps({"error": "File not found in S3"}),
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
