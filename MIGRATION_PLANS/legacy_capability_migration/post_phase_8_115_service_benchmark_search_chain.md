# Post 8-115：客服满意度行业基准搜索链路

状态：done

## 本轮目标

迁移旧产品客服 DSR / NPS 分析里“需要实时行业基准时联网搜索”的能力，但按新产品架构拆成可见 skill 链路，而不是在客服 native skill 内部静默调用搜索。

## 本轮完成

- `service` worker 在 DSR、店铺评分、服务评分、NPS、满意度驱动因素、投诉分类等客服分析意图中，识别“行业基准 / 行业均值 / 最新 / 实时 / 联网”需求。
- 命中该意图时，自动生成两个 skill 请求：
  - `service_dsr_improvement`、`service_nps_analyzer` 或 `service_nps_driver_analysis`
  - `search_industry_benchmarks`
- `search_industry_benchmarks` 继续复用统一搜索契约，携带 `industry`、`metric`、`platform`、`_needs_fresh_data`、可选 `max_results` 和显式商品上下文。
- `service` role catalog 已允许调用 `search_industry_benchmarks`。
- 搜索基准 metric 增加 `dsr`，用于 DSR / 店铺评分 / 描述相符 / 服务态度 / 物流服务行业基准搜索。

## 用户可感知变化

- 用户问“DSR下降，联网查最新行业基准并给提升方案”时，过程区会先显示客服分析 skill，再显示行业基准搜索 skill。
- 用户问“投诉分类和满意度驱动因素，查抖音 NPS 行业基准”时，过程区会显示 NPS 驱动分析和 NPS 基准搜索。
- 搜索不可用、失败、无可信来源或来源过期时，仍按搜索 skill 降级，不伪造实时行业数据。

## 边界

- 本轮不新增 HTTP API，不新增数据库表，不新增前端页面。
- 本轮不在客服 native skill 内部调用搜索 adapter。
- 本轮不接淘宝、抖音官方平台 API。
- 本轮不默认注入任何商品；只有本轮显式选中商品才携带商品上下文。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent service`
- [x] `cargo test -p agent search`
- [x] `cargo check -p agent -p web-server`
- [x] `cargo clippy -p agent -p web-server`
- [x] `git diff --check`
