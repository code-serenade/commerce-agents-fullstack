# Post 8-111：外部搜索结果数量约束

状态：done

## 本轮目标

让旧产品搜索策略里的“结果数量控制”真正进入新产品后端 contract。App 外部搜索 API 和 Agent `search_*` skill 已经会把 `maxResults` / `max_results` 传给 provider，本轮补齐 provider 返回后端归一化时的结果数量约束。

## 本轮完成

- App 侧 `/api/app/external-search/search` 归一化来源时按本次 `maxResults` 截断。
- Agent `search_trends`、`search_competitor`、`search_market_info`、`search_product_reviews`、`search_platform_policy`、`search_industry_benchmarks` 归一化来源时按 `SearchPlan.max_results` 截断。
- `data.items` / `sources`、`provenance.sourceCount`、`evidence` 都只基于截断后的可信来源生成。
- 继续保留全局硬上限 `12`，并在归一化层防御性 clamp 到 `1..=12`。
- 不新增前端假展示，不伪造搜索结果，不改变 provider 未配置或无可信来源时的 degraded/deferred 语义。

## 用户可感知变化

- 后续部署真实外部搜索 provider 后，如果策略或配置要求最多返回 2 条、3 条或 6 条来源，用户看到的实时来源、证据摘要和搜索证据交付物都会遵守这个数量。
- 过多 provider 返回不会把回答过程、右侧来源面板或后续上下文包撑乱。
- 搜索结果数量会更符合“策略选择 / 任务配置”的预期，避免系统看起来像不听用户设置。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p service external_search`
- [x] `cargo test -p agent search`
- [x] `cargo check -p service -p agent -p web-server`
- [x] `cargo clippy -p service -p agent -p web-server`
- [x] `git diff --check`
