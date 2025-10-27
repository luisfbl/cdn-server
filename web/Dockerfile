FROM node:20-alpine AS builder

WORKDIR /app

COPY package.json pnpm-lock.yaml ./

RUN npm install -g pnpm
RUN pnpm install

COPY . .

RUN pnpm build

FROM node:20-alpine

WORKDIR /app

# Copy the built application
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./
COPY --from=builder /app/node_modules ./node_modules

EXPOSE 3001

ENV PORT=3001
ENV HOST=0.0.0.0

# Run the SvelteKit Node server
CMD ["sh", "-c", "echo \"window.CONTAINER_ID = '$(hostname)';\" > /app/build/client/container-id.js && node build/index.js"]
