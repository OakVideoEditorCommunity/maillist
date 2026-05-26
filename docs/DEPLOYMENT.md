# Deployment Guide

This guide covers how to build and deploy the Oak MailList service in production environments.

## Overview

Oak MailList is a Rust-based mailing list service consisting of:

- **Backend**: Axum (HTTP API) + SeaORM (database ORM), supporting SQLite, MySQL, and PostgreSQL
- **Frontend**: Vue 3 + Vite + Element Plus SPA
- **SMTP Server**: Built-in incoming SMTP server for receiving list emails
- **Background Tasks**: Digest generation, AI moderation, bounce handling, cleanup

Default ports:

| Service | Port | Description |
|---------|------|-------------|
| HTTP API | 3000 | REST API and static file serving |
| SMTP | 2525 | Incoming email server |

---

## System Requirements

### Minimum

- **OS**: Linux (glibc 2.31+), macOS 12+, Windows Server 2019+
- **CPU**: 1 core
- **Memory**: 512 MB RAM
- **Disk**: 1 GB (depends on archive storage needs)
- **Rust**: 1.85+ (if building from source)
- **Node.js**: 20+ (if building frontend from source)

### Recommended for Production

- **CPU**: 2+ cores
- **Memory**: 2 GB RAM
- **Database**: PostgreSQL 14+ or MySQL 8.0+
- **Reverse Proxy**: Nginx or Traefik (for TLS termination and static file caching)

---

## Configuration

Configuration is loaded from TOML files and can be overridden via environment variables.

### Configuration Files

Place config files in a directory (default: `./config`):

```
config/
├── default.toml      # Base configuration (required)
└── production.toml   # Environment overrides (optional)
```

Set the config directory via environment variable:

```bash
export CONFIG_DIR=/app/config
export RUN_MODE=production   # loads production.toml if present
```

### Environment Variable Overrides

All config keys can be overridden via environment variables using the prefix `OAK__` and double underscores as delimiters:

```bash
# Database
export OAK__DATABASE__URL="postgres://user:pass@localhost:5432/oak_maillist"

# Security
export OAK__SECURITY__JWT_SECRET="your-strong-secret-key-here"
export OAK__SECURITY__WEBAUTHN_RP_ID="mail.example.com"

# Server
export OAK__SERVER__BASE_URL="https://mail.example.com"

# SMTP
export OAK__SMTP__OUTGOING__HOST="smtp.example.com"
export OAK__SMTP__OUTGOING__USERNAME="noreply@example.com"
export OAK__SMTP__OUTGOING__PASSWORD="app-password"
export OAK__SMTP__OUTGOING__FROM_ADDRESS="noreply@example.com"

# AI Moderation (optional)
export OAK__AI_MODERATION__ENABLED="true"
export OAK__AI_MODERATION__ACCESS_KEY_ID="your-key"
export OAK__AI_MODERATION__ACCESS_KEY_SECRET="your-secret"
```

### Critical Production Settings

Edit `config/production.toml` or use environment variables:

```toml
[server]
host = "0.0.0.0"
port = 3000
base_url = "https://mail.example.com"

[security]
jwt_secret = "REPLACE-WITH-CRYPTOGRAPHICALLY-RANDOM-STRING"
jwt_expiration_seconds = 900
refresh_token_expiration_days = 7
session_token_expiration_seconds = 600
password_min_length = 8
webauthn_rp_id = "mail.example.com"   # Required for Passkey authentication

[database]
url = "postgres://oak:PASSWORD@localhost:5432/oak_maillist"
max_connections = 20
min_connections = 5

[smtp.outgoing]
host = "smtp.example.com"
port = 587
username = "noreply@example.com"
password = "YOUR_APP_PASSWORD"
from_address = "noreply@example.com"

[archive]
enabled = true
storage_path = "/var/lib/oak-maillist/archives"
max_attachment_size_mb = 10

[logging]
level = "info"
format = "json"   # Use "json" for production log aggregation
```

**Security checklist:**
- [ ] Change `jwt_secret` to a cryptographically random string (min 32 bytes)
- [ ] Set `webauthn_rp_id` to your domain if using Passkey authentication
- [ ] Use a strong database password
- [ ] Enable TLS at the reverse proxy level
- [ ] Restrict SMTP incoming port access with firewall rules

---

## Building from Source

### 1. Clone Repository

```bash
git clone --recursive https://github.com/your-org/oak-maillist.git
cd oak-maillist
```

> **Note**: The `frontend/` directory is a Git submodule. Use `--recursive` or run `git submodule update --init --recursive` after cloning.

### 2. Build Backend

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build release binary
cargo build --release

# Binaries will be at:
# target/release/oak-maillist   # Main server
# target/release/migration      # Database migration CLI
```

### 3. Build Frontend

```bash
cd frontend
npm install
npm run build

# Static files will be at frontend/dist/
cd ..
```

The backend serves static files from `frontend/dist/` automatically when present.

---

## Deployment Methods

### Method 1: Native Deployment

1. **Prepare directories:**

```bash
sudo mkdir -p /opt/oak-maillist/config
sudo mkdir -p /var/lib/oak-maillist/archives
sudo cp -r config/* /opt/oak-maillist/config/
sudo cp -r frontend/dist /opt/oak-maillist/
sudo cp target/release/oak-maillist /usr/local/bin/
```

2. **Create systemd service** (`/etc/systemd/system/oak-maillist.service`):

```ini
[Unit]
Description=Oak MailList Service
After=network.target

[Service]
Type=simple
User=oak
Group=oak
WorkingDirectory=/opt/oak-maillist
Environment="CONFIG_DIR=/opt/oak-maillist/config"
Environment="RUN_MODE=production"
ExecStart=/usr/local/bin/oak-maillist
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

3. **Start service:**

```bash
sudo useradd -r -s /bin/false oak
sudo systemctl daemon-reload
sudo systemctl enable --now oak-maillist
sudo systemctl status oak-maillist
```

### Method 2: Docker

1. **Build image:**

```bash
docker build -t oak-maillist:latest .
```

2. **Run container:**

```bash
docker run -d \
  --name oak-maillist \
  -p 3000:3000 \
  -p 2525:2525 \
  -v /host/config:/app/config \
  -v /host/archives:/var/lib/oak-maillist/archives \
  -e OAK__DATABASE__URL="postgres://oak:pass@db:5432/oak_maillist" \
  -e OAK__SECURITY__JWT_SECRET="your-secret" \
  -e OAK__SERVER__BASE_URL="https://mail.example.com" \
  oak-maillist:latest
```

### Method 3: Docker Compose (Recommended)

Use the provided `docker-compose.yml`:

```bash
# Edit docker-compose.yml to set your environment variables
cp docker-compose.yml docker-compose.prod.yml
# Edit docker-compose.prod.yml

docker compose -f docker-compose.prod.yml up -d
```

The Compose file includes:
- PostgreSQL database with persistent volume
- Oak MailList application with health checks
- Automatic restart policy

---

## Database Setup

### SQLite (Development / Small Deployments)

No additional setup required. The application creates the database file automatically.

```toml
[database]
url = "sqlite:///var/lib/oak-maillist/data.db?mode=rwc"
```

> **Warning**: SQLite does not support concurrent writes well. Use PostgreSQL or MySQL for production.

### PostgreSQL

```bash
# Create database and user
sudo -u postgres psql -c "CREATE USER oak WITH PASSWORD 'strong_password';"
sudo -u postgres psql -c "CREATE DATABASE oak_maillist OWNER oak;"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE oak_maillist TO oak;"
```

```toml
[database]
url = "postgres://oak:strong_password@localhost:5432/oak_maillist"
```

### MySQL

```sql
CREATE DATABASE oak_maillist CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER 'oak'@'localhost' IDENTIFIED BY 'strong_password';
GRANT ALL PRIVILEGES ON oak_maillist.* TO 'oak'@'localhost';
FLUSH PRIVILEGES;
```

```toml
[database]
url = "mysql://oak:strong_password@localhost:5432/oak_maillist"
```

### Migrations

Migrations run automatically on startup. To run manually:

```bash
# Using the migration binary
cd migration
cargo run -- up

# Or using sea-orm-cli
cargo install sea-orm-cli
sea-orm-cli migrate up
```

---

## Reverse Proxy (Nginx)

Example Nginx configuration:

```nginx
server {
    listen 443 ssl http2;
    server_name mail.example.com;

    ssl_certificate /etc/letsencrypt/live/mail.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/mail.example.com/privkey.pem;

    client_max_body_size 50M;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /api {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name mail.example.com;
    return 301 https://$server_name$request_uri;
}
```

**Important for Passkey/WebAuthn:**
The `base_url` in config must match the HTTPS URL exposed by Nginx. The `webauthn_rp_id` must be set to the domain name (e.g., `mail.example.com`).

---

## Health Checks

The service exposes a liveness probe:

```bash
curl http://localhost:3000/health/live
```

Docker Compose and Kubernetes can use this for health checks.

---

## Logging

### Pretty Format (Development)

```toml
[logging]
level = "debug"
format = "pretty"
```

### JSON Format (Production)

```toml
[logging]
level = "info"
format = "json"
```

Logs include structured fields compatible with ELK/Loki/Grafana.

---

## Upgrading

1. **Backup database:**

```bash
pg_dump -U oak oak_maillist > backup_$(date +%Y%m%d).sql
```

2. **Pull new code and rebuild:**

```bash
git pull
git submodule update --init --recursive
cargo build --release
cd frontend && npm install && npm run build && cd ..
```

3. **Restart service:**

```bash
sudo systemctl restart oak-maillist
```

Migrations run automatically on startup.

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `Connection refused` on SMTP port | Check firewall rules; ensure `smtp.incoming.enabled = true` |
| Passkey registration fails | Verify `base_url` and `webauthn_rp_id` match your HTTPS domain |
| Frontend shows 404 | Ensure `frontend/dist/` exists and contains `index.html` |
| Database connection errors | Check `database.url` and network connectivity |
| Emails not sending | Verify SMTP outgoing credentials; check logs for lettre errors |
| High memory usage | Reduce `database.max_connections`; enable archive cleanup |

---

## Reference

- [Oak MailList Repository](https://github.com/your-org/oak-maillist)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Vue 3 Documentation](https://vuejs.org/)
