# Post 8-65: 商品 / 活动指标查询 native skill

## 需求摘要

继续按“旧产品能力 = 后端内核能力来源，新产品架构 = 唯一落地框架”的路线迁移指标类能力。旧产品指标仓思想不应只停在店铺级 `query_store_metrics`，还需要支持商品级和活动级指标查询，使用户在问答中能直接围绕单个商品、SKU、营销活动或投放活动做分析。

## 本轮范围

新增两个 Agent data native skill：

- `query_product_metrics`
- `query_campaign_metrics`

它们只消费本轮显式提供的指标、显式传入的商品/活动标识，或后续 service adapter 注入的数据。本轮不读取平台官方 API，不读取未设计的数据库表，不把任何预设商品或历史商品当成默认上下文。

## 数据库设计

本轮不新增数据库表。

原因：

- 当前目标是先补齐 Agent runtime 内核能力和 SSE 契约。
- 真实商品 / 活动指标持久化读取应后续通过 service/repo adapter 统一设计，避免在 agent crate 内直连数据库。

后续如果接入真实指标仓，应优先设计：

- 用户隔离的商品指标表或统一 metrics 表维度字段。
- 活动指标表或 campaign metrics adapter。
- 按 `user_id`、`workspace_id`、`product_id`、`campaign_id` 鉴权读取。

## 接口设计

本轮不新增 HTTP API。

对外复用既有 Agent SSE：

```text
POST /api/sse/agent/session/stream
```

新增行为体现在既有事件：

```text
skill_requested
skill_result
```

## 鉴权设计

不新增鉴权入口。

Agent SSE 仍由现有 web-server 认证链路处理当前用户。新增 skill 不在 agent crate 内访问数据库，也不绕过 service 层读取用户资源。

## Skill 输入契约

`query_product_metrics` 支持：

- `product_id`
- `product_name`
- `sku_id`
- `platform`
- `days`
- `gmv` / `revenue`
- `orders` / `order_count`
- `visitors` / `uv`
- `clicks`
- `ad_spend`
- `refund_amount`
- `gross_profit`
- `inventory`
- `stockout_count`

`query_campaign_metrics` 支持：

- `campaign_id`
- `campaign_name`
- `campaign_type`
- `platform`
- `days`
- `budget`
- `spend` / `ad_spend`
- `gmv` / `revenue`
- `orders`
- `visitors` / `uv`
- `clicks`
- `impressions`
- `coupon_cost`
- `refund_amount`

## 输出契约

有显式指标时：

- `status=success`
- `adapter_status=manual_input`
- `data_readiness=explicit_product_metrics` 或 `explicit_campaign_metrics`
- 返回 totals、derived metrics、risks、next actions

无显式指标时：

- `status=deferred`
- `adapter_status=product_metrics_required` 或 `campaign_metrics_required`
- `data_readiness=missing_user_input`
- 明确说明未接真实 adapter，不伪造淘宝、抖音、千川或其它平台实时数据

## 自然语言触达

worker autosuggest 增加自然触达：

- 商品 / SKU / 单品指标、商品表现、商品数据 -> `query_product_metrics`
- 活动 / 大促 / campaign 指标、活动复盘、投放活动数据 -> `query_campaign_metrics`

更具体的店铺大盘、竞品、退款、趋势、渠道 ROI 等已有 skill 继续保持优先级，不被新查询 skill 抢走。

## 验证计划

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p agent query_product_metrics -- --nocapture`
- `cargo test -p agent query_campaign_metrics -- --nocapture`
- `cargo test -p agent autosuggests_product_and_campaign_metrics -- --nocapture`
- `cargo check -p agent`
- `cargo clippy -p agent`

