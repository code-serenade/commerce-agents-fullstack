# Post 8-109 外部搜索来源归一化增强

## 本轮目标

继续按“旧产品能力迁移到新产品后端内核”的路线，补强搜索类 skill 的真实 adapter 能力。

本轮不接入具体淘宝、抖音或搜索商业账号，不伪造外部数据；只增强已经存在的 Rust 外部搜索 adapter，让后续真实 provider 接入时能更稳地识别来源，并过滤掉不可追溯的弱结果。

## 修改范围

后端：

- `backend/crates/service/src/api/external_search.rs`
- `backend/crates/agent/src/skills/search.rs`

文档：

- `temp/DEVELOPMENT_DOCS/backend/external_search_source_normalization_post_8_109.md`
- `API_DOCS/app.md`
- `API_DOCS/sse.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`

## 后端内核变化

App 侧 `/api/app/external-search/search` 和 Agent 侧 `search_*` skill 现在使用一致的来源归一化规则：

- 来源数组兼容 `data.items`、`data.results`、`data.sources`、`data.organic`、`data.organic_results`、`data.webPages.value`、`items`、`sources`、`results`、`organic`、`organic_results`、`webPages.value`、`value`。
- 来源字段兼容 `title/name/headline`、`url/link/href/source_url/sourceUrl`、`snippet/body/summary/content/description/text/excerpt`、`published_at/publishedAt/pubDate/published/datePublished/date`。
- 来源必须带可追溯 URL，并能从 URL 本身形成 domain；相对路径或只有 provider 单独给 domain 的条目不会进入 live evidence。
- 重复 URL 继续去重并重新分配 rank。
- 如果 provider 返回了结果但全部缺少可信 URL / domain，后端会继续降级，不把它当 live evidence。

## 前端变化

本轮没有改前端。

原因是 API 响应字段没有变化，前端仍消费已有的 `status`、`data.items`、`sources`、`evidence`、`provenance`、`degradedReason` 字段。

## 用户可感知变化

后续真实外部搜索 provider 配置好后：

- 用户问“最新趋势 / 竞品公开资料 / 平台规则 / 真实评价”等问题时，系统更容易识别不同 provider 返回的搜索来源。
- 前端展示的实时来源会更可靠，因为只有可追溯 URL 的来源才会进入证据链。
- 如果 provider 只给了不可点击、不可核验的标题摘要，系统不会假装搜索成功，而是明确降级。

## 验收结论

通过。

本轮完成的是外部搜索 skill 的后端证据内核增强，不解决真实 provider 账号、搜索额度、淘宝/抖音官方接口接入问题；这些仍属于后续 adapter 部署和平台接入阶段。
