# 部署指南

本文档介绍如何在生产环境中构建和部署 Oak MailList 邮件列表服务。

## 概述

Oak MailList 是一款基于 Rust 的邮件列表服务，包含以下组件：

- **后端**：Axum（HTTP API）+ SeaORM（数据库 ORM），支持 SQLite、MySQL 和 PostgreSQL
- **前端**：Vue 3 + Vite + Element Plus 单页应用
- **SMTP 服务器**：内置接收邮件的 SMTP 服务器
- **后台任务**：摘要生成、AI 审核、退信处理、定时清理

默认端口：

| 服务 | 端口 | 说明 |
|---------|------|-------------|
| HTTP API | 3000 | REST API 和静态文件服务 |
| SMTP | 2525 | 接收邮件服务器 |

---

## 系统要求

### 最低配置

- **操作系统**：Linux（glibc 2.31+）、macOS 12+、Windows Server 2019+
- **CPU**：1 核
- **内存**：512 MB RAM
- **磁盘**：1 GB（视归档存储需求而定）
- **Rust**：1.85+（如从源码构建）
- **Node.js**：20+（如从源码构建前端）

### 生产环境推荐配置

- **CPU**：2 核以上
- **内存**：2 GB RAM
- **数据库**：PostgreSQL 14+ 或 MySQL 8.0+
- **反向代理**：Nginx 或 Traefik（用于 TLS 终止和静态文件缓存）

---

## 配置说明

配置从 TOML 文件加载，也可通过环境变量覆盖。

### 配置文件

将配置文件放在目录中（默认：`./config`）：

```
config/
├── default.toml      # 基础配置（必需）
└── production.toml   # 环境覆盖（可选）
```

通过环境变量设置配置目录：

```bash
export CONFIG_DIR=/app/config
export RUN_MODE=production   # 如存在则加载 production.toml
```

### 环境变量覆盖

所有配置项都可以通过 `OAK__` 前缀和双下划线分隔符的环境变量覆盖：

```bash
# 数据库
export OAK__DATABASE__URL="postgres://user:pass@localhost:5432/oak_maillist"

# 安全
export OAK__SECURITY__JWT_SECRET="your-strong-secret-key-here"
export OAK__SECURITY__WEBAUTHN_RP_ID="mail.example.com"

# 服务器
export OAK__SERVER__BASE_URL="https://mail.example.com"

# SMTP
export OAK__SMTP__OUTGOING__HOST="smtp.example.com"
export OAK__SMTP__OUTGOING__USERNAME="noreply@example.com"
export OAK__SMTP__OUTGOING__PASSWORD="app-password"
export OAK__SMTP__OUTGOING__FROM_ADDRESS="noreply@example.com"

# AI 审核（可选）
export OAK__AI_MODERATION__ENABLED="true"
export OAK__AI_MODERATION__ACCESS_KEY_ID="your-key"
export OAK__AI_MODERATION__ACCESS_KEY_SECRET="your-secret"
```

### 关键生产环境配置

编辑 `config/production.toml` 或使用环境变量：

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
webauthn_rp_id = "mail.example.com"   # 使用 Passkey 认证时必需

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
format = "json"   # 生产环境使用 "json" 便于日志聚合
```

**安全清单：**
- [ ] 将 `jwt_secret` 改为密码学安全的随机字符串（至少 32 字节）
- [ ] 如使用 Passkey 认证，设置 `webauthn_rp_id` 为你的域名
- [ ] 使用强数据库密码
- [ ] 在反向代理层启用 TLS
- [ ] 使用防火墙规则限制 SMTP 接收端口的访问

---

## 从源码构建

### 1. 克隆仓库

```bash
git clone --recursive https://github.com/your-org/oak-maillist.git
cd oak-maillist
```

> **注意**：`frontend/` 目录是 Git 子模块。克隆时请使用 `--recursive`，或在克隆后运行 `git submodule update --init --recursive`。

### 2. 构建后端

```bash
# 安装 Rust（如未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建发布版本
cargo build --release

# 二进制文件将位于：
# target/release/oak-maillist   # 主服务
# target/release/migration      # 数据库迁移 CLI
```

### 3. 构建前端

```bash
cd frontend
npm install
npm run build

# 静态文件将输出到 frontend/dist/
cd ..
```

后端会在 `frontend/dist/` 存在时自动提供静态文件服务。

---

## 部署方式

### 方式一：原生部署

1. **准备目录：**

```bash
sudo mkdir -p /opt/oak-maillist/config
sudo mkdir -p /var/lib/oak-maillist/archives
sudo cp -r config/* /opt/oak-maillist/config/
sudo cp -r frontend/dist /opt/oak-maillist/
sudo cp target/release/oak-maillist /usr/local/bin/
```

2. **创建 systemd 服务**（`/etc/systemd/system/oak-maillist.service`）：

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

3. **启动服务：**

```bash
sudo useradd -r -s /bin/false oak
sudo systemctl daemon-reload
sudo systemctl enable --now oak-maillist
sudo systemctl status oak-maillist
```

### 方式二：Docker

1. **构建镜像：**

```bash
docker build -t oak-maillist:latest .
```

2. **运行容器：**

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

### 方式三：Docker Compose（推荐）

使用项目自带的 `docker-compose.yml`：

```bash
# 编辑 docker-compose.yml 设置你的环境变量
cp docker-compose.yml docker-compose.prod.yml
# 编辑 docker-compose.prod.yml

docker compose -f docker-compose.prod.yml up -d
```

Compose 文件包含：
- 带持久化卷的 PostgreSQL 数据库
- 带健康检查的自愈策略的 Oak MailList 应用
- 自动重启策略

---

## 数据库设置

### SQLite（开发/小型部署）

无需额外配置。应用会自动创建数据库文件。

```toml
[database]
url = "sqlite:///var/lib/oak-maillist/data.db?mode=rwc"
```

> **警告**：SQLite 不支持高并发写入。生产环境请使用 PostgreSQL 或 MySQL。

### PostgreSQL

```bash
# 创建数据库和用户
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

### 数据库迁移

迁移在启动时自动运行。如需手动执行：

```bash
# 使用迁移二进制文件
cd migration
cargo run -- up

# 或使用 sea-orm-cli
cargo install sea-orm-cli
sea-orm-cli migrate up
```

---

## 反向代理（Nginx）

Nginx 配置示例：

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

# HTTP 重定向到 HTTPS
server {
    listen 80;
    server_name mail.example.com;
    return 301 https://$server_name$request_uri;
}
```

**Passkey/WebAuthn 注意事项：**
配置中的 `base_url` 必须与 Nginx 暴露的 HTTPS URL 一致。`webauthn_rp_id` 必须设置为域名（如 `mail.example.com`）。

---

## 健康检查

服务暴露了存活探针：

```bash
curl http://localhost:3000/health/live
```

Docker Compose 和 Kubernetes 可使用此接口进行健康检查。

---

## 日志

### 美观格式（开发环境）

```toml
[logging]
level = "debug"
format = "pretty"
```

### JSON 格式（生产环境）

```toml
[logging]
level = "info"
format = "json"
```

日志包含结构化字段，兼容 ELK/Loki/Grafana 等日志聚合系统。

---

## 升级

1. **备份数据库：**

```bash
pg_dump -U oak oak_maillist > backup_$(date +%Y%m%d).sql
```

2. **拉取新代码并重新构建：**

```bash
git pull
git submodule update --init --recursive
cargo build --release
cd frontend && npm install && npm run build && cd ..
```

3. **重启服务：**

```bash
sudo systemctl restart oak-maillist
```

迁移会在启动时自动运行。

---

## 故障排查

| 问题 | 解决方案 |
|-------|----------|
| SMTP 端口连接被拒绝 | 检查防火墙规则；确保 `smtp.incoming.enabled = true` |
| Passkey 注册失败 | 确认 `base_url` 和 `webauthn_rp_id` 与 HTTPS 域名一致 |
| 前端显示 404 | 确保 `frontend/dist/` 存在且包含 `index.html` |
| 数据库连接错误 | 检查 `database.url` 和网络连通性 |
| 邮件发送失败 | 验证 SMTP 外发凭据；查看日志中的 lettre 错误 |
| 内存占用过高 | 减小 `database.max_connections`；启用归档清理 |

---

## 参考

- [Oak MailList 仓库](https://github.com/your-org/oak-maillist)
- [SeaORM 文档](https://www.sea-ql.org/SeaORM/)
- [Axum 文档](https://docs.rs/axum/latest/axum/)
- [Vue 3 文档](https://vuejs.org/)
