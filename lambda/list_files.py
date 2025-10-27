import json
import boto3
from datetime import datetime

dynamodb = boto3.client("dynamodb", endpoint_url="http://localstack:4566")


def lambda_handler(event, context):
    print(f"Event: {json.dumps(event)}")

    try:
        query_params = event.get("queryStringParameters", {}) or {}
        status_filter = query_params.get("status")
        from_date = query_params.get("from")
        to_date = query_params.get("to")

        print(f"Filters: status={status_filter}, from={from_date}, to={to_date}")

        response = dynamodb.scan(TableName="files", Limit=100)

        items = response.get("Items", [])
        print(f"Found {len(items)} items")

        files = []
        for item in items:
            try:
                pk = item.get("pk", {}).get("S", "")
                status = item.get("status", {}).get("S", "")
                processed_at_str = item.get("processedAt", {}).get("S", "")

                if status_filter and status != status_filter:
                    continue

                try:
                    processed_at = datetime.fromisoformat(
                        processed_at_str.replace("Z", "+00:00")
                    )
                except:
                    processed_at = datetime.utcnow()

                if from_date:
                    try:
                        from_dt = datetime.fromisoformat(
                            from_date.replace("Z", "+00:00")
                        )
                        if processed_at < from_dt:
                            continue
                    except:
                        pass

                if to_date:
                    try:
                        to_dt = datetime.fromisoformat(to_date.replace("Z", "+00:00"))
                        if processed_at > to_dt:
                            continue
                    except:
                        pass

                file_obj = {
                    "id": pk,
                    "pk": pk,
                    "hash": item.get("checksum", {}).get("S", pk),
                    "bucket": item.get("bucket", {}).get("S", ""),
                    "key": item.get("key", {}).get("S", ""),
                    "size": int(item.get("size", {}).get("N", "0")),
                    "etag": item.get("etag", {}).get("S", ""),
                    "status": status,
                    "contentType": item.get("contentType", {}).get(
                        "S", "application/octet-stream"
                    ),
                    "processedAt": processed_at_str,
                    "checksum": item.get("checksum", {}).get("S", ""),
                }

                files.append(file_obj)
            except Exception as e:
                print(f"Error parsing item: {e}")
                continue

        return {
            "statusCode": 200,
            "headers": {
                "Content-Type": "application/json",
                "Access-Control-Allow-Origin": "*",
            },
            "body": json.dumps({"files": files, "count": len(files)}),
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
