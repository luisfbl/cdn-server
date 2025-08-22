FROM rust:1.89.0 AS builder

WORKDIR /app

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl

RUN rm -rf src
COPY src ./src
COPY migrations ./migrations
COPY static ./static

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest
RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/backend /app/backend
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/static /app/static

EXPOSE 3000
ENTRYPOINT ["/app/backend"]