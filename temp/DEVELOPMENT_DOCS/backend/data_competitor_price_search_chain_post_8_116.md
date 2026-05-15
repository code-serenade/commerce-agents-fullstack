# Post 8-116：数据竞品与价格弹性搜索链路

## 需求摘要

继续迁移旧产品能力。旧产品 `data_analysis.py` 中 `DataCompetitorAnalysis` 和 `DataPriceElasticity` 会在 skill 内部静默搜索实时竞品情报或市场定价数据。新产品不能把搜索藏在 native skill 内部，需要按现有 Agent runtime 和 SSE 过程透明规则，把本地数据分析和外部搜索拆成两个可见 skill。

## 后端设计

本阶段不新增数据库表、不新增 HTTP 路由、不修改认证模型。

涉及既有 Agent 能力：

- native skill：`data_competitor_analysis`
- native skill：`data_price_elasticity`
- adapter skill：`search_competitor`
- SSE：继续使用既有 `skill_requested` / `skill_result`

## 核心规则

1. 用户只说“竞品分析 / 价格弹性分析”时，仍只触发对应 native data skill。
2. 用户同时要求“实时 / 最新 / 联网 / 搜索 / 公开资料 / 市场定价 / 竞品价格区间”时，data worker 追加 `search_competitor`。
3. 竞品分析链路：`data_competitor_analysis` -> `search_competitor`。
4. 价格弹性链路：`data_price_elasticity` -> `search_competitor`，搜索 focus 使用 `price`。
5. 搜索作为独立 skill 暴露到 SSE 和 skill run，不混入 native skill 输出。
6. 未显式选择商品时不注入默认商品；只有本轮选中商品时才携带 `product_ref/product_name/product_category`。
7. search adapter 未配置、失败、无可信来源或来源过期时，按既有搜索降级规则处理，不伪造实时竞品数据。

## API 影响

不新增 HTTP API。

SSE 事件结构不变，但 data 岗位在特定输入下会连续出现两个 `skill_requested` / `skill_result`：

```json
[
  {
    "skillId": "data_competitor_analysis",
    "role": "data"
  },
  {
    "skillId": "search_competitor",
    "role": "data"
  }
]
```

或：

```json
[
  {
    "skillId": "data_price_elasticity",
    "role": "data"
  },
  {
    "skillId": "search_competitor",
    "role": "data"
  }
]
```

## 用户可感知变化

用户要求数据岗位做竞品分析或价格弹性分析，并明确要实时市场/竞品公开资料时，过程区会看到本地分析和外部搜索两个真实步骤。这样用户能分清“系统基于输入做的确定性分析”和“外部来源检索”，不会把无来源的数据当成实时市场情报。

## 验证计划

- `cargo fmt --all`
- `cargo test -p agent data`
- `cargo test -p agent search`
- `cargo check -p agent -p web-server`
- `cargo clippy -p agent -p web-server`
- `git diff --check`
