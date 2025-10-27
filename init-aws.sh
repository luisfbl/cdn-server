#!/bin/bash

cd /tmp

awslocal dynamodb create-table \
  --table-name files \
  --attribute-definitions AttributeName=pk,AttributeType=S \
  --key-schema AttributeName=pk,KeyType=HASH \
  --billing-mode PAY_PER_REQUEST \
  --region us-east-1

awslocal s3 mb s3://ingestor-raw --region us-east-1
awslocal s3 mb s3://ingestor-processed --region us-east-1

cp /etc/localstack/init/ready.d/lambda/upload.py .
zip -q upload.zip upload.py
awslocal lambda create-function \
  --function-name upload \
  --runtime python3.11 \
  --handler upload.lambda_handler \
  --role arn:aws:iam::000000000000:role/lambda-role \
  --zip-file fileb:///tmp/upload.zip \
  --region us-east-1 \
  --timeout 60 \
  --environment "Variables={AWS_ENDPOINT_URL=http://localstack:4566}"

cp /etc/localstack/init/ready.d/lambda/ingest.py .
zip -q ingest.zip ingest.py
awslocal lambda create-function \
  --function-name file-ingestor \
  --runtime python3.11 \
  --handler ingest.lambda_handler \
  --role arn:aws:iam::000000000000:role/lambda-role \
  --zip-file fileb:///tmp/ingest.zip \
  --region us-east-1 \
  --timeout 60 \
  --environment "Variables={AWS_ENDPOINT_URL=http://localstack:4566}"

cp /etc/localstack/init/ready.d/lambda/list_files.py .
zip -q list_files.zip list_files.py
awslocal lambda create-function \
  --function-name list-files \
  --runtime python3.11 \
  --handler list_files.lambda_handler \
  --role arn:aws:iam::000000000000:role/lambda-role \
  --zip-file fileb:///tmp/list_files.zip \
  --region us-east-1 \
  --timeout 60 \
  --environment "Variables={AWS_ENDPOINT_URL=http://localstack:4566}"

cp /etc/localstack/init/ready.d/lambda/get_file.py .
zip -q get_file.zip get_file.py
awslocal lambda create-function \
  --function-name get-file \
  --runtime python3.11 \
  --handler get_file.lambda_handler \
  --role arn:aws:iam::000000000000:role/lambda-role \
  --zip-file fileb:///tmp/get_file.zip \
  --region us-east-1 \
  --timeout 60 \
  --environment "Variables={AWS_ENDPOINT_URL=http://localstack:4566}"

awslocal lambda wait function-active-v2 --function-name file-ingestor --region us-east-1

awslocal lambda add-permission \
  --function-name file-ingestor \
  --statement-id s3-trigger \
  --action lambda:InvokeFunction \
  --principal s3.amazonaws.com \
  --source-arn arn:aws:s3:::ingestor-raw \
  --region us-east-1

awslocal s3api put-bucket-notification-configuration \
  --bucket ingestor-raw \
  --notification-configuration '{
    "LambdaFunctionConfigurations": [
      {
        "LambdaFunctionArn": "arn:aws:lambda:us-east-1:000000000000:function:file-ingestor",
        "Events": ["s3:ObjectCreated:*"]
      }
    ]
  }' \
  --region us-east-1

API_ID=$(awslocal apigateway create-rest-api \
  --name "file-ingestor-api" \
  --description "File Ingestor API with Lambda integration" \
  --region us-east-1 \
  --output text \
  --query 'id' 2>&1)

ROOT_ID=$(awslocal apigateway get-resources \
  --rest-api-id $API_ID \
  --region us-east-1 \
  --output text \
  --query 'items[0].id')

FILES_RESOURCE_ID=$(awslocal apigateway create-resource \
  --rest-api-id $API_ID \
  --parent-id $ROOT_ID \
  --path-part files \
  --region us-east-1 \
  --output text \
  --query 'id')

FILE_ID_RESOURCE_ID=$(awslocal apigateway create-resource \
  --rest-api-id $API_ID \
  --parent-id $FILES_RESOURCE_ID \
  --path-part '{id}' \
  --region us-east-1 \
  --output text \
  --query 'id')

awslocal apigateway put-method \
  --rest-api-id $API_ID \
  --resource-id $FILES_RESOURCE_ID \
  --http-method POST \
  --authorization-type NONE \
  --region us-east-1

awslocal apigateway put-integration \
  --rest-api-id $API_ID \
  --resource-id $FILES_RESOURCE_ID \
  --http-method POST \
  --type AWS_PROXY \
  --integration-http-method POST \
  --uri arn:aws:apigateway:us-east-1:lambda:path/2015-03-31/functions/arn:aws:lambda:us-east-1:000000000000:function:upload/invocations \
  --region us-east-1

awslocal apigateway put-method \
  --rest-api-id $API_ID \
  --resource-id $FILES_RESOURCE_ID \
  --http-method GET \
  --authorization-type NONE \
  --region us-east-1

awslocal apigateway put-integration \
  --rest-api-id $API_ID \
  --resource-id $FILES_RESOURCE_ID \
  --http-method GET \
  --type AWS_PROXY \
  --integration-http-method POST \
  --uri arn:aws:apigateway:us-east-1:lambda:path/2015-03-31/functions/arn:aws:lambda:us-east-1:000000000000:function:list-files/invocations \
  --region us-east-1

awslocal apigateway put-method \
  --rest-api-id $API_ID \
  --resource-id $FILE_ID_RESOURCE_ID \
  --http-method GET \
  --authorization-type NONE \
  --region us-east-1

awslocal apigateway put-integration \
  --rest-api-id $API_ID \
  --resource-id $FILE_ID_RESOURCE_ID \
  --http-method GET \
  --type AWS_PROXY \
  --integration-http-method POST \
  --uri arn:aws:apigateway:us-east-1:lambda:path/2015-03-31/functions/arn:aws:lambda:us-east-1:000000000000:function:get-file/invocations \
  --region us-east-1

awslocal apigateway create-deployment \
  --rest-api-id $API_ID \
  --stage-name prod \
  --region us-east-1

rm -f /tmp/*.py /tmp/*.zip
