FROM rust:1.89.0 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY static ./static

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/static /app/static

EXPOSE 3000
ENTRYPOINT ["/app/backend"]