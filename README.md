# File Ingestor - LocalStack Pipeline

Sistema de ingestão de arquivos utilizando LocalStack com S3, DynamoDB, Lambda e API Gateway.

## Arquitetura

Pipeline serverless para processamento de arquivos:
1. Upload via API Gateway → Lambda Upload → S3 (ingestor-raw)
2. S3 Trigger → Lambda Ingest → S3 (ingestor-processed) + DynamoDB
3. Consulta via API Gateway → Lambda Get/List → DynamoDB + S3

## Pré-requisitos

- Docker e Docker Compose instalados
- Porta 8080 e 4566 disponíveis

## Inicialização

### 1. Subir os serviços
```bash
docker compose up -d
```

### 2. Aguardar inicialização (aproximadamente 30 segundos)
```bash
docker logs cdn-server-localstack-1 --follow
```

## Estrutura dos Recursos AWS

### S3 Buckets
- `ingestor-raw`: Bucket de entrada para arquivos
- `ingestor-processed`: Bucket de saída com arquivos processados

### DynamoDB Table: files
- **PK**: `pk` (string) - Checksum SHA256 do arquivo
- **Atributos**: bucket, key, size, etag, status, contentType, processedAt, checksum

### Lambda Functions
- `upload`: Recebe arquivo via API Gateway e salva no bucket raw
- `file-ingestor`: Processa arquivo (trigger S3), calcula checksum, move para processed
- `list-files`: Lista arquivos com filtros opcionais (status, from/to)
- `get-file`: Retorna arquivo por ID (checksum)

### API Gateway
- `POST /files`: Upload de arquivo
- `GET /files`: Lista arquivos (query params: status, from, to)
- `GET /files/{id}`: Obtém arquivo por ID

## Testando o Pipeline

### 1. Executar arquivo de teste
```bash
./test-pipeline.sh
```

O API_ID pode ser obtido dos logs do LocalStack.

### 2. Verificar processamento
```bash
docker exec cdn-server-localstack-1 awslocal s3 ls s3://ingestor-raw/ --recursive

docker exec cdn-server-localstack-1 awslocal s3 ls s3://ingestor-processed/processed/

docker exec cdn-server-localstack-1 awslocal dynamodb scan --table-name files --region us-east-1
```

### 3. Listar arquivos via API
```bash
curl "http://localhost:8080/api/documents" | jq

curl "http://localhost:8080/api/documents?status=PROCESSED" | jq

curl "http://localhost:8080/api/documents?from=2025-01-01T00:00:00Z" | jq
```

### 4. Obter arquivo por ID
```bash
# Substitua {id} pelo checksum retornado no upload
curl "http://localhost:8080/api/documents/{id}" --output arquivo.txt
```

## Comandos Úteis

### Parar e remover tudo
```bash
docker compose down
```

## Acesso à Aplicação

- **Frontend**: http://localhost:8080
- **LocalStack**: http://localhost:4566
- **LocalStack Health**: http://localhost:4566/_localstack/health
