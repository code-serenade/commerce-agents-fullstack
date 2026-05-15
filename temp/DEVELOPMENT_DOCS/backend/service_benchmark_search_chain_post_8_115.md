# Post 8-115：客服满意度行业基准搜索链路

## 需求摘要

继续按“旧产品能力 = 后端内核能力来源，新产品架构 = 唯一落地框架”的迁移路线开发。本阶段迁移旧产品客服 DSR / NPS 分析中“需要实时行业基准时联网搜索”的能力，但不照搬旧产品在客服 skill 内部静默搜索的方式。

## 后端设计

本阶段不新增数据库表、不新增 HTTP 路由、不修改认证模型、不在 web-server 中新增 LLM client。

使用既有 Agent runtime、worker 自动补偿、SkillCatalog 权限和 SSE 事件：

- native skill：`service_dsr_improvement`、`service_nps_analyzer`、`service_nps_driver_analysis`
- adapter skill：`search_industry_benchmarks`
- SSE：继续通过既有 `skill_requested` / `skill_result` 展示每个 skill 步骤

## 核心规则

1. 用户只要求 DSR / NPS / 客服满意度分析时，仍只触发客服 native skill。
2. 用户同时要求“最新 / 实时 / 联网 / 行业基准 / 均值 / benchmark”时，客服 worker 追加一个 `search_industry_benchmarks` 请求。
3. 搜索请求必须作为独立 skill 暴露给 SSE 和 skill run 记录，不允许藏在客服 native skill 内部。
4. 搜索 input 复用统一外部搜索契约，包含 `industry`、`metric`、`platform`、`_needs_fresh_data`、可选 `max_results` 和显式商品上下文。
5. 未选择商品时不注入默认商品，不从历史摘要偷取商品上下文。
6. 如果搜索 adapter 未配置、失败、无可信来源或来源过期，由既有 search skill 明确 deferred / degraded，不伪造行业基准。

## API 影响

不新增 HTTP API。

SSE 事件结构不变，但客服岗位在特定输入下会连续出现两个 `skill_requested` / `skill_result`：

```json
[
  {
    "skillId": "service_dsr_improvement",
    "role": "service"
  },
  {
    "skillId": "search_industry_benchmarks",
    "role": "service"
  }
]
```

或：

```json
[
  {
    "skillId": "service_nps_driver_analysis",
    "role": "service"
  },
  {
    "skillId": "search_industry_benchmarks",
    "role": "service"
  }
]
```

## 用户可感知变化

用户让客服虚拟员工分析 DSR / NPS，并明确要求查看最新行业基准时，前端过程区可以看到两个真实步骤：先做客服分析，再查行业基准来源。用户不会误以为系统暗中用了不可见资料；如果外部搜索不可用，系统会明确显示降级，而不是假装有实时行业数据。

## 验证计划

- `cargo fmt --all`
- `cargo test -p agent service`
- `cargo test -p agent search`
- `cargo check -p agent -p web-server`
- `cargo clippy -p agent -p web-server`
- `git diff --check`
