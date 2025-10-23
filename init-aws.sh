#!/bin/bash

awslocal dynamodb create-table \
  --table-name files \
  --attribute-definitions \
    AttributeName=pk,AttributeType=S \
  --key-schema \
    AttributeName=pk,KeyType=HASH \
  --billing-mode PAY_PER_REQUEST \
  --region us-east-1 2>&1 || echo "Table might already exist"

awslocal s3 mb s3://ingestor-processed --region us-east-1 2>&1 || echo "Bucket might already exist"
