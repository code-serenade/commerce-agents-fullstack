# Post 8-114：外部搜索来源时效过滤

## 需求摘要

继续按“旧产品能力 = 后端内核能力来源，新产品架构 = 唯一落地框架”的迁移计划开发。外部搜索已提升为高优先级，当前要继续增强真实搜索结果的可信度，避免 provider 返回过期资料时被当作实时来源进入 Agent 回答、右侧来源面板或 `search_evidence` artifact。

## 后端设计

本阶段不新增数据库表、不新增 HTTP 路由、不修改认证模型。

涉及两条既有链路：

- App 侧外部搜索 API：`GET /api/app/external-search/search`
- Agent 搜索 skill：`search_trends`、`search_competitor`、`search_market_info`、`search_product_reviews`、`search_platform_policy`、`search_industry_benchmarks`

## 核心规则

1. provider 返回来源后，后端先归一化 URL、标题、摘要、发布时间和 domain。
2. 如果来源有可解析发布时间，且早于 `now - requested_days`，该来源视为 stale，不能进入 live `sources` / `data.items` / `evidence`。
3. 如果来源没有发布时间或发布时间无法解析，后端保留该来源，但不会给它额外 freshness 加分。
4. 如果过滤后仍有可信来源，返回 `status=live`，并在 `provenance.staleFilteredCount` / `freshness.stale_filtered_count` 中记录过滤数量。
5. 如果过滤后没有可信来源，返回 degraded / deferred，不创建假搜索证据 artifact。

## API 影响

`ExternalSearchProvenance` 增加：

```json
{
  "staleFilteredCount": 1
}
```

Agent `search_*` skill 的 `output.provenance` 增加：

```json
{
  "stale_filtered_count": 1
}
```

Agent `search_*` skill 的 `output.freshness` 增加：

```json
{
  "stale_filtered_count": 1
}
```

## 用户可感知变化

用户要求“最近 7 天”“最新”“实时”时，后端不会把 90 天前或更早的旧资料当作实时来源展示。前端后续在来源面板可以看到有多少条过期资料被过滤；如果 provider 只返回过期资料，系统会明确降级，而不是假装已经搜到有效实时资料。

## 验证计划

- `cargo fmt --all`
- `cargo test -p agent search`
- `cargo test -p service external_search`
- `cargo check -p service -p agent -p web-server`
- `cargo clippy -p service -p agent -p web-server`
- `git diff --check`
