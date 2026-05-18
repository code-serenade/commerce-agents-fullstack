# Post 8-103 Search Selected Product Context

## 状态

done

## 本次目标

- 让 `search_*` 外部搜索类 skill 能消费本轮显式选中商品上下文。
- 当用户问“搜这个商品竞品公开资料 / 查本品评价 / 看所选商品趋势 / 查这个商品平台规则”时，后端搜索 input 使用选中商品名称、类目、平台和 `product_ref=product:{uuid}`。
- 未选商品时保持普通搜索链路，不写空商品字段，不从历史、预设、测试商品里推断默认商品。
- 搜索 provider 未配置、失败或无可信来源时继续明确 `deferred/degraded`，不伪造 live 来源。

## 本次修改

- 后端 worker：`autosuggest_search` 接收 `Task`，可以读取已校验的 `context.selected_summary`。
- 后端 worker：新增“这个商品 / 本品 / 当前商品 / 所选商品 / selected product”等指代识别。
- 后端 worker：搜索 query / industry / platform 在显式选品场景下使用选中商品名称、类目和平台。
- 后端 worker：`search_product_reviews`、`search_industry_benchmarks`、`search_competitor`、`search_trends`、`search_market_info`、`search_platform_policy` 自动补偿 input 可携带 `product_ref`、`product_name`、`product_category`。
- 后端 search skill：搜索输出 `query` 对象回显可选商品上下文，便于 SSE、artifact 和前端过程面板区分指定商品搜索。
- API/SSE：同步 `API_DOCS/sse.md`。
- 迁移文档：同步阶段验收、总计划和 skill 矩阵。

## 用户可感知变化

- 用户手动勾选商品后，再问“搜这个商品的竞品公开资料 / 查这个商品用户口碑 / 看本品最近趋势”，后端会把本轮选中商品的名称、类目、平台和 `product:<uuid>` 带进搜索 skill。
- 用户没有手动选商品时，即使问题里有“咖啡”等类目词，搜索仍按普通问题搜索，不会暗中绑定千乐咖啡、历史商品、预设商品或测试商品。
- 前端后续展示 `skill_result.output.query` 或搜索证据 artifact 时，可以看到 `product_ref/product_name/product_category`，从而区分“指定商品外部搜索”和“普通市场搜索”。
- 当前仍不伪造联网结果；只有 `[external_search]` 配好且 provider 返回可信来源时，搜索结果才会是 live。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent selected_product_context_feeds_search_queries -- --nocapture`
- [x] `cargo test -p agent search_without_selection_has_no_empty_product_context -- --nocapture`
- [x] `cargo test -p agent search_skill_echoes_product_context_in_degraded_output -- --nocapture`
- [x] `cargo test -p agent autosuggests_search_skills_from_external_search_intent -- --nocapture`
- [x] `cargo test -p agent autosuggest_search_preserves_native_analysis_without_external_intent -- --nocapture`
- [x] `cargo fmt --all --check`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`

## 后续缺口

- 真实外部搜索 provider 的部署、密钥、来源质量治理和实时数据覆盖仍属于后续 adapter/service 配置工作。
- 前端后续应在过程面板和搜索证据详情里展示 `product_ref/product_name/product_category`，并明确 provider 未配置、降级或无来源时的原因。
