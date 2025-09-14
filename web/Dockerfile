FROM node:20-alpine AS builder

WORKDIR /app

COPY package.json pnpm-lock.yaml ./

RUN npm install -g pnpm
RUN pnpm install

COPY . .

RUN pnpm build

FROM node:20-alpine

WORKDIR /app

RUN npm install -g serve

COPY --from=builder /app/build ./build

EXPOSE 3001

CMD ["sh", "-c", "echo \"window.CONTAINER_ID = '$(hostname)';\" > /app/build/container-id.js && serve -s build -l 3001"]
