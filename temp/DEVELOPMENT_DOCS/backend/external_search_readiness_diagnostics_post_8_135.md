# External Search Readiness Diagnostics - Post 8-135

## 1. 需求摘要

继续按旧产品能力迁移计划推进后端能力。当前很多电商 skill 已经能在需要“最新 / 联网 / 实时 / 公开来源”时追加 `search_*` 步骤，但搜索 provider 是否真的可用、缺少什么配置、失败后为什么降级，还没有形成足够清晰的后端诊断契约。

本轮目标是增强外部搜索 adapter 的后端可诊断性，让 App API 与 Agent `search_*` skill 都能返回同一类 readiness 信息。前端后续只消费真实状态，不伪造实时来源。

## 2. 业务边界

- 不接淘宝、抖音、巨量千川等官方平台写操作。
- 不填假的搜索 provider 地址或 API key。
- 不改变现有 `/api/app/external-search/*` 路由。
- 不创建新数据库表。
- 不改变用户鉴权模式，继续要求有效 App 用户 JWT。
- 搜索结果仍然只有在 provider 返回可追溯 URL 来源时才可作为 live evidence。

## 3. 数据库设计

本轮无数据库变更。

原因：

- readiness 只来自 `backend/config/services.toml` 的 `[external_search]` 配置和 provider 调用结果。
- 不需要持久化新实体。
- 后续如需保存 provider 健康检查历史，可另开独立设计。

## 4. 后端契约

### 4.1 新增只读结构：`ExternalSearchReadiness`

字段：

- `status`：`configured | disabled | missing_config`
- `enabled`：配置中的 `[external_search].enabled`
- `configured`：是否满足可发起 provider 请求的最低条件
- `provider`：provider 名称，空时为 `external_search`
- `missingConfig`：缺失配置项，不包含密钥值
- `hasApiKey`：是否配置了非空 `api_key`，只返回布尔值
- `timeoutSec`：实际超时时间
- `defaultTopic`：默认搜索 topic
- `defaultDays`：默认 freshness 天数
- `defaultMaxResults`：默认来源数量上限
- `maxResultsLimit`：后端硬上限，当前为 `12`
- `message`：给前端/用户展示的中文诊断说明

### 4.2 `GET /api/app/external-search/status`

在现有响应基础上追加：

- `readiness`

旧字段 `adapter/enabled/configured/provider/requiredConfig/status` 保持兼容。

### 4.3 `GET /api/app/external-search/search`

在现有响应基础上追加：

- `readiness`

无论 live 还是 degraded，都返回同一结构，方便前端解释当前搜索能力状态。

### 4.4 Agent `search_*` skill 输出

在 `output.adapter` 下追加：

- `readiness`

并在 metadata 中追加：

- `adapter_readiness`

这样 SSE `skill_result`、workspace artifact 和后续前端来源面板都能看到搜索 adapter 的真实准备状态。

## 5. 鉴权设计

- App API 仍由 web-server 提取 `AuthUser`。
- service 层继续校验当前 `app_users` 存在且为 `enabled`。
- Agent search skill 不直接处理用户鉴权，只消费 web-server 注入到 runtime 的 `[external_search]` 配置。

## 6. 错误与降级

- `disabled`：`enabled=false`，说明外部搜索未启用。
- `missing_config`：已启用但缺 `base_url` 或 `search_endpoint`。
- `configured`：配置满足发起请求条件，但 provider 仍可能失败、无来源或只返回过期来源。
- provider 失败、无可信来源、只返回 stale 来源时，仍返回 degraded，不伪造 live 数据。

## 7. 开发顺序

1. 在 service DTO 中增加 readiness 结构和配置诊断方法。
2. 在 App 外部搜索 status/search 响应中追加 readiness。
3. 在 Agent `search_*` 输出中追加同语义 readiness。
4. 同步 `API_DOCS/app.md` 与 `API_DOCS/sse.md`。
5. 更新迁移计划和开发日志。
6. 运行 cargo fmt、相关测试、cargo check、cargo clippy。

