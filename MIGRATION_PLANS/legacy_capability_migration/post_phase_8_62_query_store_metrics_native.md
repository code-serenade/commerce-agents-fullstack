# Post 8-62：店铺指标查询 skill native 显式输入补强

状态：done

## 本次目标

- 继续按后端优先方式迁移旧产品 `query_store_metrics` 能力。
- 让新产品 Rust Agent 内的 `query_store_metrics` 不再只是固定 `deferred`。
- 支持本轮显式提供 GMV、订单、UV、点击、广告费、退款、毛利、多平台指标时生成结构化店铺指标摘要。
- 没有真实指标时仍明确降级，不伪造淘宝、抖音、千川或其它平台实时数据。

## 本次修改

- 后端 agent：`query_store_metrics` spec 从“固定延迟计划”改为 deterministic native skill。
- 后端 agent：新增显式指标输入字段，包括 `gmv`、`orders`、`visitors/uv`、`clicks`、`ad_spend`、`refund_amount`、`gross_profit`、`platforms[]` 等。
- 后端 agent：有显式指标时输出 `status=success`、`adapter_status=manual_input`、`data_readiness=explicit_metrics`。
- 后端 agent：输出 `totals`、`derived_metrics`、`week_over_week`、`anomalies`、`platforms` 和数据来源说明。
- 后端 agent：没有显式指标时输出 `status=deferred`、`adapter_status=store_metrics_required`、`data_readiness=missing_user_input`。
- 后端 agent：支持多平台指标按 `platform` 过滤；只汇总用户提供的行。
- API/SSE：同步 `query_store_metrics` 的 `skill_result.output` 语义。
- 迁移文档：同步迁移矩阵、开发记录、阶段验收记录和总计划。

## 用户可感知变化

- 用户或上游 Agent 如果明确提供经营指标，系统可以先真实汇总店铺大盘，再让后续数据、运营、财务 skill 使用这些指标。
- 如果用户没有提供指标，系统会明确提示缺少店铺指标或 `store_metrics` adapter，而不是假装已经读到平台后台。
- 多平台指标输入时，用户可以指定 `platform=douyin` 只分析抖音这一组显式数据。
- 前端右侧过程区可以把这个 skill 展示为“已基于本轮输入读取数据”或“等待真实数据输入/adapter”，不会混淆为平台实时读取。

## 当前验证

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/query_store_metrics_native_post_8_62.md`
- [x] Agent 单测：显式指标成功路径
- [x] Agent 单测：无指标 deferred 路径
- [x] Agent 单测：多平台过滤路径

## 后续缺口

- 尚未接入 service 层真实 `store_metrics` adapter。
- 尚未接淘宝、抖音、千川等官方平台数据 API。
- 后续如果要读取数据库指标，必须通过已鉴权 service/adapter 注入 Agent，不应让 `agent` crate 直接绕过用户隔离访问数据库。
