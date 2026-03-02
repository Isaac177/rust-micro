FROM rust:1.85-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY services ./services

RUN cargo build --release -p api-gateway

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/api-gateway /usr/local/bin/api-gateway

EXPOSE 8080

CMD ["api-gateway"]
