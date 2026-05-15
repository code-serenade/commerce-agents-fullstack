# Post 8-121：财务健康/财务基准搜索链路

## 状态

done

## 本轮目标

把旧产品财务健康诊断中隐藏的行业财务基准搜索能力，迁成新产品框架里的可见、可审计、可降级 skill 链路。

旧产品做法：

- `accounting_financial_narrative` 内部固定搜索“电商行业 毛利率 净利率 广告ROI 财务基准 2026”。
- 搜索过程对前端不可见，失败时容易和 native 诊断混在一起。

新产品做法：

- 财务岗位先运行真实 native skill。
- 当用户明确要求行业财务基准、财务对标、最新毛利率/净利率/广告 ROI 基准或联网搜索时，追加 `search_market_info(topic=finance)`。
- 搜索通过统一 `search_*` adapter、SSE `skill_requested` / `skill_result` 和降级规则展示。

## 涉及能力

- `accounting_financial_narrative`
- `accounting_pl_statement`
- `accounting_gmv_waterfall`
- `accounting_budget_vs_actual`
- `search_market_info(topic=finance)`

## 后端变化

- `accounting` 岗位新增 `autosuggest_accounting_requests`，支持 native + search 双 skill 编排。
- 财务健康、P&L、GMV 瀑布、预算执行分析可以根据自然语言触发对应 native skill。
- 财务基准搜索统一落到 `search_market_info(topic=finance)`，不隐藏在 native 财务 skill 内部。
- 纯“查财务基准 / 搜公开来源”请求只触发搜索，不误触发预算编制或财务诊断。
- `accounting` 角色白名单允许使用 `search_market_info` 和 `search_industry_benchmarks`。

## 商品上下文边界

- 只有本轮显式选中商品时，搜索 input 才携带 `product_ref/product_name/product_category`。
- 未显式选品时，不从历史摘要、默认商品或演示商品注入商品上下文。
- 这继续保持“指定商品”和“未指定商品”两条链路分开。

## 用户可感知变化

- 用户问“做财务健康诊断，并联网查最新行业财务基准”时，过程区会看到两个步骤：财务健康诊断、外部财务基准搜索。
- 用户只问“查最新电商行业毛利率、净利率和广告 ROI 财务基准”时，只会进入搜索步骤，不会生成无关预算计划。
- 用户选中商品后再问“给这个商品做财务健康诊断并查财务基准”，搜索会带上该商品引用；未选商品时不会出现千乐咖啡或其它历史商品上下文。

## 验证

- `cargo fmt --all`
- `cargo test -p agent accounting`
- `cargo test -p agent search`
- `cargo test -p agent keeps_roles_from_requesting_unowned_skills`
- `cargo check -p agent -p web-server`
- `cargo clippy -p agent -p web-server`
- `git diff --check`
