# 测试覆盖率报告

> 生成时间：2026-05-26
> 工具：cargo-tarpaulin

## 总体概况

| 指标 | 数值 |
|------|------|
| **整体覆盖率** | **15.45%** |
| 覆盖行数 | 548 / 3548 |

## 各模块覆盖率详情

### 已覆盖较好（>70%）

| 文件 | 覆盖行数 | 覆盖率 |
|------|---------|--------|
| `src/tasks/cleanup.rs` | 32/32 | **100%** |
| `src/utils/crypto.rs` | 20/20 | **100%** |
| `src/utils/email.rs` | 8/8 | **100%** |
| `migration/src/lib.rs` | 18/18 | **100%** |
| `src/tasks/ai_moderate.rs` | 16/17 | 94% |
| `src/tasks/deliver.rs` | 38/41 | 93% |
| `src/config.rs` | 13/14 | 93% |
| `src/services/mfa_service.rs` | 133/150 | 89% |
| `src/services/domain_service.rs` | 42/48 | 88% |
| `src/services/auth_service.rs` | 92/111 | 83% |
| `src/services/subscriber_service.rs` | 61/84 | 73% |

### 覆盖较差（<50%）

| 文件 | 覆盖行数 | 覆盖率 |
|------|---------|--------|
| `src/services/list_service.rs` | 29/81 | 36% |
| `src/services/moderation_service.rs` | 27/76 | 36% |
| `src/utils/response.rs` | 3/15 | 20% |

### 完全未覆盖（0%）

以下模块一行测试都没有：

#### API 层（约 2,000+ 行）
- `src/api/middleware/auth.rs` — JWT 认证中间件
- `src/api/middleware/error.rs` — 错误处理
- `src/api/v1/admin.rs` — 管理员接口
- `src/api/v1/auth.rs` — 登录/注册/刷新等认证接口
- `src/api/v1/domains.rs` — 域名管理接口
- `src/api/v1/health.rs` — 健康检查
- `src/api/v1/lists.rs` — 邮件列表 CRUD（最大文件，758 行）
- `src/api/v1/messages.rs` — 消息接口（空路由 stub）
- `src/api/v1/moderation.rs` — 审核接口
- `src/api/v1/subscribers.rs` — 订阅者接口（空路由 stub）
- `src/api/v1/templates.rs` — 模板接口
- `src/api/v1/users.rs` — 用户管理接口
- `src/api/mod.rs` — 路由聚合

#### SMTP 层（约 600+ 行）
- `src/smtp/server.rs` — SMTP 服务器
- `src/smtp/processor.rs` — 邮件处理逻辑
- `src/smtp/auth_check.rs` — SMTP 认证
- `src/smtp/parser.rs` — 协议解析
- `src/smtp/client.rs` — SMTP 客户端
- `src/smtp/verp.rs` — VERP 地址编解码

#### 其他核心服务
- `src/services/ai_service.rs` — AI 服务封装
- `src/services/archive_service.rs` — 归档与搜索
- `src/services/mail_service.rs` — 邮件服务（空 stub）
- `src/services/notification_service.rs` — 通知发送
- `src/services/template_service.rs` — 模板渲染
- `src/main.rs` — 程序入口

#### 任务调度
- `src/tasks/bounce.rs` — 退信处理
- `src/tasks/digest.rs` — 摘要生成

#### AI 模块
- `src/ai/client.rs` — 阿里云 AI 客户端
- `src/ai/aliyun_signer.rs` — 阿里云请求签名
- `src/ai/parser.rs` — AI 响应解析（stub）
- `src/ai/policy.rs` — 审核策略

#### 工具函数
- `src/utils/validation.rs` — 邮箱/域名验证

#### 数据模型
- `src/models/*.rs` — 模型文件（除 migration 外均为 SeaORM 派生代码，覆盖率统计中几乎为 0）

## 现有测试分布

项目中共有 **19 个集成测试**，分布在 5 个测试文件中：

| 测试文件 | 测试数量 | 覆盖范围 |
|---------|---------|---------|
| `tests/auth_test.rs` | 3 | 注册/登录/TOTP 生命周期 |
| `tests/domain_test.rs` | 1 | 域名的 CRUD |
| `tests/integration_test.rs` | 3 | 列表服务、审核服务、订阅者生命周期 |
| `tests/tasks_test.rs` | 3 | cleanup/deliver/ai_moderate 任务 |
| `tests/utils_test.rs` | 9 | 工具函数（UUID、密码哈希、VERP 地址等）|

**关键缺失：**
- `src/` 下**没有任何单元测试**（`lib.rs` 和 `main.rs` 的 unit test 均为 0）
- 没有任何 **API 端到端测试**（没有测试任何 HTTP handler）
- 没有任何 **SMTP 相关测试**
- 没有任何 **AI 调用相关测试**

## 结论

该项目**远未做到测试全覆盖**。核心功能中，仅认证、域名、订阅者等少数服务有较完整的集成测试；API 路由、SMTP 协议处理、AI 审核、邮件归档、退信处理等大量核心模块完全没有测试覆盖。
