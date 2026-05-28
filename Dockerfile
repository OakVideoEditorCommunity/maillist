# syntax=docker/dockerfile:1

# ── Stage 1: Build frontend ──
FROM node:26-alpine AS frontend-builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm install
COPY frontend/ ./
RUN npm run build

# ── Stage 2: Prepare Rust dependencies with cargo-chef ──
FROM rust:1.94 AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# ── Stage 3: Generate recipe.json ──
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/Cargo.toml
COPY src ./src
COPY migration/src ./migration/src
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 4: Build dependencies (cached layer) ──
FROM chef AS deps-builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# ── Stage 5: Build full Rust backend ──
FROM rust:1.94 AS builder
WORKDIR /app
# Copy cached dependencies
COPY --from=deps-builder /app/target target
COPY --from=deps-builder /usr/local/cargo /usr/local/cargo
# Copy source
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/Cargo.toml
COPY src ./src
COPY migration/src ./migration/src
# Copy frontend dist so rust-embed can include it at compile time
COPY --from=frontend-builder /app/dist ./frontend/dist
RUN cargo build --release --workspace

# ── Stage 6: Runtime ──
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries
COPY --from=builder /app/target/release/oak-maillist /usr/local/bin/oak-maillist
COPY --from=builder /app/target/release/migration /usr/local/bin/migration

# Ensure writable directories for auto-generated config, SQLite DB, and archives
RUN mkdir -p /app/config /app/storage/archives && chmod -R 755 /app/config /app/storage

ENV CONFIG_DIR=/app/config
ENV RUN_MODE=production

EXPOSE 3000 2525

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health/live || exit 1

CMD ["oak-maillist"]
