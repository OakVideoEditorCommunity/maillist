FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Install Rust tooling
RUN rustup component add rustfmt clippy \
    && cargo install cargo-watch cargo-tarpaulin sea-orm-cli \
    && cargo install diesel_cli --no-default-features --features postgres

# Ensure vscode user owns cargo home
RUN mkdir -p /usr/local/cargo && chown -R vscode:vscode /usr/local/cargo

USER vscode
WORKDIR /workspace
