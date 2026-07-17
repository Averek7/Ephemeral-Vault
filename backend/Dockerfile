FROM rust:1.85-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock Anchor.toml ./
COPY backend ./backend
COPY programs ./programs

RUN cargo build --release -p backend

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/backend /usr/local/bin/ephemeral-vault-backend
COPY backend/migrations ./migrations

ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080

EXPOSE 8080

CMD ["ephemeral-vault-backend"]
