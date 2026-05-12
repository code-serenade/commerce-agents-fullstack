# Post 8-39：Web 增长交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 39 个补强项：`Post 8-39`。

本次目标：

- 把旧产品网站增长 / 货架增长类能力中已经 native 化的 Web skill 接入 workspace artifact 体系。
- 让 SEO 优化、标题生成、页面转化优化、店铺装修、关键词研究、详情页文案和标题 SEO 评分成为可查询、可预览、可导出的 Web 增长交付物。

涉及范围：

- 产品内核：Web 增长岗位从“当轮 skill 结果”推进到“可复用的增长交付物”。
- 后端 agent：`web_*` skill 输出补齐 `status=success`、`has_data`、`adapter_status`、`data_readiness` 和 `summary`，继续保留缺必填参数时的 validation error。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `web_seo_plan`、`web_title_candidates`、`web_conversion_plan`、`web_store_design_brief`、`web_keyword_matrix`、`web_product_description`、`web_title_seo_score`。
- API/SSE 文档：同步 Web growth artifact 类型、默认估算披露和缺输入不创建假交付物的语义。

用户可感知变化：

- 用户让 Web 增长岗位做 SEO、标题、页面转化、店铺装修、关键词、详情页文案或标题评分时，成功结果可以在当前 workspace 真实新增对应增长交付物。
- 这些交付物沿用现有 artifact 列表、详情、预览和导出链路。
- 如果使用平台规则或基准默认值，结果会通过 `data_readiness=calculated_from_defaults` 明确告知前端和用户。
- 如果缺少必填参数，例如店铺装修缺少 `store_type` 或标题评分缺少 `title`，后端返回 validation error，不会创建假交付物。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent web_ -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_web_ -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`
