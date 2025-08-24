FROM rust:1.89.0 AS builder

WORKDIR /app

# Copy all files needed for compilation
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY static ./static

# Build the application
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/static /app/static

EXPOSE 3000
ENTRYPOINT ["/app/backend"]