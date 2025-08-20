FROM rust:1.89.0 AS builder

RUN rustup target add x86_64-unknown-linux-gnu
WORKDIR /app

COPY . .
RUN cargo build --release --target x86_64-unknown-linux-gnu

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/backend /backend

EXPOSE 3000
ENTRYPOINT ["backend"]