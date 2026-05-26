# Build stage
FROM rust:1.94 AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml migration/Cargo.lock ./migration/
COPY src ./src
COPY migration/src ./migration/src

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/oak-maillist /usr/local/bin/oak-maillist
COPY --from=builder /app/target/release/migration /usr/local/bin/migration
COPY config ./config

ENV CONFIG_DIR=/app/config
ENV RUN_MODE=production

EXPOSE 3000 2525

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health/live || exit 1

CMD ["oak-maillist"]
