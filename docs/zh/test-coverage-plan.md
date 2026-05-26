# 测试全覆盖实施计划

> 目标：将项目测试覆盖率从 15.45% 提升至 **≥90%**
> 策略：单元测试 + 服务集成测试 + API 端到端测试（E2E）分层覆盖

---

## 一、测试基础设施改进

### 1.1 提取公共测试辅助模块

**新建 `tests/common/mod.rs`**，将现有 5 个测试文件中重复出现的 `setup_db()` 和 `AppConfig` 构造提取为公共辅助函数：

```rust
pub async fn setup_db() -> AppState { ... }
pub fn test_config() -> AppConfig { ... }
pub async fn setup_app() -> Router { ... }  // 创建带内存数据库的路由器
```

### 1.2 引入 API 测试辅助

为所有 API 端到端测试提供统一的请求构造辅助：

```rust
pub async fn api_get(app: &Router, path: &str, token: Option<&str>) -> Response;
pub async fn api_post(app: &Router, path: &str, body: Value, token: Option<&str>) -> Response;
pub async fn api_put(app: &Router, path: &str, body: Value, token: Option<&str>) -> Response;
pub async fn api_delete(app: &Router, path: &str, token: Option<&str>) -> Response;
pub async fn register_user(app: &Router, email: &str, password: &str) -> (User, String); // 返回用户和 JWT
```

### 1.3 引入 Mock 辅助

- **AI HTTP Mock**：使用 `wiremock` 或 `mockito` 模拟阿里云内容审核 API 响应
- **SMTP Mock**：对 SMTP 发送逻辑，通过注入 `SmtpClient` trait 实现或使用 `lettre` 的 `FileTransport` 做测试替身

---

## 二、实施阶段与优先级

### Phase 1：单元测试 + 工具函数全覆盖（预计新增 ~80 个测试）

**目标模块**：`utils`、`ai/policy`、`ai/parser`、`ai/aliyun_signer`、`models`、`smtp/verp`、`smtp/parser`、`config`

#### 1. `src/utils/crypto.rs`（已有 3 个测试）
- [ ] `hash_password` 错误处理：空密码边界
- [ ] `verify_password` 对非法 hash 格式的处理

#### 2. `src/utils/email.rs`（已有 4 个测试）
- [ ] `extract_local_part` 空字符串、多 `@` 符号
- [ ] `extract_domain` 空字符串、无 `@`、多 `@`
- [ ] `build_list_email` 边界空值
- [ ] `build_verp_address` 特殊字符 token

#### 3. `src/utils/validation.rs`（当前 0%）
- [ ] `is_valid_email`：标准邮箱、含 `+` 的邮箱、无 `@`、空字符串、超长（>254）、国际化邮箱
- [ ] `is_valid_domain`：标准域名、子域名、含端口（应拒绝）、IP 地址（应拒绝）、空字符串、超长（>253）
- [ ] `normalize_email`：大写转小写、前后空格去除

#### 4. `src/utils/response.rs`（当前 20%）
- [ ] `ApiError` 各错误码到 HTTP 状态码的映射（`VALIDATION_ERROR→400`、`UNAUTHORIZED→401`、`FORBIDDEN→403`、`NOT_FOUND→404`、`CONFLICT→409`、`RATE_LIMITED→429`、`AI_MODERATION_ERROR→503`、默认→500）
- [ ] `ApiResponse` 序列化输出格式

#### 5. `src/config.rs`（已有 93%）
- [ ] `AppConfig::load` 在 `CONFIG_DIR` 指向不存在的目录时的回退行为
- [ ] 环境变量 `OAK__SERVER__PORT` 覆盖 toml 配置

#### 6. `src/ai/policy.rs`（当前 0%）
- [ ] `verdict` 边界值：`high_risk_threshold` 及 `medium_risk_threshold` 精确边界
- [ ] 低于 `medium` → `"clean"`
- [ ] 等于 `medium` → `"caution"`
- [ ] 等于 `high` → `"flagged"`

#### 7. `src/ai/parser.rs`（当前 0%）
- [ ] `parse_ai_response` 返回默认值（当前为 stub）

#### 8. `src/ai/aliyun_signer.rs`（当前 0%）
- [ ] `sign_request` 对空 body、带 query、带额外 header 的不同输入生成正确签名
- [ ] 签名结果包含 `Authorization` 和 `x-acs-date` 等必需头

#### 9. `src/smtp/verp.rs`（当前 0%）
- [ ] `encode` 正常编解码
- [ ] `decode` 正常解析
- [ ] `decode` 非法地址（无 `@`、无 `-bounces+`）返回 `None`
- [ ] encode/decode 往返一致性

#### 10. `src/smtp/parser.rs`（当前 0%）
- [ ] `EmailParser::parse` 标准 MIME 邮件
- [ ] 多部分邮件（text + html + attachments）
- [ ] 非法原始数据返回 Err

#### 11. `src/models/*`（SeaORM 实体）
- [ ] 每个 ActiveModel 的字段类型约束（如 UUID 唯一性、外键约束）通过集成测试验证

---

### Phase 2：服务层补充测试（预计新增 ~60 个测试）

**目标模块**：`services/*`、`tasks/bounce`、`tasks/digest`

#### 1. `src/services/auth_service.rs`（已有 83%）
- [ ] `register` 重复邮箱 → `Conflict` 错误
- [ ] `register` 密码长度不足（由 config 驱动）
- [ ] `login` 不存在的邮箱 → `Unauthorized`
- [ ] `login` 用户 `is_active=false` → `Unauthorized`
- [ ] `generate_access_token` / `verify_access_token` 过期令牌 → `Unauthorized`
- [ ] `verify_access_token` 非法签名 → `Unauthorized`
- [ ] `create_refresh_token` → 生成非空字符串
- [ ] `verify_refresh_token` 过期令牌 → `Unauthorized`
- [ ] `verify_refresh_token` 已撤销令牌 → `Unauthorized`
- [ ] `revoke_all_user_tokens` 后验证所有令牌失效

#### 2. `src/services/mfa_service.rs`（已有 89%）
- [ ] `setup_totp` 重复设置（用户已有 credential）→ 替换或报错
- [ ] `verify_totp_setup` 错误验证码 → 失败
- [ ] `verify_totp` 错误验证码 → false
- [ ] `disable_totp` 错误验证码 → 失败
- [ ] `regenerate_backup_codes` 生成 10 个唯一码
- [ ] `get_backup_codes_count` 无 credential 时返回 0

#### 3. `src/services/domain_service.rs`（已有 88%）
- [ ] `find_by_id` 不存在的 UUID → `Ok(None)`
- [ ] `update` 不存在的 UUID → `NotFound`
- [ ] `update` 只更新部分字段，其他字段保持不变
- [ ] `delete` 不存在的 UUID → `NotFound`

#### 4. `src/services/list_service.rs`（当前 36%）
- [ ] `create` 正常创建，默认值验证（`visibility=public`, `subscription_policy=confirm` 等）
- [ ] `create` 无效 domain_id → `NotFound`
- [ ] `list_public` 分页：page=1, per_page=20 默认值
- [ ] `list_public` 只返回 `visibility=public` 且 `is_active=true`
- [ ] `list_public` 不返回已软删除的列表
- [ ] `update` 部分字段更新（如只改 `display_name`）
- [ ] `delete` 软删除（`is_active=false`）而非硬删除

#### 5. `src/services/subscriber_service.rs`（已有 73%）
- [ ] `subscribe` 已存在 active 订阅者 → `Conflict`
- [ ] `subscribe` 已存在 pending 订阅者 → 返回现有记录
- [ ] `subscribe` 无效 list_id → `NotFound`
- [ ] `confirm` 无效 token → `NotFound`
- [ ] `confirm` 已 active 的订阅者 → 正常返回或忽略
- [ ] `unsubscribe` 无效 token → `NotFound`
- [ ] `list_by_list` 分页
- [ ] `update_digest_mode` 无效 subscriber_id → `NotFound`
- [ ] `update_digest_mode` 无效 mode 值 → `ValidationError`

#### 6. `src/services/moderation_service.rs`（当前 36%）
- [ ] `approve` 不存在 moderation_id → `NotFound`
- [ ] `reject` 不存在 moderation_id → `NotFound`
- [ ] `discard` 不存在 moderation_id → `NotFound`
- [ ] `whitelist_sender` 成功后创建 `sender_policy` whitelist 记录
- [ ] `blacklist_sender` 成功后创建 `sender_policy` blacklist 记录
- [ ] 重复 whitelist/blacklist 不报错（幂等）

#### 7. `src/services/template_service.rs`（当前 0%）
- [ ] `render_template` 模板不存在 → `NotFound`
- [ ] `render_template` 只有 html 无 text → 自动 fallback
- [ ] `render_template` 变量渲染（Tera 语法）
- [ ] `send_templated_email` SMTP 未配置（host 为空）→ 跳过不报错
- [ ] `send_templated_email` 正常发送（使用 FileTransport mock）

#### 8. `src/services/notification_service.rs`（当前 0%）
- [ ] `send_subscription_confirm` 调用 template_service 正确渲染
- [ ] `send_welcome` 正确渲染
- [ ] SMTP 未配置时跳过

#### 9. `src/services/archive_service.rs`（当前 0%）
- [ ] `build_threads` 无回复头消息 → 以自身 id 为 thread_id
- [ ] `build_threads` 有 `In-Reply-To` → 找到父消息 thread_id
- [ ] `build_threads` 有 `References` → 使用第一个引用
- [ ] `search` 关键词匹配 subject/body_text/from_addr
- [ ] `search` 结果不超过 100 条
- [ ] `search` `from` 参数过滤
- [ ] `get_thread_messages` 按 received_at 升序
- [ ] `get_thread_messages` 排除 `is_deleted=true`

#### 10. `src/services/ai_service.rs`（当前 0%）
- [ ] `moderate_email` enabled=false 时返回 clean
- [ ] `should_flag` 边界值（等于 high_risk_threshold）
- [ ] `should_caution` 边界值

#### 11. `src/tasks/bounce.rs`（当前 0%）
- [ ] `process_bounce` 未知 token → `NotFound`
- [ ] hard bounce 第 1 次 → bounce_count=1，不取消订阅
- [ ] hard bounce 第 3 次 → 取消订阅（status=unsubscribed）
- [ ] soft bounce → 只增加计数
- [ ] 未知 bounce_type → 作为 soft 处理

#### 12. `src/tasks/digest.rs`（当前 0%）
- [ ] `run` 无 digest 订阅者 → 空操作
- [ ] `run` daily 模式订阅者获取 24h 内消息
- [ ] `run` weekly 模式订阅者获取 7d 内消息
- [ ] SMTP 未配置 → 跳过发送
- [ ] 无新消息 → 不发送空摘要

---

### Phase 3：API 端到端测试（预计新增 ~120 个测试）

**目标模块**：`api/v1/*`、`api/middleware/*`、`main.rs`、`health`

使用 `axum::Router` + `tower::ServiceExt::oneshot` 或 `axum::serve` 做 HTTP 级别的 E2E 测试。

#### 1. `src/api/middleware/auth.rs`（当前 0%）
- [ ] `require_auth` 无 Authorization header → 401
- [ ] `require_auth` 非法 Bearer 格式 → 401
- [ ] `require_auth` 过期 JWT → 401
- [ ] `require_auth` 有效 JWT → 正常通过，Claims 注入 extensions
- [ ] `require_admin` 非 admin role → 403
- [ ] `require_admin` admin role → 正常通过
- [ ] `optional_auth` 无 token → 正常通过，无 Claims
- [ ] `optional_auth` 有效 token → 注入 Claims
- [ ] `optional_auth` 无效 token → 正常通过，无 Claims

#### 2. `src/api/middleware/error.rs`（当前 0%）
- [ ] 正常响应不受影响
- [ ] 5xx 错误触发日志记录

#### 3. `src/api/v1/auth.rs`（当前 0%，约 394 行）
- [ ] `POST /register` 成功注册
- [ ] `POST /register` 重复邮箱 → 409
- [ ] `POST /register` 密码过短 → 400
- [ ] `POST /login` 成功登录
- [ ] `POST /login` 错误密码 → 401
- [ ] `POST /login` 不存在用户 → 401
- [ ] `POST /login` MFA 已启用 → 403 MFA_REQUIRED
- [ ] `POST /logout` 有效 token → 200
- [ ] `POST /logout` 无 token → 401
- [ ] `POST /logout-all` 撤销所有刷新令牌
- [ ] `POST /refresh` 有效 refresh token → 新 access token
- [ ] `POST /refresh` 无效 refresh token → 401
- [ ] `POST /forgot-password` stub → 返回占位信息
- [ ] `POST /reset-password` stub → 返回占位信息
- [ ] `POST /magic-link` stub → 返回占位信息
- [ ] `GET /magic-link/callback` stub → 返回占位信息
- [ ] `POST /mfa/totp/setup` 成功生成 secret
- [ ] `POST /mfa/totp/setup` 未登录 → 401
- [ ] `POST /mfa/totp/verify-setup` 正确 code → 启用 MFA
- [ ] `POST /mfa/totp/verify-setup` 错误 code → 401
- [ ] `POST /mfa/totp/verify` 正确 code → true
- [ ] `POST /mfa/totp/verify` 错误 code → false
- [ ] `POST /mfa/totp/disable` 正确 code → 禁用 MFA
- [ ] `POST /mfa/totp/disable` 错误 code → 401
- [ ] `POST /mfa/totp/regenerate-backup-codes` → 10 个码
- [ ] `GET /mfa/totp/backup-codes` → 数量正确
- [ ] Passkey 相关路由 stub → NOT_IMPLEMENTED

#### 4. `src/api/v1/users.rs`（当前 0%，约 439 行）
- [ ] `GET /me` 获取当前用户信息
- [ ] `PUT /me` 更新用户名
- [ ] `PUT /me/password` 正确旧密码 → 更新成功
- [ ] `PUT /me/password` 错误旧密码 → 403
- [ ] `PUT /me/password` 新密码过短 → 400
- [ ] `GET /me/sessions` 列出会话
- [ ] `DELETE /me/sessions/{id}` 删除自己的会话 → 204
- [ ] `DELETE /me/sessions/{id}` 删除他人会话 → 403
- [ ] `GET /me/mfa` 获取 MFA 状态
- [ ] `GET /me/passkeys` 列出 passkeys
- [ ] `DELETE /me/passkeys/{id}` 删除自己的 → 204
- [ ] `DELETE /me/passkeys/{id}` 删除他人的 → 403
- [ ] `PUT /me/passkeys/{id}` 重命名自己的 → 200
- [ ] `GET /` 用户列表（admin）
- [ ] `GET /{id}` 获取指定用户
- [ ] `PUT /{id}` admin 更新其他用户
- [ ] `DELETE /{id}` admin 删除用户

#### 5. `src/api/v1/domains.rs`（当前 0%，约 80 行）
- [ ] `GET /` 域名列表
- [ ] `POST /` 创建域名（name 必填）
- [ ] `POST /` name 为空 → 400
- [ ] `GET /{id}` 获取域名
- [ ] `GET /{id}` 无效 UUID → 400
- [ ] `GET /{id}` 不存在 → 404
- [ ] `PUT /{id}` 更新域名
- [ ] `DELETE /{id}` 删除域名
- [ ] `POST /{id}/verify-dkim` stub → 返回占位信息

#### 6. `src/api/v1/lists.rs`（当前 0%，约 758 行，最大模块）
- [ ] `GET /` 公共列表分页
- [ ] `GET /` per_page 上限 100
- [ ] `POST /` 创建列表（需登录）
- [ ] `GET /{id}` 获取列表详情
- [ ] `PUT /{id}` 更新列表设置
- [ ] `DELETE /{id}` 删除列表
- [ ] `GET /{id}/archive` 归档消息
- [ ] `GET /{id}/archive/threads` 主题串
- [ ] `GET /{id}/archive/search` 搜索
- [ ] `GET /{id}/stats` 统计
- [ ] `GET /{id}/settings` 设置
- [ ] `PUT /{id}/settings` 更新设置
- [ ] `POST /{id}/subscribe` 公开订阅
- [ ] `POST /{id}/subscribe` 已订阅 → 409
- [ ] `GET /{id}/confirm` 确认订阅
- [ ] `GET /{id}/confirm` 无效 token → 400
- [ ] `GET /{id}/unsubscribe` 退订
- [ ] `GET /{id}/subscribers` 订阅者列表（需权限）
- [ ] `POST /{id}/subscribers/import` 批量导入
- [ ] `POST /{id}/subscribers/bulk-update` 批量更新
- [ ] `GET /{id}/messages` 消息列表
- [ ] `GET /{id}/messages/{msg_id}` 单条消息
- [ ] `GET /{id}/messages/{msg_id}/raw` 原始内容
- [ ] `GET /{id}/messages/{msg_id}/attachments` 附件
- [ ] `DELETE /{id}/messages/{msg_id}` 软删除（需 auth）
- [ ] `GET /{id}/moderation-queue` 审核队列
- [ ] `GET /{id}/policies` 发送策略

#### 7. `src/api/v1/moderation.rs`（当前 0%，约 101 行）
- [ ] `GET /{id}` 获取审核项
- [ ] `POST /{id}/approve` 批准（需 auth）
- [ ] `POST /{id}/reject` 拒绝（需 auth）
- [ ] `POST /{id}/discard` 丢弃（需 auth）
- [ ] `POST /{id}/whitelist-sender` 白名单（需 auth）
- [ ] `POST /{id}/blacklist-sender` 黑名单（需 auth）
- [ ] `POST /{id}/ai-feedback` stub → 返回占位信息
- [ ] 无权限 → 401
- [ ] 不存在 id → 404

#### 8. `src/api/v1/templates.rs`（当前 0%，约 92 行）
- [ ] `GET /` 模板列表
- [ ] `GET /{name}` 获取模板
- [ ] `GET /{name}` 不存在 → 404
- [ ] `PUT /{name}` 更新模板
- [ ] `POST /{name}/preview` 预览渲染

#### 9. `src/api/v1/admin.rs`（当前 0%，约 88 行）
- [ ] `GET /dashboard` 统计数据
- [ ] `GET /stats` 统计
- [ ] `GET /activity-log` stub
- [ ] `GET /settings` 获取设置
- [ ] `PUT /settings` 更新设置（stub）
- [ ] `GET /ai-moderation/stats` AI 审核统计
- [ ] 非 admin → 403

#### 10. `src/api/v1/health.rs`（当前 0%，约 22 行）
- [ ] `GET /health` DB 正常 → healthy
- [ ] `GET /health/ready` DB 正常 → ready
- [ ] `GET /health/live` → alive
- [ ] `GET /metrics` → 返回 prometheus 格式（stub 检查）

#### 11. `src/api/v1/subscribers.rs` 和 `messages.rs`
- [ ] 验证路由为空（stub）→ 确保返回 404 或正确占位

#### 12. `src/main.rs`（当前 0%）
- [ ] `init_tracing` 各 log level 映射（trace/debug/info/warn/error/默认）
- [ ] `main` 函数启动流程通过集成测试间接覆盖（E2E 测试启动应用）

---

### Phase 4：SMTP 层测试（预计新增 ~40 个测试）

**目标模块**：`smtp/server.rs`、`smtp/processor.rs`、`smtp/auth_check.rs`、`smtp/client.rs`

#### 1. `src/smtp/server.rs`（当前 0%）
- [ ] SMTP 会话完整流程：EHLO → MAIL → RCPT → DATA → QUIT
- [ ] HELO 替代 EHLO
- [ ] MAIL FROM 语法错误 → 501
- [ ] RCPT TO 语法错误 → 501
- [ ] DATA 后的 dot-stuffing（`..` → `.`）
- [ ] RSET 重置状态
- [ ] NOOP 返回 250
- [ ] 未知命令 → 500
- [ ] 连接断开（EOF）处理
- [ ] 邮件处理失败 → 451
- [ ] 邮件处理成功 → 250

#### 2. `src/smtp/processor.rs`（当前 0%，328 行，核心逻辑）
- [ ] `resolve_list` 有效 local part → 找到列表
- [ ] `resolve_list` 无效 local part → None
- [ ] `resolve_list` 列表 inactive → 拒绝
- [ ] `check_subscriber` 订阅者 → true
- [ ] `check_subscriber` 非订阅者 + subscriber_only → false
- [ ] `check_subscriber` 非订阅者 + open → true
- [ ] AI moderation enabled + flagged → 进入 moderation_queue
- [ ] AI moderation enabled + clean → 保存并投递
- [ ] AI moderation disabled → 直接保存并投递
- [ ] AI service 失败 → fallback 到 clean 处理
- [ ] `get_body_text` 纯文本邮件
- [ ] `get_body_text` HTML 邮件提取 text
- [ ] `get_body_html` 提取 html
- [ ] `deliver_to_subscribers` 跳过 digest 用户
- [ ] `deliver_to_subscribers` SMTP 未配置 → 跳过
- [ ] Thread 构建：有 In-Reply-To → 关联父线程
- [ ] Thread 构建：无 In-Reply-To → 新线程

#### 3. `src/smtp/auth_check.rs`（当前 0%）
- [ ] `check` stub 返回全 None
- [ ] `is_suspicious` SPF Fail → true
- [ ] `is_suspicious` DKIM Fail → true
- [ ] `is_suspicious` DMARC Fail → true
- [ ] `is_suspicious` 全 Pass → false

#### 4. `src/smtp/client.rs`（当前 0%）
- [ ] `SmtpClient::new` 正常构建
- [ ] `send` 正常发送（使用 stub transport 或 FileTransport）

---

### Phase 5：AI 模块集成测试（预计新增 ~15 个测试）

**目标模块**：`ai/client.rs` + 各 tasks 边界

#### 1. `src/ai/client.rs`（当前 0%）
- [ ] `new` 正常构建
- [ ] `analyze` enabled=false → 返回 clean 默认结果
- [ ] `analyze` enabled=true + mock 阿里云响应 high risk → flagged
- [ ] `analyze` enabled=true + mock 阿里云响应 medium risk → caution
- [ ] `analyze` enabled=true + mock 阿里云响应 low risk → clean
- [ ] `analyze` HTTP 非 2xx → 返回错误
- [ ] `analyze` 阿里云 code != 200 → 返回错误
- [ ] `analyze` 文本超长截断到 max_text_length
- [ ] `analyze` 空文本 → 返回 clean

#### 2. `src/tasks/ai_moderate.rs`（已有 94%）
- [ ] 无可审核项 → 空操作
- [ ] 已全部审核 → 空操作

#### 3. `src/tasks/deliver.rs`（已有 93%）
- [ ] 无 approved 待投递项 → 空操作
- [ ] 已投递项（已有 message_id）→ 跳过

#### 4. `src/tasks/cleanup.rs`（已有 100%）
- [ ] 刚好 30 天前的记录 → 不删除（边界）
- [ ] 30 天零 1 秒前的记录 → 删除

---

## 三、测试文件组织建议

```
tests/
├── common/
│   ├── mod.rs          # setup_db, test_config, api helpers
│   └── mocks.rs        # wiremock / FileTransport 辅助
├── unit/               # 单元测试（不依赖数据库）
│   ├── utils_test.rs   # crypto, email, validation, response, verp, policy
│   ├── ai_test.rs      # signer, parser, policy, client (mock)
│   └── smtp_unit_test.rs  # parser, verp, auth_check
├── integration/        # 服务层集成测试
│   ├── auth_service_test.rs
│   ├── mfa_service_test.rs
│   ├── domain_service_test.rs
│   ├── list_service_test.rs
│   ├── subscriber_service_test.rs
│   ├── moderation_service_test.rs
│   ├── template_service_test.rs
│   ├── archive_service_test.rs
│   ├── notification_service_test.rs
│   ├── ai_service_test.rs
│   ├── bounce_task_test.rs
│   └── digest_task_test.rs
├── api/                # API E2E 测试
│   ├── auth_api_test.rs
│   ├── users_api_test.rs
│   ├── domains_api_test.rs
│   ├── lists_api_test.rs
│   ├── moderation_api_test.rs
│   ├── templates_api_test.rs
│   ├── admin_api_test.rs
│   └── health_api_test.rs
└── smtp/               # SMTP 测试
    ├── server_test.rs
    └── processor_test.rs
```

> **兼容性说明**：为保持向后兼容，现有的 5 个 `tests/*.rs` 文件保留不动，新增测试按上述目录组织。后续可逐步将旧测试迁移到 `tests/integration/`。

---

## 四、关键实施决策

### 4.1 数据库策略
- 继续使用 `sqlite::memory:` 作为测试数据库
- 所有需要数据库的测试标记为 `async` + `#[tokio::test]`
- 每个测试独立运行，利用内存数据库的隔离性

### 4.2 Mock 策略
- **HTTP Mock**：引入 `wiremock`（dev-dependency），在 `tests/common/mocks.rs` 中提供 `start_ai_mock_server()` 辅助
- **SMTP Mock**：对 `template_service.rs` 中的发送逻辑，通过 `lettre::FileTransport` 或自定义 trait 注入测试替身
- **时间 Mock**：对 TOTP 测试，使用固定时间戳或 `totp_rs` 的测试模式

### 4.3 覆盖率目标

| 阶段 | 目标覆盖率 | 预计新增测试 |
|------|-----------|------------|
| Phase 1 | 35% | ~80 |
| Phase 2 | 55% | ~60 |
| Phase 3 | 80% | ~120 |
| Phase 4 | 88% | ~40 |
| Phase 5 | 92% | ~15 |
| **总计** | **≥90%** | **~315** |

---

## 五、风险与注意事项

1. **Axum E2E 测试复杂度**：`lists.rs` 是 758 行的大文件，包含 30+ 个路由，测试编写量大但模式重复，可通过辅助函数批量构造。
2. **SMTP 会话测试**：需要模拟 TCP 流，可使用 `tokio::io::DuplexStream` 或启动真实端口测试。
3. **AI 模块外部依赖**：阿里云 API 需要 mock，避免测试依赖真实网络。
4. **Passkey / WebAuthn**：当前为 stub，测试只需验证返回 NOT_IMPLEMENTED。
5. **TOTP 时间敏感性**：测试需固定时间或使用 `totp_rs` 的生成函数避免 flaky test。
6. **SeaORM Migration 覆盖率**：migration 文件中的 `up`/`down` 方法在 coverage 中统计困难，不计入主要目标。
