# Oak MailList

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

一个基于 Rust 和 Vue 3 构建的现代化自托管邮件列表服务。支持多因素认证、Passkey 登录、AI 智能内容审核，以及带主题串功能的完整邮件归档。

🌐 [English README](README.md) | 📖 [部署指南](docs/zh/DEPLOYMENT.md)

---

## 功能特性

### 核心邮件列表

- **多列表管理**：在自定义域名下创建和管理多个邮件列表
- **订阅者管理**：公开或邀请制订阅、批量导入/导出（CSV）、摘要模式（无/每日/每周）
- **邮件归档**：完整 RFC 822 原始存储，支持文本/HTML 提取、附件处理和通过 `In-Reply-To` / `References` 构建主题串
- **SMTP 服务器**：内置异步接收邮件 SMTP 服务器（端口 2525）
- **SMTP 中继**：通过可配置 SMTP 主机发送出站邮件，支持 DKIM
- **退信处理**：VERP 编码的回退路径，3 次硬退信后自动取消订阅
- **摘要生成**：每小时后台任务为订阅者编译每日/每周摘要邮件

### 安全与认证

- **密码认证**：Argon2id 哈希密码，支持配置最小长度
- **JWT 令牌**：短时效访问令牌（15 分钟）+ 可撤销刷新令牌（7 天）
- **TOTP 多因素认证**：基于时间的一次性密码，兼容 Google/Microsoft Authenticator，附带 10 个备用码
- **Passkey / WebAuthn**：使用 `webauthn-rs` 进行 FIDO2 凭证注册和登录
- **会话管理**：查看和按设备撤销活跃会话

### 内容审核

- **人工审核**：批准、拒绝、丢弃、白名单或黑名单发送者
- **AI 智能审核**：集成阿里云内容安全（绿网）API 进行自动化风险评分
  - 主题和正文内容分析
  - 可配置风险阈值（`flagged` ≥ 80，`caution` ≥ 50）
  - AI 标记的邮件进入人工审核队列
  - 误判反馈机制
- **发送者策略**：按列表设置白名单/黑名单规则，支持邮件地址模式匹配

### 管理功能

- **仪表盘**：概览统计（用户、列表、订阅者、邮件、待审核数量）
- **域名管理**：配置带 SMTP 和 DKIM 设置的域名
- **邮件模板**：基于 Tera 引擎的模板系统，支持订阅确认、取消订阅、审核通知和欢迎邮件
- **用户管理**：用户的增删改查、角色分配、资料更新
- **活动日志**：通过 `tracing` 输出结构化日志，支持美观或 JSON 格式

### 前端

- **Vue 3 + Vite + Element Plus** 单页应用
- **国际化**：英文和中文，支持浏览器语言自动检测和按用户语言偏好设置
- **Cookie 同意**：符合 GDPR 的 Cookie 同意横幅
- **响应式界面**：侧边栏布局，包含列表管理、审核队列、邮件归档、用户资料和设置

### 可观测性

- **健康检查**：`/health/live`、`/health/ready`、`/health` 端点
- **Prometheus 指标**：`/metrics` 监控端点
- **结构化日志**：通过 `tracing` 配置日志级别和格式（美观 / JSON）

---

## 技术栈

| 层级 | 技术 |
|-------|------------|
| 后端 | Rust, Axum, Tokio |
| ORM | SeaORM（SQLite / MySQL / PostgreSQL） |
| 认证 | jsonwebtoken, argon2, totp-rs, webauthn-rs |
| 邮件 | lettre（SMTP）, mailparse（解析） |
| 模板 | Tera |
| 前端 | Vue 3, Vue Router, Axios, Element Plus, vue-i18n |
| AI | 阿里云内容安全 API |
| 指标 | metrics + metrics-exporter-prometheus |

---

## 快速开始

### 环境要求

- Rust 1.85+
- Node.js 20+（构建前端）
- PostgreSQL 14+ / MySQL 8.0+（或 SQLite 用于测试）

### 1. 克隆仓库

```bash
git clone --recursive https://github.com/your-org/oak-maillist.git
cd oak-maillist
```

### 2. 配置

```bash
cp config/default.toml config/production.toml
# 编辑 config/production.toml 设置你的参数
```

最低配置：

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

### 3. 构建并运行

```bash
# 后端
cargo build --release

# 前端
cd frontend
npm install
npm run build
cd ..

# 运行
CONFIG_DIR=./config RUN_MODE=production ./target/release/oak-maillist
```

服务将在 `http://localhost:3000` 可用。

### Docker Compose（一键启动）

```bash
docker compose up -d
```

完整的生产环境部署（含 TLS、Nginx、systemd）请参见 [部署指南](docs/zh/DEPLOYMENT.md)。

---

## 配置说明

配置采用分层系统：

1. `config/default.toml` — 基础设置
2. `config/{RUN_MODE}.toml` — 环境覆盖
3. 环境变量（`OAK__*` 前缀）

### 通过环境变量覆盖示例

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

### 完整配置文件示例

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

## API 概览

所有 API 路由以 `/api/v1` 为前缀。

### 认证

| 方法 | 端点 | 说明 |
|--------|----------|-------------|
| POST | `/auth/register` | 注册新用户 |
| POST | `/auth/login` | 登录（返回令牌或 MFA 挑战） |
| POST | `/auth/refresh` | 刷新访问令牌 |
| POST | `/auth/mfa/totp/verify` | 验证 TOTP 码 |
| POST | `/auth/passkey/auth-options` | 开始 Passkey 登录 |
| POST | `/auth/passkey/login` | 完成 Passkey 登录 |

### 列表

| 方法 | 端点 | 说明 |
|--------|----------|-------------|
| GET | `/lists` | 列出所有列表 |
| POST | `/lists` | 创建列表 |
| GET | `/lists/{id}` | 获取列表详情 |
| PUT | `/lists/{id}` | 更新列表 |
| DELETE | `/lists/{id}` | 停用列表 |

### 订阅者

| 方法 | 端点 | 说明 |
|--------|----------|-------------|
| POST | `/lists/{id}/subscribe` | 公开订阅 |
| GET | `/lists/{id}/subscribers` | 列出订阅者 |
| POST | `/lists/{id}/subscribers/import` | 批量导入 |
| GET | `/lists/{id}/subscribers/export` | 导出 CSV |

### 邮件与归档

| 方法 | 端点 | 说明 |
|--------|----------|-------------|
| GET | `/lists/{id}/messages` | 列出归档邮件 |
| GET | `/lists/{id}/messages/{msg_id}` | 获取邮件详情 |
| GET | `/lists/{id}/threads` | 列出主题串 |
| GET | `/lists/{id}/search` | 搜索归档 |

### 审核

| 方法 | 端点 | 说明 |
|--------|----------|-------------|
| GET | `/lists/{id}/moderation` | 审核队列 |
| POST | `/moderation/{id}/approve` | 批准邮件 |
| POST | `/moderation/{id}/reject` | 拒绝邮件 |
| POST | `/moderation/{id}/discard` | 丢弃邮件 |

### 健康检查与指标

| 方法 | 端点 | 说明 |
|--------|----------|-------------|
| GET | `/health/live` | 存活探针 |
| GET | `/health/ready` | 就绪探针 |
| GET | `/metrics` | Prometheus 指标 |

完整端点列表请查看 `src/api/v1/` 下的源代码。

---

## 前端

前端是由后端静态托管的 Vue 3 单页应用。

### 页面

- **登录/注册** — 邮箱/密码和 Passkey 登录
- **仪表盘** — 统计概览和待审核数量
- **列表** — 创建、管理和配置邮件列表
- **域名** — 域名和 DKIM 管理
- **审核** — 审查 AI 标记和人工拦截的邮件
- **模板** — 编辑邮件模板并实时预览
- **用户** — 用户管理
- **个人资料** — 更新资料、管理 TOTP、注册 Passkey
- **设置** — 查看和编辑系统设置

### 开发

```bash
cd frontend
npm install
npm run dev        # Vite 开发服务器，代理到后端
npm run test       # 运行 Vitest 测试
npm run build      # 生产构建
```

---

## 开发

### 后端测试

```bash
# 运行全部测试
cargo test

# 运行特定测试
cargo test --test integration_auth_service_test

# 运行覆盖率测试
cargo tarpaulin --out Html
```

### 数据库迁移

```bash
cd migration
cargo run -- up     # 应用迁移
cargo run -- down   # 回滚一次迁移
```

### 代码结构

```
src/
├── api/           # Axum 路由和中间件
├── models/        # SeaORM 实体和 AppState
├── services/      # 业务逻辑（认证、列表、订阅者等）
├── smtp/          # SMTP 服务器、客户端、解析器、处理器
├── tasks/         # 后台任务（摘要、退信、清理、AI）
├── ai/            # AI 审核客户端和策略引擎
├── utils/         # 验证、加密、邮件辅助函数
└── config.rs      # 配置类型和加载器

frontend/
├── src/
│   ├── api/       # Axios API 客户端
│   ├── components/# Vue 组件（布局、语言切换、Cookie 同意）
│   ├── views/     # 页面视图
│   ├── i18n/      # vue-i18n 配置和语言文件
│   └── router/    # Vue Router 配置
└── src/__tests__/ # Vitest 测试
```

---

## 部署

详细的部署文档请参见 [部署指南](docs/zh/DEPLOYMENT.md)，涵盖：

- 系统要求和资源配置建议
- 原生二进制部署与 systemd
- Docker 和 Docker Compose 配置
- 数据库设置（SQLite、PostgreSQL、MySQL）
- Nginx 反向代理与 TLS
- 环境变量配置
- 健康检查和监控
- 升级流程

English deployment guide: [Deployment Guide](docs/DEPLOYMENT.md)

---

## 开源协议

本项目采用 **GNU Affero General Public License v3.0 (AGPL-3.0)** 授权。

您可以依据 AGPL-3.0 的条款自由使用、修改和分发本软件。如果您将修改后的版本作为网络服务运行，必须向该服务的用户提供源代码。

完整协议文本请参见 [LICENSE](LICENSE) 或 https://www.gnu.org/licenses/agpl-3.0.html。

---

## 贡献

欢迎提交贡献！请提交 Issue 或 Pull Request。

对于重大变更，请先提交 Issue 讨论您希望进行的修改。
