# Oak MailList

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

A modern, self-hosted mailing list service built with Rust and Vue 3. Supports multi-factor authentication, Passkey login, AI-powered content moderation, and full email archiving with threading.

🌐 [中文文档](README-zh.md) | 📖 [Deployment Guide](docs/DEPLOYMENT.md)

---

## Features

### Core Mailing List

- **Multiple Lists**: Create and manage multiple mailing lists under custom domains
- **Subscriber Management**: Public or invite-only subscriptions, bulk import/export (CSV), digest modes (none/daily/weekly)
- **Email Archiving**: Full RFC 822 raw storage with text/HTML extraction, attachment handling, and thread building via `In-Reply-To` / `References`
- **SMTP Server**: Built-in async incoming SMTP server (port 2525) for receiving list emails directly
- **SMTP Relay**: Outbound email delivery via configurable SMTP host with DKIM support
- **Bounce Handling**: VERP-encoded return paths with automatic unsubscribe after 3 hard bounces
- **Digest Generation**: Hourly background task compiles daily/weekly digests for subscribers

### Security & Authentication

- **Password Auth**: Argon2id-hashed passwords with configurable minimum length
- **JWT Tokens**: Short-lived access tokens (15 min) + revocable refresh tokens (7 days)
- **TOTP MFA**: Time-based one-time passwords compatible with Google/Microsoft Authenticator, with 10 backup codes
- **Passkey / WebAuthn**: FIDO2 credential registration and login using `webauthn-rs`
- **Session Management**: List and revoke active sessions per device

### Content Moderation

- **Manual Moderation**: Approve, reject, discard, whitelist, or blacklist senders from a moderation queue
- **AI Moderation**: Integration with Aliyun Green (Content Moderation) API for automated risk scoring
  - Subject + body text analysis
  - Configurable risk thresholds (`flagged` ≥ 80, `caution` ≥ 50)
  - AI-flagged emails enter moderation queue for human review
  - Feedback loop for false-positive reporting
- **Sender Policies**: Per-list whitelist/blacklist rules with email pattern matching

### Administration

- **Dashboard**: Overview stats (users, lists, subscribers, messages, pending moderation)
- **Domain Management**: Configure domains with SMTP and DKIM settings
- **Email Templates**: Tera-powered templates for subscription confirm, unsubscribe, moderation notice, and welcome emails
- **User Management**: CRUD operations on users, role assignments, profile updates
- **Activity Logging**: Structured tracing logs in pretty or JSON format

### Frontend

- **Vue 3 + Vite + Element Plus** single-page application
- **Internationalization**: English and Chinese with automatic browser locale detection and per-user language preference
- **Cookie Consent**: GDPR-compliant cookie consent banner
- **Responsive UI**: Sidebar layout with list management, moderation queue, message archive, user profiles, and settings

### Observability

- **Health Checks**: `/health/live`, `/health/ready`, `/health` endpoints
- **Prometheus Metrics**: `/metrics` endpoint for monitoring
- **Structured Logging**: Configurable levels and formats (pretty / JSON) via `tracing`

---

## Technology Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust, Axum, Tokio |
| ORM | SeaORM (SQLite / MySQL / PostgreSQL) |
| Auth | jsonwebtoken, argon2, totp-rs, webauthn-rs |
| Email | lettre (SMTP), mailparse (parsing) |
| Templates | Tera |
| Frontend | Vue 3, Vue Router, Axios, Element Plus, vue-i18n |
| AI | Aliyun Green Content Moderation API |
| Metrics | metrics + metrics-exporter-prometheus |

---

## Quick Start

### Prerequisites

- Rust 1.85+
- Node.js 20+ (for frontend build)
- PostgreSQL 14+ / MySQL 8.0+ (or SQLite for testing)

### 1. Clone

```bash
git clone --recursive https://github.com/your-org/oak-maillist.git
cd oak-maillist
```

### 2. Configure

```bash
cp config/default.toml config/production.toml
# Edit config/production.toml with your settings
```

Minimum configuration:

```toml
[server]
host = "0.0.0.0"
port = 3000
base_url = "http://localhost:3000"

[security]
jwt_secret = "change-me-to-a-random-string"

[database]
url = "postgres://oak:password@localhost:5432/oak_maillist"
```

### 3. Build & Run

```bash
# Backend
cargo build --release

# Frontend
cd frontend
npm install
npm run build
cd ..

# Run
CONFIG_DIR=./config RUN_MODE=production ./target/release/oak-maillist
```

The service will be available at `http://localhost:3000`.

### Docker Compose (One-Command)

```bash
docker compose up -d
```

See the full [Deployment Guide](docs/DEPLOYMENT.md) for production setup with TLS, Nginx, and systemd.

---

## Configuration

Configuration uses a layered system:

1. `config/default.toml` — base settings
2. `config/{RUN_MODE}.toml` — environment overrides
3. Environment variables (`OAK__*` prefix)

### Example: Override via Environment

```bash
export OAK__DATABASE__URL="postgres://oak:pass@db:5432/oak_maillist"
export OAK__SECURITY__JWT_SECRET="$(openssl rand -hex 32)"
export OAK__SECURITY__WEBAUTHN_RP_ID="mail.example.com"
export OAK__SMTP__OUTGOING__HOST="smtp.example.com"
export OAK__SMTP__OUTGOING__USERNAME="noreply@example.com"
export OAK__SMTP__OUTGOING__PASSWORD="app-password"
export OAK__AI_MODERATION__ENABLED="true"
export OAK__AI_MODERATION__ACCESS_KEY_ID="your-key"
export OAK__AI_MODERATION__ACCESS_KEY_SECRET="your-secret"
export OAK__LOGGING__FORMAT="json"
```

### Full Config File Example

```toml
[server]
host = "0.0.0.0"
port = 3000
base_url = "https://mail.example.com"

[database]
url = "postgres://oak:password@localhost:5432/oak_maillist"
max_connections = 20
min_connections = 5
connect_timeout = 10
idle_timeout = 300

[security]
jwt_secret = "your-cryptographically-random-secret"
jwt_expiration_seconds = 900
refresh_token_expiration_days = 7
session_token_expiration_seconds = 600
password_min_length = 8
webauthn_rp_id = "mail.example.com"

[smtp.incoming]
enabled = true
host = "0.0.0.0"
port = 2525

[smtp.outgoing]
host = "smtp.example.com"
port = 587
username = "noreply@example.com"
password = "app-password"
from_address = "noreply@example.com"

[ai_moderation]
enabled = true
provider = "aliyun"
access_key_id = ""
access_key_secret = ""
region = "cn-shanghai"
service = "ugc_moderation_byllm"
endpoint = "https://green-cip.cn-shanghai.aliyuncs.com"
high_risk_threshold = 80
medium_risk_threshold = 50
request_timeout_seconds = 30
max_text_length = 2000

[archive]
enabled = true
storage_path = "./storage/archives"
max_attachment_size_mb = 10

[logging]
level = "info"
format = "json"
```

---

## API Overview

All API routes are prefixed with `/api/v1`.

### Authentication

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/auth/register` | Register new user |
| POST | `/auth/login` | Login (returns tokens or MFA challenge) |
| POST | `/auth/refresh` | Refresh access token |
| POST | `/auth/mfa/totp/verify` | Verify TOTP code |
| POST | `/auth/passkey/auth-options` | Start Passkey login |
| POST | `/auth/passkey/login` | Complete Passkey login |

### Lists

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/lists` | List all lists |
| POST | `/lists` | Create a list |
| GET | `/lists/{id}` | Get list details |
| PUT | `/lists/{id}` | Update list |
| DELETE | `/lists/{id}` | Deactivate list |

### Subscribers

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/lists/{id}/subscribe` | Public subscribe |
| GET | `/lists/{id}/subscribers` | List subscribers |
| POST | `/lists/{id}/subscribers/import` | Bulk import |
| GET | `/lists/{id}/subscribers/export` | Export CSV |

### Messages & Archive

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/lists/{id}/messages` | List archived messages |
| GET | `/lists/{id}/messages/{msg_id}` | Get message details |
| GET | `/lists/{id}/threads` | List threads |
| GET | `/lists/{id}/search` | Search archive |

### Moderation

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/lists/{id}/moderation` | Moderation queue |
| POST | `/moderation/{id}/approve` | Approve message |
| POST | `/moderation/{id}/reject` | Reject message |
| POST | `/moderation/{id}/discard` | Discard message |

### Health & Metrics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health/live` | Liveness probe |
| GET | `/health/ready` | Readiness probe |
| GET | `/metrics` | Prometheus metrics |

See the source code in `src/api/v1/` for the complete endpoint list.

---

## Frontend

The frontend is a Vue 3 SPA served statically by the backend.

### Pages

- **Login/Register** — Email/password and Passkey login
- **Dashboard** — Stats overview and pending moderation
- **Lists** — Create, manage, and configure mailing lists
- **Domains** — Domain and DKIM management
- **Moderation** — Review AI-flagged and manually held messages
- **Templates** — Edit email templates with live preview
- **Users** — User administration
- **Profile** — Update profile, manage TOTP, register Passkeys
- **Settings** — View and edit system settings

### Development

```bash
cd frontend
npm install
npm run dev        # Vite dev server with proxy to backend
npm run test       # Run Vitest tests
npm run build      # Production build
```

---

## Development

### Backend Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test --test integration_auth_service_test

# Run with coverage
cargo tarpaulin --out Html
```

### Database Migrations

```bash
cd migration
cargo run -- up     # Apply migrations
cargo run -- down   # Rollback one migration
```

### Code Structure

```
src/
├── api/           # Axum routers and middleware
├── models/        # SeaORM entities and AppState
├── services/      # Business logic (auth, list, subscriber, etc.)
├── smtp/          # SMTP server, client, parser, processor
├── tasks/         # Background tasks (digest, bounce, cleanup, AI)
├── ai/            # AI moderation client and policy engine
├── utils/         # Validation, crypto, email helpers
└── config.rs      # Configuration types and loader

frontend/
├── src/
│   ├── api/       # Axios API clients
│   ├── components/# Vue components (Layout, LangSwitch, CookieConsent)
│   ├── views/     # Page views
│   ├── i18n/      # vue-i18n setup and locale files
│   └── router/    # Vue Router configuration
└── src/__tests__/ # Vitest tests
```

---

## Deployment

See the detailed [Deployment Guide](docs/DEPLOYMENT.md) for:

- System requirements and resource recommendations
- Native binary deployment with systemd
- Docker and Docker Compose setup
- Database setup (SQLite, PostgreSQL, MySQL)
- Nginx reverse proxy with TLS
- Environment variable configuration
- Health checks and monitoring
- Upgrade procedures

中文版部署文档请见：[部署指南](docs/zh/DEPLOYMENT.md)

---

## License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

You are free to use, modify, and distribute this software under the terms of the AGPL-3.0. If you run a modified version of this software as a network service, you must make your source code available to the users of that service.

See [LICENSE](LICENSE) or https://www.gnu.org/licenses/agpl-3.0.html for the full license text.

---

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

For major changes, please open an issue first to discuss what you would like to change.
