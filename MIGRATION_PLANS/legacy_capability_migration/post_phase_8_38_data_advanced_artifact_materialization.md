# Post 8-38：Data 进阶预测与留存交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 38 个补强项：`Post 8-38`。

本次目标：

- 把旧产品数据岗位中已经 native 化的进阶预测、留存、LTV、价格弹性能力接入 workspace artifact 体系。
- 让多周期趋势、LTV 客户价值、队列留存、季节性分解、价格弹性和需求预测成为可查询、可预览、可导出的数据交付物。

涉及范围：

- 产品内核：数据岗位从异常/趋势/分群继续推进到预测、留存、客户价值和价格分析。
- 后端 agent：沿用现有 `data::execute` 统一出口，确认进阶 data skill 的 `status=success/deferred`、`has_data`、`adapter_status`、`data_readiness` 和 `summary` 契约。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `data_multi_period_trend_report`、`data_ltv_report`、`data_cohort_report`、`data_seasonal_decompose_report`、`data_price_elasticity_report`、`data_demand_forecast_report`。
- API/SSE 文档：同步 data advanced artifact 类型、默认估算披露和缺输入不创建假交付物的语义。

用户可感知变化：

- 用户让数据岗位做多周期趋势、LTV、队列留存、季节性分解、价格弹性或需求预测时，成功结果可以在当前 workspace 真实新增对应数据进阶报告。
- 这些交付物沿用现有 artifact 列表、详情、预览和导出链路。
- 如果缺少必要时序数据，后端返回 `deferred`，不会创建看似真实的假预测报告。
- 如果 LTV、队列留存或价格弹性使用确定性默认数据，结果会通过 `data_readiness=calculated_from_defaults` 明确告知前端和用户。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent data_ -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_data_ -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`
