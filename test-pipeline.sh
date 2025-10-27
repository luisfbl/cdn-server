#!/bin/bash

set -e

API_ID=$(docker logs cdn-server-localstack-1 2>&1 | grep "API ID:" | tail -1 | awk '{print $NF}')
echo "API ID: $API_ID"
echo ""

echo "Test content - $(date)" > /tmp/test.txt
RESPONSE=$(curl -s -X POST -H "Content-Type: text/plain" --data-binary "@/tmp/test.txt" \
  "http://localhost:4566/restapis/$API_ID/prod/_user_request_/files")
FILE_ID=$(echo $RESPONSE | jq -r '.id')

echo "Waiting for Lambda processing..."
sleep 5
echo ""

docker exec cdn-server-localstack-1 awslocal dynamodb get-item \
  --table-name files \
  --key "{\"pk\":{\"S\":\"$FILE_ID\"}}" \
  --region us-east-1 | jq '.Item | {pk: .pk.S, status: .status.S, size: .size.N}'

echo ""
curl -s "http://localhost:4566/restapis/$API_ID/prod/_user_request_/files" | jq '.files | length'
echo ""

echo "Downloading file by ID..."
curl -s "http://localhost:4566/restapis/$API_ID/prod/_user_request_/files/$FILE_ID"
