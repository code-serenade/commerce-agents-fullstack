# Post 8-109 外部搜索来源归一化增强

## 需求摘要

旧产品的搜索类 skill 是电商多智能体的重要能力来源。新产品已经有 `search_*` skill、App 侧外部搜索接口、SSE 搜索证据 artifact 和降级契约，但 provider 返回格式仍偏窄，真实接入不同搜索服务时容易因为字段名不一致而拿不到来源，或把缺少 URL / domain 的弱结果误当作可信联网证据。

本轮目标是在同事给的新产品框架内增强后端搜索 adapter 的来源归一化和可信过滤：

- 支持更多常见 provider 返回结构。
- 只有具备可追溯 URL / domain 的结果才进入 live source。
- 没有可信来源时继续走 degraded/deferred，不伪造实时结果。
- App 侧外部搜索接口和 Agent `search_*` skill 保持同一语义。

## 数据库设计

本轮不新增表、不修改表结构、不新增 migration。

现有搜索证据仍通过已有 workspace artifact 链路保存：

- `artifactType=search_evidence`
- `format=json`
- `storageKind=inline`
- `metadata.kind=agent_search_evidence`

## 鉴权设计

不改变鉴权。

- App 侧 `/api/app/external-search/*` 仍要求有效 App 用户 JWT。
- Service 层仍校验 `app_users.status=enabled`。
- Agent SSE 仍通过既有登录态、workspace 和 run 链路执行。

## 后端接口设计

不新增 HTTP API，不改变响应字段名。

影响范围：

- `backend/crates/service/src/api/external_search.rs`
- `backend/crates/agent/src/skills/search.rs`

增强点：

1. provider source array 兼容：
   - `data.items`
   - `data.results`
   - `data.sources`
   - `data.organic`
   - `data.organic_results`
   - `data.webPages.value`
   - `items`
   - `sources`
   - `results`
   - `organic`
   - `organic_results`
   - `webPages.value`
   - `value`

2. source 字段兼容：
   - title: `title` / `name` / `headline`
   - url: `url` / `link` / `href` / `source_url` / `sourceUrl`
   - snippet: `snippet` / `body` / `summary` / `content` / `description` / `text` / `excerpt`
   - published: `published_at` / `publishedAt` / `pubDate` / `published` / `datePublished` / `date`
   - domain: 从 URL 提取，避免 provider 单独提供的 `domain` 把相对路径包装成可信来源。

3. 可信过滤：
   - 缺少 URL 的结果不进入 live source。
   - URL 和 domain 均无法形成可信追溯时不进入 live source。
   - 重复 URL 仍去重并重新分配 rank。

## 失败处理

- provider 未配置：保持 `degraded` / `deferred`。
- provider 请求失败：保持 `degraded` / `deferred`，带 `degraded_reason`。
- provider 返回结果但无可信 URL / domain：保持 `degraded` / `deferred`，不创建假搜索证据 artifact。

## 验证方式

计划运行：

- `cargo fmt --all`
- `cargo test -p service external_search`
- `cargo test -p agent search`
- `cargo check -p service -p agent -p web-server`
- `cargo clippy -p service -p agent -p web-server`
- `git diff --check`
