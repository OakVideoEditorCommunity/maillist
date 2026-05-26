#!/bin/bash
set -e

echo "🌳 Setting up Oak MailList dev environment..."

# Ensure git submodules are initialized
git submodule update --init --recursive

# Install frontend dependencies
if [ -f "frontend/package.json" ]; then
    echo "📦 Installing frontend dependencies..."
    cd frontend
    npm install
    cd ..
fi

# Create local config override if not exists
if [ ! -f "config/development.toml" ]; then
    echo "⚙️  Creating config/development.toml..."
    cat > config/development.toml << 'EOF'
[server]
host = "0.0.0.0"
port = 3000
base_url = "http://localhost:3000"

[database]
url = "postgres://oak:oak_secret@db:5432/oak_maillist"
max_connections = 10
min_connections = 2

[security]
jwt_secret = "dev-secret-change-me"
jwt_expiration_seconds = 3600

[smtp.incoming]
enabled = true
host = "0.0.0.0"
port = 2525

[smtp.outgoing]
host = "mailpit"
port = 1025
from_address = "noreply@example.com"

[ai_moderation]
enabled = false

[logging]
level = "debug"
format = "pretty"
EOF
fi

echo ""
echo "✅ Dev environment ready!"
echo ""
echo "Quick start:"
echo "  1. Backend:  cargo run"
echo "  2. Frontend: cd frontend && npm run dev"
echo "  3. Tests:    cargo test && cd frontend && npm run test"
echo "  4. Mailpit:  http://localhost:8025"
echo ""
echo "Auto-create admin on first run:"
echo "  OAK_INIT_ADMIN_EMAIL=admin@example.com OAK_INIT_ADMIN_PASSWORD=pass123 cargo run"
