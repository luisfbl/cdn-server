# CDN Server - Docker Swarm

Sistema de CDN distribuído utilizando Docker Swarm para demonstrar balanceamento de carga e escalabilidade horizontal.

## Pré-requisitos

- Docker instalado
- Portas 8080, 9000 e 8000 disponíveis

## Inicialização do Docker Swarm

### 1. Inicializar o Swarm
```bash
docker swarm init
```

### 2. Construir as Imagens
```bash
docker build -t cdn-frontend:latest ./web

docker build -t cdn-backend:latest ./rest-server
```

### 3. Deploy da Stack
```bash
docker stack deploy -c docker-swarm.yml cdn-stack
```

## Verificação dos Serviços

### Listar serviços
```bash
docker service ls
```

### Verificar réplicas de um serviço
```bash
docker service ps cdn-stack_frontend
docker service ps cdn-stack_backend
```

## Comandos de Scaling

### Escalar frontend para 5 réplicas
```bash
docker service scale cdn-stack_frontend=5
```

### Escalar backend para 4 réplicas
```bash
docker service scale cdn-stack_backend=4
```

### Reduzir frontend para 2 réplicas
```bash
docker service scale cdn-stack_frontend=2
```

## Acesso à Aplicação

- **Frontend**: http://localhost:8080
- **Portainer**: http://localhost:9000

## Demonstração de Load Balancing

1. Acesse o frontend em http://localhost:8080
2. Abra o console do navegador (F12)
3. Recarregue a página várias vezes
4. Observe que o "Frontend Container ID" muda, demonstrando o balanceamento entre as réplicas

## Comandos Úteis

### Atualizar um serviço
```bash
docker service update --force cdn-stack_frontend
```

### Remover a stack
```bash
docker stack rm cdn-stack
```

### Sair do modo Swarm
```bash
docker swarm leave --force
```

## Estrutura da Aplicação

- **Frontend**: Aplicação SvelteKit que exibe arquivos e permite upload
- **Backend**: API REST em Rust para gerenciamento de arquivos
- **Postgres**: Banco de dados para metadados
- **Nginx**: Load balancer e proxy reverso
- **Portainer**: Interface web para gerenciamento do Docker
