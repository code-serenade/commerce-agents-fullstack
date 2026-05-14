# 迁移开发记录

## Post 8-64：搜索类 skill 自然触达补强

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 64 个补强项：`Post 8-64`。

本次目标：

- 继续补强旧产品外部搜索 / 实时资料能力在新产品 Agent 里的真实可用性。
- 让已有 `search_*` adapter 契约能被自然语言触发。
- 不新增假前端入口，不伪造联网搜索结果。

本次修改：

- 后端 agent：worker autosuggest 新增搜索类 skill 触达逻辑。
- 后端 agent：覆盖趋势、竞品、市场行情、商品评价、平台政策、行业基准 6 类搜索 skill。
- 后端 agent：识别“搜索/联网/实时/最新/公开来源/公开资料/真实评价”等外部搜索意图。
- 后端 agent：保留普通竞品分析、趋势预测、关键词研究、创意趋势捕捉等 native skill 的优先级。
- 后端角色目录：补齐 ops/data/service/creative/web 角色对相关搜索 skill 的白名单权限。
- API/SSE：同步说明自然语言可能自动产生 `search_*` 的 `skill_requested`。
- 迁移文档：同步阶段验收记录、总计划、矩阵和本轮归档文件。

用户可感知变化：

- 用户说“联网搜一下咖啡饮品最近趋势”时，后端能自动进入 `search_trends`。
- 用户说“查抖音最新平台规则”时，后端能自动进入 `search_platform_policy`。
- 用户说“搜冷萃咖啡用户差评和真实口碑”时，后端能自动进入 `search_product_reviews`。
- 如果外部搜索 provider 没配置，系统会明确返回未接入/降级状态，不会编造实时来源。

当前验证：

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/search_skill_reachability_post_8_64.md`
- [x] Agent autosuggest 单测覆盖 6 类搜索自然触达
- [x] Agent autosuggest 单测覆盖 native skill 不被搜索覆盖
- [x] Agent 搜索执行器 deferred/degraded 契约回归
- [x] 角色白名单回归

后续缺口：

- 真实搜索 provider 配置和线上稳定性仍待环境侧完成。
- 前端来源面板后续应更清楚展示 live/degraded/deferred 三种状态。

## Post 8-63：店铺指标查询 skill 自然触达补强

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 63 个补强项：`Post 8-63`。

本次目标：

- 继续补强旧产品 data skill 生态在新产品 Agent 里的真实可用性。
- 让 Post 8-62 已实现的 `query_store_metrics` 可以被自然语言触发。
- 避免前端做假入口，也避免用户必须懂内部 skill JSON。

本次修改：

- 后端 agent：worker autosuggest 新增 `query_store_metrics` 触达逻辑。
- 后端 agent：支持从用户文本解析 GMV、订单、UV、点击、广告费、退款、毛利。
- 后端 agent：支持中文金额单位 `万/千/亿`。
- 后端 agent：无平台明确输入时使用 `platform=all`，不默认淘宝。
- 后端 agent：保留 `data_dashboard`、竞品、渠道、退款、异常、趋势等更具体 skill 的优先级。
- API/SSE：同步说明自然语言可能自动产生 `query_store_metrics` 的 `skill_requested`。
- 迁移文档：同步阶段验收记录、总计划和本轮归档文件。

用户可感知变化：

- 用户说“查询店铺指标：GMV18万，订单1800，UV3.6万”时，后端能自动进入店铺指标查询 skill。
- 用户说“先加载店铺指标作为数据基线”时，后端能明确返回缺真实指标或待接 adapter，不会假装读取平台。
- 没指定平台时不会默认淘宝，避免再次出现系统擅自塞默认商品/平台上下文的问题。

当前验证：

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/query_store_metrics_reachability_post_8_63.md`
- [x] Agent autosuggest 单测覆盖显式指标、缺指标加载意图、平台识别、看板意图不覆盖

后续缺口：

- service 层真实 `store_metrics` adapter 仍待实现。
- 前端过程面板后续要展示该 skill 的数据加载状态。

## Post 8-62：店铺指标查询 skill native 显式输入补强

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 62 个补强项：`Post 8-62`。

本次目标：

- 继续迁移旧产品 data skill 生态。
- 把 `query_store_metrics` 从固定 deferred 占位补强为新产品 Rust Agent 内的显式指标查询/汇总能力。
- 不接淘宝、抖音、千川官方系统，不伪造平台实时数据。

本次修改：

- 后端 agent：`query_store_metrics` spec 改为 deterministic native skill，并补充 GMV、订单、UV、点击、广告费、退款、毛利和多平台输入字段。
- 后端 agent：有本轮显式指标时返回 `status=success`、`adapter_status=manual_input`、`data_readiness=explicit_metrics`。
- 后端 agent：输出 `totals`、`derived_metrics`、`week_over_week`、`anomalies`、`platforms` 和明确数据来源。
- 后端 agent：没有显式指标时返回 `status=deferred`、`adapter_status=store_metrics_required`，并说明不会伪造平台实时数据。
- 后端 agent：多平台指标支持按 `platform` 过滤和汇总。
- API/SSE：同步该 skill 的 `skill_result.output` 语义。
- 迁移文档：同步迁移矩阵、阶段验收记录、总计划和本轮归档文件。

用户可感知变化：

- 如果用户提供“GMV、订单、UV、广告费”等真实指标，Agent 可以先生成结构化店铺指标摘要，后续运营、数据、财务分析链路可以继续使用。
- 如果没有指标，系统会明确告诉用户缺少数据或 adapter，不会给出假平台大盘。
- 前端后续可以在过程面板中区分“本轮显式输入已读取”和“等待 store_metrics adapter”。

当前验证：

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/query_store_metrics_native_post_8_62.md`
- [x] 迁移矩阵：`MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- [x] API 文档：`API_DOCS/sse.md`
- [x] Agent 单测：显式指标成功、无指标 deferred、多平台过滤

后续缺口：

- service 层真实 `store_metrics` adapter 仍待实现。
- 官方平台数据 API 仍待后续 adapter 决策。
- 该 skill 当前不直接创建工作区 artifact，主要作为数据加载/过程 skill 服务后续链路。

## Post 8-61：技能链规划 skill native 补强

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 61 个补强项：`Post 8-61`。

本次目标：

- 继续迁移旧产品 coordination skill 生态。
- 把 `coordination_skill_chain_planner` 从静态建议补强为新产品 Rust Agent 内的结构化技能链规划能力。
- 为后续 Kimi Agent 风格过程面板提供真实后端数据，而不是让前端猜测步骤、状态和风险。

本次修改：

- 后端 agent：`coordination_skill_chain_planner` 输出新增 `status=success`、`角色`、`动作`、`前置依赖缺口`、`不可用技能`、`审批提示`、`adapter提示` 和 `可用技能过滤`。
- 后端 agent：每个推荐步骤新增 `阶段`、`负责角色`、`执行模式`、`风险等级`、`前置依赖`、`可用` 和说明。
- 后端 agent：把旧产品常见 `data_query_store_metrics` 规范为新产品真实 `query_store_metrics`，避免推荐不存在的旧 ID。
- 后端 agent：补强 NPS、SLA、CTR、ABC-XYZ、广告疲劳、RFM、SEO、财务、预测、库存、大促、新品、数据、客服、创意等电商场景模板。
- API/SSE：同步 `coordination_skill_chain_planner` 的 `skill_result.output` 契约说明。
- 迁移文档：同步迁移矩阵、阶段验收记录、总计划和本轮归档文件。

用户可感知变化：

- 后续前端接入后，用户能看到一条任务应该先加载数据、再分析、再生成内容或进入审批的清晰路径。
- 每个步骤都能显示负责的虚拟员工、执行阶段、是否需要 adapter、是否需要审批、当前是否可用。
- 外部数据和审批步骤会被明确标出，不会被展示成已经完成。
- 这轮不新增前端入口，不执行外部 Agent，也不发布平台动作。

当前验证：

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/skill_chain_planner_native_post_8_61.md`
- [x] 迁移矩阵：`MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- [x] API 文档：`API_DOCS/sse.md`
- [x] `cargo fmt --all`
- [x] `cargo test -p agent coordination_skill_chain -- --nocapture`

后续缺口：

- 前端仍需把结构化技能链接入右侧过程面板，并做成可折叠的过程卡片。
- 当前只做 deterministic planning，不自动批量执行技能链。
- `coordination_external_agent_call` 仍需要白名单、鉴权、审批、审计和失败降级后再真实接入。

## Post 8-60：外部搜索证据契约补强

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 60 个补强项：`Post 8-60`。

本次目标：

- 继续补强旧产品 search skill 生态在新产品里的后端落点。
- 不接淘宝、抖音、千川官方系统，不伪造实时来源。
- 把旧产品 `_needs_fresh_data` 实时搜索语义迁入新产品 Rust Agent search adapter。
- 让 App API、Agent `search_*` skill 和工作区搜索证据 artifact 输出统一证据摘要。

本次修改：

- 后端 agent：`search_*` skill 输出新增 `query.bypass_cache` 和 `evidence`。
- 后端 agent：支持 `bypass_cache`、`bypassCache`、`_needs_fresh_data`，并转发给外部搜索 provider。
- 后端 agent：来源归一化增加 URL/标题去重，避免重复来源污染证据。
- 后端 service：`ExternalSearchResponse` 新增 `maxResults`、`bypassCache` 和 `evidence`。
- 后端 service：`search_evidence` artifact metadata 新增 `evidence`，便于前端来源面板直接消费。
- API/SSE：同步新增字段和搜索证据语义。
- 迁移文档：同步迁移矩阵、阶段验收记录、总计划和本轮归档文件。

用户可感知变化：

- 用户提出需要最新趋势、竞品、评价、平台政策或行业基准的问题时，后端能保留“需要最新数据”的意图并传给 provider。
- 如果 provider 返回真实来源，系统能输出更清楚的来源标题、来源域名、证据摘要和可复用来源文本。
- 如果 provider 不可用或没有可信来源，系统仍会明确降级，不会编造搜索结果。
- 后续前端可以基于这些字段做右侧来源面板，而不是解析杂乱 provider 原始响应。

当前验证：

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/external_search_evidence_contract_post_8_60.md`
- [x] 迁移矩阵：`MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- [x] API 文档：`API_DOCS/app.md`
- [x] API 文档：`API_DOCS/sse.md`
- [x] `cargo fmt --all`
- [x] `cargo test -p agent search -- --nocapture`
- [x] `cargo test -p service external_search -- --nocapture`

后续缺口：

- 真实部署仍需要配置可访问的 `[external_search]` provider。
- 前端还需要把 `evidence/sources/provenance/confidence/freshness` 做成 Kimi Agent 风格来源面板。
- 官方平台直接 API 仍暂缓，不在本阶段接淘宝、抖音、千川官方系统。

## Post 8-59：综合业务诊断 skill native 编排迁移

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 59 个补强项：`Post 8-59`。

本次目标：

- 将旧产品 `DataComprehensiveDiagnosis` 迁为新产品 Rust Agent native skill。
- 继续使用同事给的新产品框架，不引入旧产品 Python skill 运行链路。
- 组合已有 Rust 数据 skill 输出，形成综合诊断，而不是伪造平台指标。

本次修改：

- 后端 agent：`data_comprehensive_diagnosis` 从 deferred spec 改为 deterministic/orchestrator spec，并新增 `values`、`channels`、`items` 等综合输入。
- 后端 agent：编排经营看板、漏斗、异常、客户分层、退款、需求预测、渠道归因和 LTV 分项。
- 后端 agent：输出整体健康评分、综合诊断报告、分项诊断、核心发现、风险预警、增长机会、7 天行动计划和缺失数据。
- 后端 agent：整体无有效用户数据时返回 `status=deferred`，不创建假综合报告。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增 `data_comprehensive_diagnosis_report` artifact 映射。
- API/SSE：同步 `data_comprehensive_diagnosis` 支持名单和 artifact 类型说明。
- 迁移文档：同步 `skill_migration_matrix.md`、本开发日志、阶段验收记录和本轮归档文件。

用户可感知变化：

- 用户提供经营、漏斗、时序、渠道、退款或客户数据后，数据分析师可以生成一份综合业务诊断，而不是只返回单个分项报告。
- 报告会同时说明哪些分项已经基于真实输入计算，哪些分项缺数据，避免误以为系统读取了外部平台后台。
- 前端后续接入后，该综合诊断可以作为工作区交付物沉淀，便于汇报和复盘。

当前验证：

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/data_comprehensive_diagnosis_native_post_8_59.md`
- [x] 迁移矩阵：`MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- [x] API 文档：`API_DOCS/app.md`
- [x] API 文档：`API_DOCS/sse.md`
- [x] `cargo fmt --all`
- [x] `cargo fmt --all --check`
- [x] `cargo test -p agent --lib data_comprehensive_diagnosis -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_data_comprehensive_skill_output -- --nocapture`
- [x] `cargo check -p agent -p service`
- [x] `cargo clippy -p agent -p service`

后续缺口：

- 真实店铺指标、淘宝、抖音、千川和数仓 adapter 仍未接入。
- 前端仍需展示综合诊断 artifact、缺失数据和分项状态。
- 后续可以继续迁移更细的经营诊断/教学/导出能力。

## Post 8-58：渠道归因分析 skill native 迁移

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 58 个补强项：`Post 8-58`。

本次目标：

- 将旧产品 `DataAttributionAnalysis` 迁为新产品 Rust Agent native skill。
- 继续使用同事给的新产品框架，不引入第二套 Python skill 运行链路。
- 不接淘宝、抖音、千川或数据仓库官方接口；只基于用户或未来 adapter 显式提供的渠道数据计算。

本次修改：

- 后端 agent：`data_attribution_analysis` 从 deferred spec 改为 deterministic native spec，并新增 `channels` 入参。
- 后端 agent：实现末次归因、线性归因、时间衰减归因、位置归因四种模型。
- 后端 agent：缺渠道、少于两个有效渠道或总 GMV 为 0 时返回 `status=deferred`，不伪造平台指标。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增 `data_attribution_report` artifact 映射。
- API/SSE：同步 `data_attribution_analysis` 支持名单和 artifact 类型说明。
- 迁移文档：同步 `skill_migration_matrix.md`、本开发日志、阶段验收记录和本轮归档文件。

用户可感知变化：

- 用户提供多渠道 GMV、UV、转化率、广告花费后，数据分析师可以输出多模型渠道归因、关键洞察和预算再分配建议。
- 前端后续接入后，该结果可以作为工作区交付物查看，不只停留在一次流式回答里。
- 如果用户没有提供渠道数据，系统会明确提示缺数据，而不是假装读取了淘宝/抖音后台。

当前验证：

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/data_attribution_native_post_8_58.md`
- [x] 迁移矩阵：`MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- [x] API 文档：`API_DOCS/app.md`
- [x] API 文档：`API_DOCS/sse.md`
- [x] `cargo fmt --all`
- [x] `cargo fmt --all --check`
- [x] `cargo test -p agent --lib data_attribution_analysis -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_data_advanced_skill_output -- --nocapture`
- [x] `cargo check -p agent -p service`
- [x] `cargo clippy -p agent -p service`

后续缺口：

- 真实平台/数仓 adapter 仍未接入；未来应通过 adapter 注入 `channels`，不要在 agent crate 直连数据库或平台。
- `data_comprehensive_diagnosis` 仍待迁移，它可以在后续组合归因、漏斗、异常、退款和预测结果。
- 前端仍需展示归因明细和 artifact 链接。

## Post 8-57：高级财务 skill 输出契约与 artifact 沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 57 个补强项：`Post 8-57`。

本次目标：

- 把已经迁入新产品 Agent 框架的高级财务/经营分析 skill，从“能计算”推进到“能按统一后端契约返回，并能沉淀为 workspace artifact”。
- 继续以后端为主，不新增前端假入口，不接淘宝、抖音、千川等官方平台 API。
- 保持新产品架构：skill 输出统一由 Agent runtime 产生，service 层只在 `status=success` 且命中支持名单时创建交付物。

本次修改：

- 后端 agent：`accounting_break_even_calc`、`accounting_compliance_check`、`accounting_pl_statement`、`accounting_budget_vs_actual`、`accounting_gmv_waterfall`、`accounting_scenario_analysis`、`accounting_cash_flow_forecast`、`accounting_financial_narrative` 补齐 `status`、`has_data`、`data_readiness`、`adapter_status`、`summary`。
- 后端 agent：缺少必要核心输入的高级财务 skill 返回 `status=deferred`，不会创建假财务报告。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增高级财务 artifact 映射。
- API/SSE：同步高级财务 artifact 类型和 `workspace_artifact_created` 契约说明。
- 迁移文档：同步 `skill_migration_matrix.md`、本开发日志、阶段验收记录和本轮归档文件。

用户可感知变化：

- 用户让财务岗位做盈亏平衡、合规检查、利润表、预算执行、GMV瀑布、情景分析、现金流预测或财务健康诊断时，后端成功结果可以沉淀为工作区交付物。
- 前端后续接入后，可以在交付物/过程区看到这些报告，而不是只在流式回答里一闪而过。
- 如果用户没有提供必要财务数据，后端会明确返回 `deferred`，不会伪造平台财务数据或默认生成看似真实的报告。

当前验证：

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/accounting_advanced_artifact_materialization_post_8_57.md`
- [x] 迁移矩阵：`MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- [x] API 文档：`API_DOCS/app.md`
- [x] API 文档：`API_DOCS/sse.md`
- [x] `cargo fmt --all`
- [x] `cargo fmt --all --check`
- [x] `cargo test -p agent accounting_ -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_accounting_skill_output -- --nocapture`
- [x] `cargo check -p agent -p service`
- [x] `cargo clippy -p agent -p service`
- [x] `git diff --check`

后续缺口：

- 当前高级财务 skill 仍基于用户输入和内置确定性规则计算，不读取真实店铺账务、广告账户、平台订单或 ERP 数据。
- 若要接入真实淘宝、抖音、千川、ERP 或财务系统，需要后续设计 adapter、权限、审批和审计链路。
- 前端仍需把财务岗位的报告类 artifact 展示到工作区、过程面板和历史运行详情里。

## Post 8-56：客服商品查询 skill 读取显式商品上下文

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 56 个补强项：`Post 8-56`。

本次目标：

- 将旧产品 `server/src/skills/customer_service.py::ServiceQueryProduct` 迁为新产品 Agent 框架内的 native skill。
- 保持新产品架构边界：`agent` crate 不直接查询数据库，不新增第二套商品查询服务。
- 保持用户前面强调的商品通路规则：只有本轮手动选择/引用的商品上下文才能用于指定商品回答；未指定商品时不默认注入任何商品。

本次修改：

- 后端 agent：`service_query_product` 从 `deferred` spec 改为 `deterministic + data_source` native spec。
- 后端 agent：新增 `ServiceQueryProduct` 独立实现，读取 `SkillContext.metadata` 中的 `request.context.*` / `context.*` 商品 ID、refs、summary、sources。
- 后端 agent：支持旧产品整数商品 ID 和新产品字符串商品 ID。
- 后端 agent：有显式商品上下文且筛选命中时返回 `status=success`、`adapter_status=context_metadata`、`data_readiness=explicit_product_context`。
- 后端 agent：无显式商品上下文或商品 ID/名称筛选不命中时返回 `status=deferred`，明确说明不会改用默认商品或隐式搜索其它商品。
- API/SSE：不新增事件类型；更新 `API_DOCS/sse.md`，说明该 skill 通过既有 `skill_result` 承载上下文查询结果。
- 迁移文档：同步 `skill_migration_matrix.md`、本开发日志、阶段验收记录和本轮归档文件。

用户可感知变化：

- 当用户在前端“引用商品”里手动勾选某个商品后，客服/运营/内容 Agent 调用 `service_query_product` 时可以读取该商品资料，过程区会看到真实 `success/native` skill 结果。
- 当用户没有勾选商品时，系统不会再把某个默认商品当作上下文；商品查询 skill 会明确返回“需要显式商品上下文”。
- 当用户勾选了 A 商品却询问 B 商品名时，后端不会把 A 商品强行当成 B 商品回答，而是返回筛选不命中。

当前验证：

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/service_query_product_native_post_8_56.md`
- [x] 迁移矩阵：`MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- [x] API 文档：`API_DOCS/sse.md`
- [x] `cargo fmt --all`
- [x] `cargo test -p agent service_query_product`
- [x] `cargo test -p agent executor_runs_service_query_product_with_context`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `git diff --check`

后续缺口：

- `service_query_product` 当前只读取 Agent 显式上下文，不做实时平台商品库查询；如果未来要按商品 ID 直接查用户商品库，应在 service/repo/web-server 层设计正式 adapter 或上下文注入链路，而不是在 agent crate 里直连数据库。
- 平台最新退换货、投诉、赔付规则仍需要 `search_platform_policy` 或 `shop_policy` adapter。
- 前端仍需把“引用商品/未引用商品”的状态和 skill 结果清楚展示，避免用户误以为系统用了默认商品。

## Post 8-55：外部搜索提升为第 2 优先级后的 RAG/SSE 可用性契约

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 55 个补强项：`Post 8-55`。

本次目标：

- 按最新要求，把“外部搜索 / 实时信息增强”从后续第 5 优先级提升到第 2 优先级。
- 先补后端主链路契约，让前端后续能按真实状态展示来源和降级原因。
- 不接淘宝、抖音、千川等官方平台直接 API，不伪造外部搜索结果。

涉及范围：

- 后端 service：`BuildAgentRetrievalContextRequest` 增加 `external_search_available`。
- 后端 service：本地 RAG 未命中时，降级原因会区分外部搜索 adapter 未配置和已配置。
- 后端 web-server：Agent SSE 从 `[external_search]` 配置读取 `is_configured()`，并写入 RAG response 和 runtime metadata。
- API 文档：`API_DOCS/sse.md` 更新 `rag_context_loaded.externalSearchAvailable` 语义。
- 迁移计划：总计划把外部搜索提升为第 2 优先级，平台官方直接集成暂缓。

用户可感知变化：

- 用户问实时趋势、竞品、平台政策等问题时，系统能更准确说明当前是否具备外部搜索 adapter。
- 如果本地商品知识没有命中，前端后续可以明确展示“没有本地来源但可请求实时搜索”或“没有本地来源且外部搜索未配置”。
- 系统仍不会假装已经搜到外部资料；只有同一轮 `search_*` skill 返回 live 来源时，才可展示搜索证据。

当前验证：

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/agent_external_search_priority_post_8_55.md`
- [x] 迁移总计划：`temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- [x] API 文档：`API_DOCS/sse.md`
- [x] `cargo fmt`
- [x] `cargo test -p service agent_context`
- [x] `cargo test -p web-server retrieval_metadata`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`
- [x] `git diff --check`

后续缺口：

- 前端还需要把外部搜索可用性、搜索证据 artifact、来源、域名、发布时间、置信度整理成右侧来源面板。
- 真实部署仍需要配置 `[external_search]` provider。
- 官方平台直接 API 仍暂缓，等内部问答、skill 和外部搜索体验更稳定后再做。

## Post 8-54：问答商品上下文显式选择与工作台遮挡修正

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 54 个补强项：`Post 8-54`。

本次目标：

- 修正问答页里商品上下文可能被误用的问题：用户没有手动选择预设商品时，后端不得把已有商品、商品知识、商品素材或工作区绑定商品当作默认上下文。
- 让“预设商品 / 未指定商品”的入口在输入区更明显，避免用户找不到商品选择位置。
- 按 Kimi Agent 集群参考图继续收敛左右折叠体验：左上角避免重复折叠按钮，右侧虚拟员工区不再覆盖正文。
- 调整任务过程卡片顺序和占位：过程卡片放在用户提问之后、正文回答之前；回答结束后默认折叠，用户需要时再展开。

涉及范围：

- 后端 service：`BuildAgentRetrievalContextRequest` 增加 `allow_implicit_products`，SSE 问答链路固定为 `false`，禁止通过“咖啡”等关键词隐式命中用户已有商品。
- 后端 workspace：`GetWorkspaceContextSummaryRequest` / `GetWorkspaceContextPackRequest` 增加 `include_workspace_product_context`，普通工作区 API 保持原行为，Agent SSE 问答链路不再回退到工作区绑定商品。
- SSE：只有本轮显式传入 `metadata.context.productIds` 或 `product:{id}` context ref 时，才把商品 RAG 摘要合入 runtime metadata。
- API 文档：`API_DOCS/sse.md` 明确“未选择商品 = 不注入任何默认商品上下文”。
- 前端输入区：任务设置摘要显示“未指定商品”或已选商品名，点击商品 chip 可展开选择。
- 前端商品加载：问答页商品选择从只查 `active` 扩展为读取 `active/listing/draft`，覆盖预售/上架中/草稿商品。
- 前端工作台：右侧虚拟员工区改成真实分栏/堆叠区域，收起后是布局内按钮，不再浮在正文上遮挡内容。
- 前端过程卡片：按最新用户消息挂载，显示在回答正文之前；运行时展开，完成后默认折叠，保留手动展开能力。

用户可感知变化：

- 用户进入问答页时，左上角不会再出现两个功能重叠的侧栏折叠入口。
- 用户不选择商品时，输入区会明确显示“未指定商品”；此时即使问题里写了“咖啡”，后端也不会自动拿某个已保存咖啡商品来回答。
- 用户点击“未指定商品 / 已选商品”后，可以看到预售/上架中、草稿、运营中的商品并手动勾选；只有勾选后该商品资料才进入本轮回答。
- 用户发送复杂问题时，读取目标、分配任务、进度和产物过程会先出现在问题下面，正文回答在后面输出，阅读顺序更清楚。
- 回答结束后，过程卡片自动收起，主回答正文占据更多空间；需要复盘过程时可以再展开。
- 右侧虚拟员工工作区不再盖住主问答内容，展开时是独立右栏或窄屏下方区域，收起时保留不遮挡正文的入口。

当前验证：

- [x] `cargo fmt`（现有 nightly-only rustfmt 配置项仅产生警告）
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `cargo test -p service agent_context`
- [x] `cargo test -p service workspace::tests::context`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

浏览器验证说明：

- 本地前端 `http://localhost:5173/commerce` 和后端 `http://localhost:19878` 均在监听。
- 已用 Edge 打开产品页面供人工试用。
- Codex 内置浏览器插件本轮因 Windows 会话凭据错误无法接管截图，错误表现为 `CreateProcessWithLogonW failed: 1326`；因此本轮没有自动截图验证，但已完成构建、规则和后端链路验证。

后续缺口：

- 仍建议在 Edge 中用 `test123456 / test123456` 登录后真实发送一轮“不选商品”和“选择预设商品”的对比问题，人工确认回答内容差异。
- 更完整的 Kimi Agent 集群视觉还可以继续细化，但本轮已先解决遮挡、重复折叠入口和过程卡片阅读顺序。
- 真实外部搜索 provider、平台写 adapter 和更多旧 skill 生态仍按后端优先计划继续做。

## Post 8-53：问答工作台折叠收纳与演示清爽化

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 53 个补强项：`Post 8-53`。

本次目标：

- 按用户反馈解决问答页“所有东西都满满当当”的问题。
- 今天向公司汇报时，第一屏优先呈现核心问答，不让左侧导航、右侧过程工作区和输入区高级配置同时抢占视线。
- 不新增后端能力，不新增 API，不伪造未完成 adapter；只把已对齐后端的前端入口做折叠收纳。
- 继续基于同事给的 `SidebarShell`、页面组件和现有 app-kit/ui-components 框架，不另写第二套 UI。

涉及范围：

- app-kit：`SidebarShell` 新增 `defaultSidebarOpen`，默认仍保持展开；只有电商多智能体页显式默认收起。
- 电商多智能体 shell：左侧导航在 `/commerce`、`/agent` 和首页 Agent 路由默认收成图标栏，仍可点击原框架按钮展开。
- 问答页：右侧虚拟员工工作区默认收起为浮动入口，需要查看过程时再展开。
- 输入区：策略、商品上下文、运行模型、岗位选择和 playbook 场景入口被收进“任务设置”，默认只显示一行当前摘要。
- 过程卡片：把内联步骤从纵向大块改为横向紧凑条，减少主问答区被过程卡片撑满。

用户可感知变化：

- 用户进入电商多智能体页面时，首先看到更清爽的主问答区。
- 左侧菜单默认只保留图标，占用空间更少；需要完整菜单时仍可展开。
- 右侧虚拟员工工作区默认不挡住主问答；点击右侧浮动按钮可展开查看 Agent 集群过程。
- 输入框上方不再铺满策略、商品、模型、岗位和场景按钮；点击“任务设置”后仍能使用这些已接后端能力。
- 这次没有减少后端能力，只是让前端默认状态更适合汇报和真实使用。

当前验证：

- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend/packages/app-kit rules`
- [x] `pnpm -C frontend/packages/app-kit typecheck`

后续缺口：

- 仍需浏览器登录后检查真实问答、任务设置展开、右侧工作区展开和左侧导航展开。
- 真实平台写 adapter、外部搜索 provider、全量旧 skill 生态仍按后端优先计划继续做。

## Post 8-52：演示可用前端对齐

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 52 个补强项：`Post 8-52`。

本次目标：

- 今天需要向公司汇报产品，优先让前端围绕后端已经做好的能力变得可用、可演示。
- 不新增后端能力，不新增第二套 UI 框架，不伪造未接入平台 adapter 的成功状态。
- 继续基于同事给的新产品框架，在 `apps/web` 页面装配层消费真实后端契约。

涉及范围：

- 产品体验：问答页默认呈现“主问答区 + 右侧虚拟员工工作区”，避免第一屏退化成普通聊天框。
- 前端页面：审批页新增后端执行结果列，展示 `executionStatus`、`adapterStatus`、`retryable`、`providerResult` 和幂等键。
- 前端页面：平台连接页展示执行 adapter 的必填参数、幂等字段和可执行边界。
- 语言文案：平台和审批页面文案对齐真实后端能力，说明真实平台写 adapter 尚未接入时会显示不可执行。
- 架构边界：只改 `frontend/apps/web` 页面装配和 lang，不改共享组件包，不新建第二套组件体系。

用户可感知变化：

- 用户进入电商多智能体问答页时，会直接看到右侧虚拟员工工作区。
- 用户可以折叠/展开右侧工作区，问答主区域和过程区域更适合今天演示。
- 平台连接页能讲清楚当前后端已有 adapter catalog、readiness、必填参数和幂等字段。
- 审批页能讲清楚审批后的后端执行 outcome；真实平台 adapter 未接入时显示真实失败边界，不显示假成功。

当前验证：

- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`

后续缺口：

- 真实平台写 adapter 仍待后续接入 provider registry / adapter trait。
- 左侧历史/导航折叠和更完整的 Kimi Agent 集群体验仍可继续优化。
- 仍需启动产品做浏览器登录、问答、平台连接和审批页面 smoke。

## Post 8-51：平台执行 Adapter 调度边界

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 51 个补强项：`Post 8-51`。

本次目标：

- 继续基于同事给的新产品框架推进，不引入第二套执行框架，不复制旧产品代码。
- 把审批通过后“应当调用 adapter 还是降级失败”的判断收敛到 `service::api::platform_execution`。
- 保持当前真实平台 adapter 未实现的边界，不新增平台写执行，不伪造执行成功。

涉及范围：

- 产品内核：平台执行现在有统一 dispatch outcome 入口，覆盖 `validation_error`、`connection_missing`、`connection_disabled`、`unsupported_action` 和 `adapter_not_implemented`。
- 后端 service：`platform_execution.rs` 新增 `platform_execution_dispatch_outcome()`，复用 readiness 和 adapter outcome 契约。
- 后端审批闭环：`approval_audit` 删除重复的平台执行判断分支，改为只调用 dispatch outcome，再负责 execution queue、tool approval 和 audit 写回。
- Web-server/API：不新增路由，既有响应字段保持兼容。
- 前端：本轮未修改前端，也未新增假执行按钮或假成功状态。

用户可感知变化：

- 当前页面表现基本不变，已连接平台仍会明确返回 `adapter_not_implemented`。
- 审批执行详情背后的失败原因来自同一个后端内核入口，后续前端展示会更稳定一致。
- 这一步是为真实平台 adapter 接入做准备，本轮没有调用淘宝、京东、拼多多或抖音等真实平台写接口。

当前验证：

- [x] `cargo fmt --all`（在 `backend/` workspace 下运行；现有 nightly-only rustfmt 配置项仅产生警告）
- [x] `cargo test -p service platform_execution -- --nocapture`
- [x] `cargo test -p service plan_platform_execution -- --nocapture`
- [x] `cargo test -p service approval_execution_readiness -- --nocapture`
- [x] `cargo test -p service platform_execution_preflight -- --nocapture`

## Post 8-50：平台执行 Adapter Outcome 契约

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 50 个补强项：`Post 8-50`。

本次目标：

- 在真实平台 adapter 接入前，先补齐平台执行 adapter outcome 契约。
- 让审批通过后的平台执行失败结果由共享 outcome 结构生成，而不是继续散落 JSON 拼接。
- 保持当前真实平台 adapter 未实现的边界，不新增平台写执行，不伪造执行成功。

涉及范围：

- 产品内核：平台执行失败结果现在有统一 outcome 语义，包含 `status`、`adapterStatus`、`message`、`adapterRequest`、`providerResult` 和 `retryable`。
- 后端 service：`platform_execution.rs` 新增 `PlatformExecutionAdapterOutcome`、`platform_execution_adapter_outcome()` 和 `platform_execution_unimplemented_outcome()`。
- 后端审批闭环：`approval_audit` 的平台执行失败结果改为从 outcome 写入，当前未实现 adapter 返回 `providerResult=null`、`retryable=false`。
- Web-server/API：不新增路由，`ToolApprovalResponse.result` 和 `ExecutionQueueResponse.result` 只新增可选诊断字段。
- 前端：本轮未修改前端，也未新增假执行按钮或假成功状态。

用户可感知变化：

- 用户或后续前端查看平台执行失败详情时，可以区分“平台 provider 原始返回”和“当前是否可重试”的语义。
- 当前仍会明确显示 `adapter_not_implemented`，不会让用户误以为已经能真实改价、改库存或同步平台商品。
- 这个切片为后续真实平台 adapter 的成功、失败、降级和重试结果打基础。

当前验证：

- [x] `cargo fmt --all`（在 `backend/` workspace 下运行；现有 nightly-only rustfmt 配置项仅产生警告）
- [x] `cargo test -p service platform_execution -- --nocapture`
- [x] `cargo test -p service plan_platform_execution -- --nocapture`
- [x] `cargo test -p service approval_execution_readiness -- --nocapture`
- [x] `cargo test -p service platform_execution_preflight -- --nocapture`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`

## Post 8-49：平台执行 Adapter 请求信封

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 49 个补强项：`Post 8-49`。

本次目标：

- 在真实平台 adapter 接入前，先补齐平台执行的规范化 adapter request envelope。
- 让审批通过后的执行队列失败结果清楚记录“本来准备如何调用 adapter”。
- 保持当前真实平台 adapter 未实现的边界，不新增平台写执行，不伪造执行成功。

涉及范围：

- 产品内核：支持的平台执行动作现在能生成 `adapterRequest`，包含 `skillName`、`platform`、`executionMode`、`idempotencyKey`、`idempotencyKeyFields` 和规范化 `args`。
- 后端 service：`platform_execution.rs` 新增 `PlatformExecutionAdapterRequest`、`platform_execution_adapter_request()` 和基于 action spec 的幂等键生成。
- 后端审批闭环：`approval_audit` 的平台执行失败结果会写入 `adapterRequest`；unsupported 动作或缺平台时该字段为 `null`。
- Web-server/API：不新增路由，`ToolApprovalResponse.result` 和 `ExecutionQueueResponse.result` 只新增可选诊断字段。
- 前端：本轮未修改前端，也未新增假执行按钮或假成功状态。

用户可感知变化：

- 用户或后续前端查看审批后的执行失败详情时，可以看到更清晰的 adapter 请求边界和幂等键。
- 当前仍会明确显示 `adapter_not_implemented`，不会让用户误以为已经能真实改价、改库存或同步平台商品。
- 这个切片为后续真实平台 adapter、重试和审计追踪打基础。

当前验证：

- [x] `cargo fmt --all`（在 `backend/` workspace 下运行；现有 nightly-only rustfmt 配置项仅产生警告）
- [x] `cargo test -p service platform_execution -- --nocapture`
- [x] `cargo test -p service plan_platform_execution -- --nocapture`
- [x] `cargo test -p service approval_execution_readiness -- --nocapture`
- [x] `cargo test -p service platform_execution_preflight -- --nocapture`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`

## Post 8-48：平台执行动作规格注册表

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 48 个补强项：`Post 8-48`。

本次目标：

- 把平台执行 adapter 能力目录里的动作规格迁入 service 内部共享模块。
- 让 catalog、preflight、审批 readiness 和执行失败详情继续靠近同一套后端事实来源。
- 保持当前真实平台 adapter 未实现的边界，不新增平台写执行，不伪造执行成功。

涉及范围：

- 产品内核：`platform_update_price`、`platform_update_inventory`、`platform_sync_products` 的展示名、描述、required/optional 参数、幂等字段和 adapter 未实现文案现在来自共享动作规格注册表。
- 后端 service：`platform_execution.rs` 新增 `PlatformExecutionActionSpec` 和 `platform_execution_action_specs()`；`platform_connect` 改为消费共享 spec 生成 catalog。
- Web-server/API：不新增路由，`GET /api/app/platform-connect/execution-adapters` 和 `POST /api/app/platform-connect/execution-preflight` 响应结构保持不变。
- 前端：本轮未修改前端，也未新增假执行按钮或假成功状态。

用户可感知变化：

- 用户在平台连接页看到的动作目录，和 preflight / 审批链路背后的支持动作判断更一致。
- 当前仍会明确显示 `adapter_not_implemented`，不会让用户误以为已经能真实改价、改库存或同步平台商品。
- 这个切片继续为后续真实平台 adapter 接入打基础，避免后续新增动作时多处漏改。

当前验证：

- [x] `cargo fmt --all`（在 `backend/` workspace 下运行；现有 nightly-only rustfmt 配置项仅产生警告）
- [x] `cargo test -p service platform_execution -- --nocapture`
- [x] `cargo test -p service platform_execution_adapter_catalog -- --nocapture`
- [x] `cargo test -p service platform_execution_preflight -- --nocapture`
- [x] `cargo test -p service approval_execution_readiness -- --nocapture`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`

## Post 8-47：平台执行共享规则内核

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 47 个补强项：`Post 8-47`。

本次目标：

- 把平台执行 preflight、审批创建 readiness、审批通过后的执行失败详情统一到同一套后端规则来源。
- 继续复用同事给的新产品 service/API 架构，不新增第二套平台执行系统。
- 保持当前真实平台 adapter 未实现的边界：只给出真实不可执行原因，不伪造平台写成功。

涉及范围：

- 产品内核：平台写动作的参数解析、支持动作判断、连接状态判断和 adapter 未实现文案现在来自共享规则模块。
- 后端 service：新增 `backend/crates/service/src/api/platform_execution.rs`，供 `approval_audit` 和 `platform_connect` 复用。
- Web-server/API：不新增路由，`POST /api/app/platform-connect/execution-preflight`、审批 `executionReadiness` 和执行队列失败详情的外部响应结构保持不变。
- 前端：本轮未修改前端，也未新增假执行按钮或假成功状态。

用户可感知变化：

- 用户在创建平台写审批前看到的 preflight 结果、审批记录里的 readiness、审批后执行队列里的失败原因会更一致。
- 当前仍会明确显示缺参数、缺连接、连接禁用、不支持动作或 `adapter_not_implemented`，不会误导用户以为已经能真实改价、改库存或同步平台商品。
- 这个切片主要提升后端内核一致性，为后续真实平台 adapter 接入打基础。

当前验证：

- [x] `cargo fmt --all`（在 `backend/` workspace 下运行；现有 nightly-only rustfmt 配置项仅产生警告）
- [x] `cargo test -p service platform_execution -- --nocapture`
- [x] `cargo test -p service approval_execution_readiness -- --nocapture`
- [x] `cargo test -p service platform_execution_preflight -- --nocapture`
- [x] `cargo test -p service plan_platform_execution -- --nocapture`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`

## Post 8-46：平台执行 Preflight 预检契约

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 46 个补强项：`Post 8-46`。

本次目标：

- 把高风险平台写操作从“创建审批后才知道 readiness”继续前移到“创建审批前即可预检参数、连接和 adapter 状态”。
- 继续复用新产品已有 `platform_connections` 和平台执行治理规则，不新增第二套平台执行系统。
- 明确当前真实平台 adapter 尚未实现，preflight 只做只读校验，不创建审批、不创建执行队列、不调用平台写接口。

涉及范围：

- 产品内核：平台写动作现在有创建审批前的统一预检入口，可提前返回 `validation_error`、`connection_missing`、`connection_disabled`、`unsupported_action`、`adapter_not_implemented`。
- 后端 service：`PlatformConnectApi::execution_preflight` 支持读取顶层参数、`normalized_inputs` 和 `output.normalized_inputs`，与 Agent skill 产物和审批参数兼容。
- Web-server/API：新增 `POST /api/app/platform-connect/execution-preflight`，已同步 `API_DOCS/app.md`。
- 前端：同步 API 类型和请求函数，但本轮不新增假执行按钮或临时表单。

用户可感知变化：

- 后续前端或 Agent 创建平台写审批前，可以先调用后端预检，直接知道当前是否会因为参数、连接或 adapter 问题失败。
- 当前如果参数和连接都齐全，仍会明确返回 `adapter_not_implemented`，不会让用户误以为能真实改价、改库存或同步平台商品。
- 这个能力为后续审批创建表单、Agent 平台执行建议和真实 adapter 接入打底。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service platform_execution_preflight -- --nocapture`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

## Post 8-45：平台执行 Adapter 能力目录

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 45 个补强项：`Post 8-45`。

本次目标：

- 把高风险平台写操作从“审批记录里才看到失败原因”推进到“平台连接页能先看到所有 adapter 能力目录和可执行边界”。
- 继续复用新产品已有 `platform_connections`、审批和执行队列体系，不新增第二套平台执行系统。
- 明确当前真实平台 adapter 尚未实现，不伪造平台写成功，也不提供假执行入口。

涉及范围：

- 产品内核：平台写能力现在有只读能力目录，用户能先知道哪些平台动作受治理、需要哪些参数、为什么当前不能执行。
- 后端 service：`PlatformConnectApi::execution_adapter_catalog` 返回各平台连接状态和 `platform_update_price`、`platform_update_inventory`、`platform_sync_products` 三类高风险动作目录。
- Web-server/API：新增 `GET /api/app/platform-connect/execution-adapters`，已同步 `API_DOCS/app.md`。
- 前端：平台连接页新增只读“平台执行能力”面板，展示连接状态、adapter 状态、readiness 和必要参数，不新增执行按钮。

用户可感知变化：

- 用户进入平台连接页时，可以看到每个平台当前有哪些高风险执行能力。
- 如果平台连接缺失、连接被禁用或真实 adapter 未实现，页面直接展示后端返回的不可执行原因。
- 当前所有平台写动作仍不可直接执行，必须等后续真实 adapter 接入后继续走审批、执行队列和审计闭环。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service platform_execution_adapter_catalog -- --nocapture`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

## Post 8-44：平台执行 Readiness 诊断契约

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 44 个补强项：`Post 8-44`。

本次目标：

- 把高风险平台写操作的“批准后才知道失败”前移为“审批创建时就能看到执行可行性”。
- 继续使用新产品已有 `tool_approval_queue`、`execution_queue`、`audit_logs`、`platform_connections`，不新增第二套审批或执行系统。
- 明确当前真实平台 adapter 尚未实现，不伪造平台写成功。

涉及范围：

- 产品内核：平台写操作审批现在带有真实 readiness 诊断，审批不再等同于可执行成功。
- 后端 service：`ApprovalAuditApi::create_tool_approval` 会为 `platform_*` 审批写入 `result.executionReadiness`。
- 后端执行规划：未知平台动作优先返回 `unsupported_action`，参数、连接和 adapter 状态分别给出明确失败原因。
- API/SSE：`ToolApprovalResponse.result.executionReadiness` 和 `ExecutionQueueResponse.result` 已同步文档。
- 前端：审批列表和工作区执行建议表只展示后端真实 `adapterStatus/message`，不新增假执行入口。

用户可感知变化：

- 用户在审批页可直接看到平台写操作当前能不能执行，以及不能执行的原因。
- 如果缺平台连接、连接禁用、缺参数或真实 adapter 未实现，页面会用后端返回的诊断提示出来。
- 用户批准后，执行队列仍会显示真实失败原因，不会把批准误显示成平台执行成功。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service approval_execution_readiness -- --nocapture`
- [x] `cargo test -p service plan_platform_execution -- --nocapture`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

## Post 8-43：会话附件解析状态契约

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 43 个补强项：`Post 8-43`。

本次目标：

- 把旧产品“附件/素材可进入问答上下文”的能力继续补齐到新产品后端契约。
- 在既有会话附件抽取文本 API 上增加 `pending/extracted/failed/unsupported` 状态。
- 让工作区 `context-pack` 明确区分“已解析正文”“等待解析”“解析失败”“当前不支持”，避免 Agent 误读未解析文件。

涉及范围：

- 产品内核：附件从“只知道有无文本”推进到“有解析状态、失败原因和上下文可读边界”。
- 后端 service：`ConversationApi::register_file` 新上传附件默认写入 `metadata.extractionStatus=pending`；`update_file_extraction` 支持登记 `extracted/failed/unsupported/pending`。
- 后端 context：`WorkspaceApi::context_pack` 只在 `extractionStatus=extracted` 时读取附件文本；其它状态分别返回 `extraction_pending`、`extraction_failed`、`extraction_unsupported`。
- Web-server/API：扩展 `/api/app/conversation-files/{fileId}/extraction` 请求体字段，保持旧调用不传 `status` 时按 `extracted` 兼容。
- 前端：只同步 `apps/web` API adapter 类型，不新增假解析 UI。

用户可感知变化：

- 用户上传附件后，后端会把附件标记为“等待解析”，后续前端可据此展示真实待处理状态。
- 如果解析器登记成功，附件会进入 Agent 问答上下文，`context-pack` 显示 `extracted_text`。
- 如果解析失败或格式不支持，Agent 不会继续读取旧正文，右侧过程区可展示清晰警告和失败原因。
- 当前仍不实现 OCR、PDF/Office 解析、图片识别或 object storage 读取；这些要后续接 parser adapter。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service conversation_file_extraction -- --nocapture`
- [x] `cargo test -p service context_pack_respects_attachment_extraction_status -- --nocapture`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

## Post 8-42：Engineering native 工程报告沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 42 个补强项：`Post 8-42`。

本次目标：

- 把工程岗位中仍是骨架的架构评估、Bug 排查、性能优化、技术选型能力迁成 Rust native 输出。
- 补齐 SLA 监控的统一输出契约。
- 让工程岗位成功输出能沉淀为 workspace artifact，而不是只停留在 SSE 过程里。

涉及范围：

- 产品内核：工程虚拟员工现在能生成可审计、可复用的工程报告。
- 后端 Agent：`backend/crates/agent/src/skills/engineering.rs` 输出 `status=success`、`has_data=true`、`adapter_status=manual_input`、`data_readiness`、`summary` 和 `data_boundary`。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增 `engineering_arch_review_report`、`engineering_bug_analysis_report`、`engineering_perf_plan`、`engineering_tech_selection_report`、`engineering_sla_report`。
- API/SSE：更新 `API_DOCS/app.md` 与 `API_DOCS/sse.md`。
- 前端：本阶段不新增页面；后续按现有 artifact / SSE 契约展示真实工程报告。

用户可感知变化：

- 用户向工程岗位提问架构、Bug、性能、选型或 SLA 问题时，成功结果会进入工作区交付物。
- 当缺少真实上下文时，报告会以 `data_readiness=calculated_from_defaults` 明示是基于默认工程规则生成。
- 当前不假装读取真实代码仓库、日志、APM、监控系统或实时 benchmark；平台健康检查仍需要 adapter 和审批。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent engineering_ -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_engineering_ -- --nocapture`

## Post 8-41：Design 交付物契约补强

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 41 个补强项：`Post 8-41`。

本次目标：

- 把已经 native 化的设计类 skill 输出补齐为新产品统一成功契约。
- 让主图 brief、详情页 brief、配色方案、素材规范和活动海报 brief 的 workspace artifact 物化路径更稳定。
- 继续遵循新产品框架，不新增图片生成器、不新增前端假入口。

涉及范围：

- 产品内核：设计岗位从“能返回设计方案”进一步稳定为“能沉淀成可复用的设计交付物”。
- 后端 Agent：`backend/crates/agent/src/skills/design.rs` 统一成功输出字段：`status=success`、`has_data=true`、`adapter_status=manual_input`、`data_readiness`、`summary`。
- 后端 service：补充 `design_material_spec` artifact 物化测试，继续复用 `WorkspaceApi::materialize_agent_content_artifact`。
- API/SSE：更新 `API_DOCS/app.md` 与 `API_DOCS/sse.md`，说明设计类 artifact 的输出边界。
- 前端：本阶段不新增页面和假按钮；后续前端只需按已有 workspace artifact / SSE 事件展示真实设计 brief/spec。

用户可感知变化：

- 用户让设计岗位生成主图、详情页、配色、素材规范或活动海报方案时，成功结果会更稳定地进入当前 workspace 的交付物体系。
- 如果主图、详情页或素材规范使用默认商品/平台规则，结果会带 `data_readiness=calculated_from_defaults`，前端可以明确提示“基于默认规则”。
- 本阶段不会让用户误以为已经生成真实图片文件；当前沉淀的是可执行设计 brief 和素材规格。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent design_ -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_design_ -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`

## Post 8-40：Creative 高级内容交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 40 个补强项：`Post 8-40`。

本次目标：

- 把已 native 化的创意内容能力从“只返回 skill 输出”推进到“可被 SSE 自动沉淀为 workspace artifact”。
- 补齐 `creative_trend_catch` 和 `creative_title_ctr_scorer` 的工作区交付物映射。
- 修正 creative native skill 输出契约，确保已有创意交付物映射真正生效。

涉及范围：

- 产品内核：创意虚拟员工的趋势、脚本、文案、排期、直播话术、软文和标题评分结果能进入工作区交付物体系，便于后续复用、导出和前端展示。
- 后端 Agent：`backend/crates/agent/src/skills/creative.rs` 统一成功输出字段：`status=success`、`has_data=true`、`adapter_status=manual_input`、`data_readiness`、`summary`。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `creative_trend_report`、`creative_title_ctr_score`，并让已有 creative artifact 映射依赖的成功状态完整可用。
- API/SSE：更新 `API_DOCS/app.md` 与 `API_DOCS/sse.md`，说明 creative 内容 skill 的 artifact 类型和默认估算边界。
- 前端：本阶段不新增页面和假按钮；后续前端只需按已有 workspace artifact / SSE 事件展示真实交付物。

用户可感知变化：

- 用户在问答中触发创意岗位能力后，成功的短视频脚本、种草文案、IP定位、内容排期、直播脚本、产品软文、趋势方向和标题CTR评分，都会更稳定地出现在工作区交付物里。
- 如果创意 skill 使用默认商品、排期或直播参数，交付物内容会带 `data_readiness=calculated_from_defaults`，前端可以提示“基于默认估算”，不假装读取了真实平台数据。
- 如果缺少必填参数，例如 IP 定位缺品牌/类目、趋势捕捉缺类目、标题评分缺标题，后端仍返回 validation error，不创建假交付物。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent creative_ -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_creative_ -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`

## Post 8-39：Web 增长交付物真实沉淀

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

## Post 8-38：Data 进阶预测与留存交付物真实沉淀

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

## Post 8-37：Data 智能分析报告交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 37 个补强项：`Post 8-37`。

本次目标：

- 把旧产品数据岗位中已经 native 化的智能分析类能力接入 workspace artifact 体系。
- 让异常诊断、趋势预测、客户分层、竞品分析和 A/B 测试分析成为可查询、可预览、可导出的数据报告交付物。

涉及范围：

- 产品内核：数据岗位从看板/漏斗等基础分析继续推进到诊断、预测、分群、竞品和实验分析报告。
- 后端 agent：沿用现有 `data::execute` 统一出口，补齐 `status=success/deferred`、`has_data`、`adapter_status`、`data_readiness` 和 `summary`。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `data_anomaly_report`、`data_trend_forecast_report`、`data_customer_segmentation_report`、`data_competitor_report`、`data_ab_test_report`。
- API/SSE 文档：同步 data insight artifact 类型、默认数据披露和缺输入不创建假交付物的语义。

用户可感知变化：

- 用户让数据岗位做异常诊断、趋势预测、客户分层、竞品分析或 A/B 测试分析时，成功结果可以在当前 workspace 真实新增对应数据分析交付物。
- 这些交付物沿用现有 artifact 列表、详情、预览和导出链路。
- 如果缺少时序数据或实验数据，后端返回 `deferred` 或 validation error，不会创建看似真实的假数据报告。
- 如果客户分层或竞品分析使用确定性默认数据，结果会通过 `data_readiness=calculated_from_defaults` 明确告知前端和用户。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent data_anomaly_diagnosis_marks_success_for_user_series -- --nocapture`
- [x] `cargo test -p agent data_anomaly_diagnosis_defers_without_enough_series -- --nocapture`
- [x] `cargo test -p agent data_trend_forecast_marks_success_for_user_series -- --nocapture`
- [x] `cargo test -p agent data_customer_segmentation_discloses_default_inputs -- --nocapture`
- [x] `cargo test -p agent data_competitor_analysis_discloses_default_competitors -- --nocapture`
- [x] `cargo test -p agent data_ab_test_analyzer_marks_success_for_experiment -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_data_insight_skill_output -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`

## Post 8-36：Ops 高级运营分析交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 36 个补强项：`Post 8-36`。

本次目标：

- 把旧产品运营岗位中已经 native 化的高级分析类能力接入 workspace artifact 体系。
- 让智能定价、库存优化、广告疲劳检测和 ABC-XYZ 库存分类成为可查询、可预览、可导出的运营分析交付物。

涉及范围：

- 产品内核：运营岗位从核心计划交付物继续推进到高级分析报告交付物。
- 后端 agent：沿用现有 `ops::execute` 统一出口，补齐高级运营 skill 的 `data_readiness` 语义；缺 SKU/渠道等必要输入时保持 `deferred`。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `ops_smart_pricing_report`、`ops_inventory_optimization_report`、`ops_ad_fatigue_report`、`ops_abc_xyz_report`。
- API/SSE 文档：同步高级 ops artifact 类型、默认/演示数据披露和缺输入不创建假交付物的语义。

用户可感知变化：

- 用户让运营岗位做智能定价、库存优化、广告疲劳检测或 ABC-XYZ 库存分类时，成功结果可以在当前 workspace 真实新增对应运营分析交付物。
- 这些交付物沿用现有 artifact 列表、详情、预览和导出链路。
- 如果缺少 SKU、渠道、成本或售价等必要输入，后端返回 `deferred`，不会创建看似真实的假运营分析报告。
- 如果智能定价缺少竞品价格，或 ABC-XYZ 使用确定性演示 SKU，结果会通过 `data_readiness=calculated_from_defaults` 明确告知前端和用户。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent ops_smart_pricing_marks_success_for_price_inputs -- --nocapture`
- [x] `cargo test -p agent ops_smart_pricing_discloses_default_competitor_range -- --nocapture`
- [x] `cargo test -p agent ops_inventory_optimizer_defers_without_skus -- --nocapture`
- [x] `cargo test -p agent ops_ad_fatigue_detector_defers_without_channels -- --nocapture`
- [x] `cargo test -p agent ops_abc_xyz_classifier_discloses_demo_skus -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_ops_advanced_skill_output -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`

## Post 8-35：Ops 核心运营计划交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 35 个补强项：`Post 8-35`。

本次目标：

- 把旧产品运营岗位中已经 native 化的核心计划类能力接入 workspace artifact 体系。
- 让促销策划、定价策略、渠道策略、库存规划、上架文案、选品规划和运营执行计划成为可查询、可预览、可导出的运营交付物。

涉及范围：

- 产品内核：运营岗位从“当轮 skill 结果”推进到“可复用运营计划交付物”。
- 后端 agent：在现有 `ops::execute` 统一出口补齐 `status=success/deferred`、`has_data`、`adapter_status`、`data_readiness` 和 `summary`，不另建运营执行器。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `ops_promo_plan`、`ops_pricing_strategy`、`ops_channel_strategy`、`ops_inventory_plan`、`ops_listing_copy`、`ops_assortment_plan`、`ops_execution_plan`。
- API/SSE 文档：同步 ops artifact 类型和缺少必要输入不创建假交付物的语义。

用户可感知变化：

- 用户让运营岗位生成促销方案、定价策略、渠道组合、库存规划、上架文案、选品组合或 30 天运营执行计划时，成功结果可以在当前 workspace 真实新增对应运营交付物。
- 这些交付物沿用现有 artifact 列表、详情、预览和导出链路。
- 如果缺少商品名称、商品 ID 等必要用户输入，后端返回 `deferred`，不会创建一个看似可用的假运营交付物。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent ops_promo_planning_marks_success_for_user_input -- --nocapture`
- [x] `cargo test -p agent ops_pricing_strategy_discloses_default_competitor_prices -- --nocapture`
- [x] `cargo test -p agent ops_listing_copy_defers_without_product_context -- --nocapture`
- [x] `cargo test -p agent ops_execution_plan_marks_success_for_goal -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_ops_core_skill_output -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`

## Post 8-34：Service 售后与客服洞察交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 34 个补强项：`Post 8-34`。

本次目标：

- 把旧产品客服岗位中已经 native 化的退换货、NPS/CSAT、情绪分析、满意度驱动因素能力接入 workspace artifact 体系。
- 让售后处理和客服洞察不只停留在当轮问答，而是成为可查询、可预览、可导出的客服分析交付物。

涉及范围：

- 产品内核：客服岗位从“客服 SOP / playbook”继续推进到“售后处理 + 客户满意度洞察”。
- 后端 agent：`service_return_handler`、`service_nps_analyzer`、`service_sentiment_analyzer`、`service_nps_driver_analysis` 增加 `status=success`、`has_data=true`、`adapter_status=manual_input`、`data_readiness` 和 `summary`。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `service_return_plan`、`service_nps_report`、`service_sentiment_report`、`service_nps_driver_report`。
- API/SSE 文档：同步 service insight artifact 类型和默认样例数据必须明示的语义。

用户可感知变化：

- 用户让客服岗位处理退换货、分析 NPS/CSAT、识别客户情绪或诊断满意度驱动因素时，成功结果可以在当前 workspace 真实新增对应客服洞察交付物。
- 这些交付物沿用现有 artifact 列表、详情、预览和导出链路。
- NPS 或满意度驱动因素如果使用默认样例数据，会明确标记 `data_readiness=calculated_from_defaults`，前端不能展示成实时平台数据。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent service_return_handler_marks_success_for_return_case -- --nocapture`
- [x] `cargo test -p agent service_nps_analyzer_marks_success_for_score_distribution -- --nocapture`
- [x] `cargo test -p agent service_sentiment_analyzer_marks_success_for_customer_text -- --nocapture`
- [x] `cargo test -p agent service_nps_driver_analysis_marks_defaults_disclosed -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_service_insight_skill_output -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`

## Post 8-33：Service 客服支持交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 33 个补强项：`Post 8-33`。

本次目标：

- 把旧产品客服岗位中已经 native 化的核心支持能力接入 workspace artifact 体系。
- 让客服工单处理、FAQ 话术库、投诉升级流程和 DSR 改善方案成为可查询、可预览、可导出的客服支持交付物。

涉及范围：

- 产品内核：客服岗位从“当轮结构化回复”推进到“可沉淀的客服 SOP / playbook 交付物”。
- 后端 agent：`service_ticket_handler`、`service_faq_playbook`、`service_escalation_flow`、`service_dsr_improvement` 增加 `status=success`、`has_data=true`、`adapter_status=manual_input`、`data_readiness` 和 `summary`。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `service_ticket_playbook`、`service_faq_playbook`、`service_escalation_playbook`、`service_dsr_improvement_plan`。
- API/SSE 文档：同步 service artifact 类型和客服 DSR 默认样例需明示的语义。

用户可感知变化：

- 用户让客服岗位处理客诉、生成 FAQ、制定升级流程或输出 DSR 提升方案时，成功结果可以在当前 workspace 真实新增对应客服支持交付物。
- 这些交付物可以继续走现有工作区 artifact 列表、详情、预览和导出链路，不需要前端先做假按钮。
- DSR 如果没有用户提供真实评分，会明确标记 `data_readiness=calculated_from_defaults`，前端不能把它展示成实时店铺指标。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent service_ticket_handler_marks_success_for_user_issue -- --nocapture`
- [x] `cargo test -p agent service_faq_playbook_marks_success_for_policy_input -- --nocapture`
- [x] `cargo test -p agent service_escalation_flow_marks_success_for_complaint -- --nocapture`
- [x] `cargo test -p agent service_dsr_improvement_marks_success_for_scores -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_service_support_skill_output -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`

## Post 8-32：Accounting 财务报告交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 32 个补强项：`Post 8-32`。

本次目标：

- 把旧产品矩阵中已有的核心 accounting skill 成功输出接入 workspace artifact 体系。
- 让成本核算、利润分析、ROI 计算和预算编制成为可查询、可预览、可导出的财务报告交付物。

涉及范围：

- 产品内核：财务岗位从“单次计算输出”推进到“可沉淀的财务报告”。
- 后端 agent：`accounting_cost_calc`、`accounting_profit_analysis`、`accounting_roi_calc`、`accounting_budget_plan` 增加 `status=success`、`has_data=true`、`adapter_status=manual_input` 和 `summary`；缺少关键输入时返回 `deferred`。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `accounting_cost_report`、`accounting_profit_report`、`accounting_roi_report`、`accounting_budget_plan`。
- API/SSE 文档：同步 accounting artifact 类型和缺数据不造假报告的语义。

用户可感知变化：

- 用户让财务岗位做成本核算、利润分析、ROI 计算或预算编制时，成功结果可以在当前 workspace 真实新增对应财务交付物。
- 如果缺少必要财务输入，后端会明确返回 `deferred`，不会创建一个看似真实的财务报告。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent accounting_cost_calc_marks_success_for_user_input -- --nocapture`
- [x] `cargo test -p agent accounting_profit_analysis_marks_success_for_revenue -- --nocapture`
- [x] `cargo test -p agent accounting_profit_analysis_defers_without_revenue -- --nocapture`
- [x] `cargo test -p agent accounting_roi_calc_marks_success_for_investment_and_revenue -- --nocapture`
- [x] `cargo test -p agent accounting_budget_plan_marks_success_with_defaults_disclosed -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_accounting_skill_output -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`

## Post 8-31：Data 分析报告交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 31 个补强项：`Post 8-31`。

本次目标：

- 把更多 native data skill 的成功输出接入 workspace artifact 体系。
- 让漏斗分析、渠道 ROI、退款拆解不只停留在当轮问答，而是成为可查询、可预览、可导出的数据报告交付物。

涉及范围：

- 产品内核：数据岗位从“单次分析输出”继续推进到“可沉淀的数据报告”。
- 后端 agent：`data_funnel_analysis`、`data_channel_roi`、`data_refund_decomposition` 在有用户数据时明确输出 `status=success`、`has_data=true`、`adapter_status=manual_input` 和 `summary`；缺少数据时继续 `deferred`。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `data_funnel_report`、`data_channel_roi_report`、`data_refund_report`。
- API/SSE 文档：同步数据报告 artifact 类型和缺数据不造假报告的语义。

用户可感知变化：

- 用户提供漏斗数据、渠道数据或退款项数据后，数据岗位成功分析结果可以在当前 workspace 真实新增对应报告交付物。
- 如果用户没有提供必要数据，后端会明确返回 `deferred`，不会创建一个看似真实的数据报告。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent data_funnel_analysis_marks_success_for_user_data -- --nocapture`
- [x] `cargo test -p agent data_funnel_analysis_defers_when_metrics_missing -- --nocapture`
- [x] `cargo test -p agent data_channel_roi_marks_success_for_user_channels -- --nocapture`
- [x] `cargo test -p agent data_refund_decomposition_marks_success_for_user_items -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_data_analysis_skill_output -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`

## Post 8-30：Data 数据看板交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 30 个补强项：`Post 8-30`。

本次目标：

- 把 `data_dashboard` 的成功 skill 输出接入新产品已有 workspace artifact 体系。
- 让数据看板不只停留在当轮问答过程里，而是成为可查询、可预览、可导出的工作区交付物。

涉及范围：

- 产品内核：数据岗位输出从一次性 skill 结果推进到可复用交付物。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增支持 `data_dashboard`，生成 `artifactType=data_dashboard`、`contentCategory=data` 的 markdown 交付物。
- API/SSE 文档：同步 `data_dashboard` 交付物类型、metadata 和 `workspace_artifact_created` 语义。

用户可感知变化：

- 用户要求“店铺大盘”“数据看板”“经营概览”“核心经营指标”并成功得到数据岗位输出后，当前 workspace 可以真实新增一个 `data_dashboard` 交付物。
- 该交付物保留 `has_data` 和 `adapter_status`，如果缺少真实平台指标，前端仍应展示为“需要接入/补充真实经营数据”，不能伪装成实时平台看板。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service content_artifact_builds_from_data_dashboard_skill_output -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_design_skill_output -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_creative_skill_output -- --nocapture`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p service -p web-server`

## Post 8-29：Data 数据看板 native 化

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 29 个补强项：`Post 8-29`。

本次目标：

- 把 `data_dashboard` 从 pending/deferred 占位升级为新产品 Agent 框架内的 Rust native 数据看板 skill。
- 让数据岗位能真实输出经营指标看板结构，而不是只告诉用户“待接入”。

涉及范围：

- 产品内核：数据岗位补齐“店铺大盘 / 数据看板 / 经营概览 / 核心指标”能力。
- 后端 agent：`data_dashboard` 改为 deterministic native skill，输出核心指标、增长/效率/风险分区、健康度、预警和下一步动作。
- 后端 agent worker：自动补偿规则新增数据看板自然问法触达。

用户可感知变化：

- 用户问“店铺大盘”“数据看板”“经营概览”“核心经营指标”时，数据岗位可以返回结构化看板结果。
- 如果用户提供 GMV、订单、访客、广告花费、退款、毛利等指标，后端按这些真实入参计算。
- 如果没有真实指标，也会明确标记 `store_metrics_required` 和 `has_data=false`，前端不能把样例基线当成平台实时数据。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent data_dashboard_builds_from_user_metrics -- --nocapture`
- [x] `cargo test -p agent data_dashboard_marks_sample_when_metrics_missing -- --nocapture`
- [x] `cargo test -p agent autosuggests_more_specific_data_skills_by_intent -- --nocapture`
- [x] `cargo test -p agent executor_runs_representative_native_categories -- --nocapture`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`

## Post 8-28：Design 设计交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 28 个补强项：`Post 8-28`。

本次目标：

- 把设计类 native skill 从“只在问答过程里返回设计方案”推进到“自动沉淀为真实 workspace artifact”。
- 继续以后端真实能力为准，不新增前端假入口。

涉及范围：

- 产品内核：设计岗位输出从一次性过程结果推进到可查询、可复用、可导出的工作区交付物。
- 后端 service：复用 `WorkspaceApi::materialize_agent_content_artifact`，新增支持 `design_main_image`、`design_detail_page`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster`。
- API/SSE 文档：同步 creative / design 内容交付物的 artifact 类型、metadata 和 `workspace_artifact_created` 语义。

用户可感知变化：

- 用户要求“主图设计方案”“详情页结构”“配色方案”“素材规格”“活动海报 brief”时，成功的设计 skill 输出会在当前 workspace 真实新增对应交付物。
- 这些设计交付物进入已有 artifact 查询、预览和导出链路；当前仍是结构化 markdown brief，不伪造生成图片文件。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service content_artifact_builds_from_design_skill_output -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_creative_skill_output -- --nocapture`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy -p service -p web-server`

## Post 8-27：Creative 内容交付物真实沉淀

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 27 个补强项：`Post 8-27`。

本次目标：

- 把高价值 creative skill 从“只在问答里返回内容”推进到“自动沉淀为真实 workspace artifact”。
- 继续以后端真实能力为准，不新增前端假入口。

涉及范围：

- 产品内核：创意内容能力从一次性回答推进到可查询、可复用、可导出的工作区交付物。
- 后端 service：`WorkspaceApi` 新增 `materialize_agent_content_artifact`，支持 `creative_video_script`、`creative_seeding_copy`、`creative_ip_branding`、`creative_content_calendar`、`creative_live_script`、`creative_product_article`。
- 后端 web-server：SSE 在记录 `skill_result` 后自动尝试沉淀 creative 内容 artifact，并继续复用 `workspace_artifact_created` 回传。

用户可感知变化：

- 用户问“写短视频脚本”“生成种草文案”“做品牌 IP 定位”“排内容日历”“写直播话术”“写产品软文”时，除了当轮 skill 结果，还会在当前 workspace 真实新增对应交付物。
- 这些内容产物开始进入新产品已有的 artifact 查询、预览和后续导出链路，不再只停留在聊天窗口。

当前验证：

- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo test -p service content_artifact_builds_from_creative_skill_output -- --nocapture`
- [x] `cargo test -p service content_artifact_ignores_non_supported_skill -- --nocapture`

## Post 8-26：Coordination 创建营销活动草稿真实落地

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 26 个补强项：`Post 8-26`。

本次目标：

- 把 `coordination_create_campaign` 从 `pending_approval` 骨架升级为真实 Rust native skill。
- 让 Agent 在当前工作区里不只是“建议创建营销活动”，而是能通过现有 service/SSE 持久化链路真实创建 `campaign draft`。
- 继续遵循新产品后端主导，不直接写平台，不绕开审批与治理。

涉及范围：

- 产品内核：协同层从“能提出活动建议”推进到“能把活动方案沉淀成真实 campaign draft 和工作区交付物”。
- 后端 agent：`coordination_create_campaign` 改为 native deterministic skill，并补商品/工作区/渠道/预算/策略等结构化草稿输出。
- 后端 service/web-server：SSE 在记录 `skill_result` 后，调用 workspace/campaign 持久化链路，把草稿真实落成 `campaigns` 和 `workspace_artifacts`，并回传 `campaign_draft_created`。

用户可感知变化：

- 用户提出“创建活动草稿”“整理促销方案”“做一个投放复盘活动草稿”时，更容易命中真实后端活动草稿能力。
- 命中后，当前工作区会真实新增一个 `campaign draft`，不再只是返回一段待审批骨架说明。
- SSE 会出现真实 `campaign_draft_created` 事件，并同时生成一个 `campaign_draft` artifact，后续前端可直接读取、预览和继续编辑。

当前验证：

- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`

## Post 8-25：工具审批通过后的平台执行闭环

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 25 个补强项：`Post 8-25`。

本次目标：

- 把“高风险平台 skill 进入审批后就停住”的链路补成真实执行闭环。
- 让审批通过后自动创建 `execution_queue`，并把真实执行结果或失败原因写回审批记录与审计日志。
- 在远端平台 adapter 尚未真正接入前，保持真实失败，不伪造成功。

涉及范围：

- 产品内核：平台执行从“只有审批记录”推进到“审批通过后进入真实执行尝试与审计闭环”。
- 后端 service：`approve_tool_approval` 增加执行队列创建、连接校验、参数校验、失败结果持久化和审计写入。
- 后端 web-server：SSE 自动创建审批记录时补存 `normalizedInputs` 和原始 `output`，供审批后的执行闭环读取。

用户可感知变化：

- 用户审批通过平台改价、改库存、同步商品等动作后，后端不会再只停留在 `approved`。
- 当前如果平台连接缺失、连接被禁用、参数不完整或 adapter 尚未实现，用户会看到真实失败状态与原因。
- 审批、执行队列、审计记录三者开始真正打通，后续接真实平台 adapter 时不需要再重做闭环骨架。

当前验证：

- [x] `cargo fmt`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo clippy`
- [x] `cargo test -p service approval_execution_params_reads_normalized_inputs_from_skill_output -- --nocapture`
- [x] `cargo test -p service plan_platform_execution_reports_missing_connection -- --nocapture`
- [x] `cargo test -p service plan_platform_execution_reports_adapter_not_implemented_when_connected -- --nocapture`

## Post 8-24：Coordination 保存工作记忆真实落地

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 24 个补强项：`Post 8-24`。

本次目标：

- 把 `coordination_save_memory` 从 `pending_approval` 骨架升级为真实 Rust native skill。
- 让 Agent 在当前工作区里不只是“建议记住某个结论”，而是能通过现有 service/SSE 持久化链路真实沉淀一条 workspace memory。
- 保持新产品分层，不让 `agent` 直接跨层写数据库。

涉及范围：

- 产品内核：协同层从“能提出记忆建议”推进到“能把事实/决策/洞察/风险落成真实工作记忆”。
- 后端 agent：`coordination_save_memory` 改为 native deterministic skill，并补“记住/记下来/保存这个结论”等自然问法触达。
- 后端 service/web-server：SSE 在记录 `skill_result` 后，调用工作区 service 将记忆草案幂等落成真实 `agent_learning_records`，并回传 `agent_learning_record_created`。

用户可感知变化：

- 用户提出“记住这个结论”“把这个风险记下来”“保存这个决策”时，更容易命中真实后端保存记忆能力。
- 命中后，当前工作区会真实新增或更新一条工作记忆，不再只是返回一段待审批骨架说明。
- SSE 会出现真实 `agent_learning_record_created` 事件，后续同工作区问答也可优先加载这些记忆。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent autosuggest`
- [x] `cargo test -p agent coordination_save_memory`
- [x] `cargo test -p service workspace_memory`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`

## Post 8-23：Coordination 创建任务真实落地

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 23 个补强项：`Post 8-23`。

本次目标：

- 把 `coordination_create_task` 从 `pending_approval` 骨架升级为真实 Rust native skill。
- 让 Agent 在当前工作区里不只是“建议建任务”，而是能通过现有 service/SSE 持久化链路真实创建 `workspace_task`。
- 保持新产品分层，不让 `agent` 直接跨层写数据库。

涉及范围：

- 产品内核：协同层从“能提出任务建议”推进到“能把任务落成真实工作区待办”。
- 后端 agent：`coordination_create_task` 改为 native deterministic skill，并补“创建任务/待办/安排任务”等自然问法触达。
- 后端 service/web-server：SSE 在记录 `skill_result` 后，调用工作区 service 将任务草案幂等落成真实 `workspace_task`，并回传 `workspace_task_created`。

用户可感知变化：

- 用户提出“帮我建个任务”“安排一个待办”“分派给某岗位处理”时，更容易命中真实后端建任务能力。
- 命中后，当前工作区会真实新增任务，不再只是返回一段待审批骨架说明。
- SSE 会出现真实 `workspace_task_created` 事件，刷新后工作区任务列表仍可见。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent autosuggest`
- [x] `cargo test -p service workspace_task_artifact`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`

## Post 8-22：Coordination 状态查询真实化

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 22 个补强项：`Post 8-22`。

本次目标：

- 把 `coordination_status_query` 从 deferred 骨架升级为真实 Rust native skill。
- 让当前新产品的多智能体运行链路具备“可查询会话/任务/工作区状态”的后端原生能力。
- 不引入新 API、不做数据库变更、不做高风险写操作。

涉及范围：

- 产品内核：协同层状态从“只有事件流”推进到“有真实状态查询 skill”。
- 后端 agent：扩展 `SkillContext` 透传 session/task/workspace/request metadata；`coordination_status_query` 改为 native 实现。
- 问答主链路：chat / engineering 状态类自然问法可自动命中 `coordination_status_query`。

用户可感知变化：

- 用户问“当前状态”“任务进度”“workspace 状态”“系统健康”“agent 运行到哪了”时，更容易命中真实后端协调状态查询能力。
- 返回内容会真实反映当前 session 范围内可见的 worker、task、graph、workspace、run、context readiness 和 blockers。
- 没有可见信息时会明确说明缺口，不会伪造全局状态。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent autosuggest`
- [x] `cargo test -p agent executor_runs_representative_native_categories`
- [x] `cargo check -p agent`
- [x] `cargo check`
- [x] `cargo clippy`

## Post 8-21：Native skill 可达性补强

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 21 个补强项：`Post 8-21`。

本次目标：

- 不新增假前端入口，继续以后端为主。
- 把已经迁到 Rust 后端、但真实自然问法命中率还不够高的旧产品能力，补进主问答链路。
- 通过 autosuggest 补强，让用户更容易触发正确的 native skill 或正确的 deferred engineering skill。

涉及范围：

- 产品内核：问答主链路从“已有 skill”继续推进到“已有 skill 更容易被真实用户问法命中”。
- 后端 agent：补强 `autosuggest_ops/data/accounting/service/engineering` 的意图覆盖。
- 文档：重写统一后的迁移主计划，消除已失效的 `backend/crates/agent` 旧硬约束。

计划中的命中补强：

- `ops_channel_strategy`
- `ops_listing_copy`
- `ops_assortment_planning`
- `data_competitor_analysis`
- `data_channel_roi`
- `data_refund_decomposition`
- `accounting_compliance_check`
- `service_nps_driver_analysis`
- `engineering_arch_review`
- `engineering_bug_analysis`
- `engineering_perf_optimize`
- `engineering_tech_selection`

用户可感知变化：

- 用户问“渠道分配”“上架文案”“货盘组合”“竞品分析”“退款帕累托”“税务合规”“NPS驱动因素”“架构评审”“性能优化”“技术选型”等问题时，更容易看到后端命中对应能力，而不是落回泛化执行计划。
- 这些变化都来自后端真实 skill 选择和 SSE 过程事件，不是前端伪造。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent autosuggest`
- [x] `cargo check -p agent`
- [x] `cargo check`
- [x] `cargo clippy`

## 9/12：登录态恢复与旧产品内置能力包治理同步

状态：done

本次目标：

- 修复登录后刷新或直接访问受保护页面误跳 `/login` 的问题。
- 把旧产品 `server/packages` 中的内置 domain、role、skill manifest 同步到新产品能力包治理目录。
- 为后续迁移旧产品 skill runtime、外部搜索、平台执行、教学/导出/素材能力建立可治理的 package/capability 台账。

涉及范围：

- 产品内核：能力包治理目录开始承接旧产品 skill 生态，不再只能手动录入。
- 后端：新增 `POST /api/app/packages/sync-legacy-builtins`；复用 `package_registry` 和 `package_state_history`；批量同步使用事务。
- API：更新 `API_DOCS/app.md`，记录同步接口、请求、响应和运行边界。
- 前端：修复 auth restore 与 `RequireAuth` 的时序；能力包页增加同步旧产品能力包按钮。

用户可感知变化：

- 登录后刷新 `/packages`、`/products` 等页面不会再误跳登录。
- 能力包治理页点击“同步旧产品能力包”后，会出现旧产品内置能力包目录，例如 `Builtin Ops Skill Pack`。
- 同步后的能力包可以继续被搜索、筛选、校验、启停、回滚和保存运行锁。

当前验证：

- [x] `cargo fmt`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] 测试账号真实 HTTP smoke：首次同步创建 52 条记录，其中 20 条来自旧 manifest，32 条为派生 capability；`builtin.ops` 和 `role.ops-strategist` 校验通过。
- [x] 浏览器自动化：登录后直达/刷新 `/packages`、`/products` 均不跳登录；能力包页可见同步按钮和 `Builtin Ops Skill Pack`。
- [x] LLM 分层验证：云端 `/v1/models` 可访问，`gpt-oss:20b` 直接 chat/completions 返回 `LLM_OK`。
- [x] Agent 验证：`ollama_task_graph_smoke` 和真实 `/api/sse/agent/session/stream` 协同任务均跑通，真实 SSE 包含 `task_graph_planned`、`output`、`finished`。

## 10/12：Agent 代表 Skill 自动补偿触发

状态：done

本次目标：

- 解决真实问答中过度依赖 LLM 主动输出 `skill_requests` JSON 的问题。
- 当模型没有显式请求 skill 时，由后端按岗位、任务文本、上下文和上游结果自动补偿一个代表 skill。
- 保持补偿请求仍走角色白名单、`SkillExecutorSet`、SSE 事件和 worker report 合并链路。

涉及范围：

- 产品内核：skill 触发从“只依赖模型显式 JSON”升级为“模型显式请求优先 + 后端岗位自动补偿”。
- 后端：修改 Agent worker runtime；新增 ops/data/accounting/service/creative/web/design/engineering 岗位补偿规则；收紧 chat bypass，只让短问候绕过 commander；新增 stream 级测试。
- API：更新 `API_DOCS/sse.md`，说明 `skill_requested` 可能来自模型显式请求或后端自动补偿。
- 前端：本阶段不改前端；沿用既有 `skill_requested` / `skill_result` 展示链路。

用户可感知变化：

- 用户在问答页提出数据漏斗、运营计划、ROI、退换货、标题优化、素材规格等问题时，即使模型没有主动写工具调用 JSON，过程面板也更稳定出现真实能力调用。
- 例如“请分析转化漏斗：曝光10000 点击500 加购120 下单60 付款54”默认会进入 data/ops 岗位任务图，并触发 `data_funnel_analysis`。
- “hi/你好”仍保持快速聊天，不启动多岗位流程。
- 外部搜索、内容生成、平台执行等未接 adapter 的能力仍会明确返回 `deferred` 或 `pending_approval`，不会伪造结果。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent autosuggest`
- [x] `cargo test -p agent stream_emits_autosuggested_skill_events_when_worker_omits_tool_json`
- [x] `cargo test -p agent --test default_roles`
- [x] 真实云端 Ollama smoke：默认漏斗问题已输出 `task_graph_planned`、`skill_requested`、`skill_result`、`output`

## 1/8：问答策略 metadata 与后端路由事件

状态：done

本次目标：

- 把回答策略从前端 prompt 拼接迁移为后端结构化 metadata。
- 后端 SSE 输出 `route_decided`，让前端展示真实策略和路由结果。
- `collab` 策略优先触发多角色任务图。

涉及范围：

- 产品内核：agent request/context 新增 metadata 通道，策略成为 runtime 输入契约。
- 后端：`AgentRequest`、`MessageContext`、`SessionEvent`、session runtime、commander routing、SSE DTO。
- API：更新 `API_DOCS/sse.md`。
- 前端：stream request 发送 `metadata.strategy`，时间线消费 `route_decided`。

用户可感知变化：

- 切换回答策略后，不再只是把策略文字塞进问题里，而是由后端识别并进入真实路由。
- 问答过程面板会出现“后端已采用某策略、路由到某主角色”的真实事件。
- 选择协同策略时，更稳定进入多虚拟员工任务图。

验证：

- [x] `cargo check -p agent`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] agent strategy routing test
- [x] frontend rules/lint/build

## 2/8：Skill 执行与 SSE 可视化

状态：done

本次目标：

- 后端 worker 执行 skill 前后输出真实 SSE 事件。
- 前端过程面板消费 `skill_requested` / `skill_result`，展示工具调用状态。
- 保持 skill 执行结果进入 worker report 的既有链路，不用前端伪造工具状态。

涉及范围：

- 产品内核：skill 调用从“只写入 worker 内存”提升为 agent runtime 可观察事件。
- 后端：`SessionEvent`、worker runtime、session runtime worker 构造、agent stream 测试。
- API：更新 `API_DOCS/sse.md`。
- 前端：`AgentRuntimeEvent` 类型、字段兼容、时间线和运行详情卡。

用户可感知变化：

- 后端真正调用 skill 时，右侧过程面板会看到“能力调用”和返回状态。
- success、deferred、pending_approval、failed 等状态不会被前端伪造，全部来自 SSE。
- 高风险或平台写操作会展示需要审批，不会让用户误以为已经自动执行。

当前验证：

- [x] `cargo fmt`
- [x] `cargo check -p agent`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `cargo test -p agent --test default_roles stream_emits_skill_execution_events`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web build`

## 3/8：产品知识与上下文绑定

状态：done

本次目标：

- 让问答请求可以绑定真实商品上下文。
- 后端按当前用户权限读取商品和商品知识，并把摘要注入 agent runtime。
- 前端从商品 API 读取 active 商品，用户可选择商品 chip 后随 SSE 请求传入。

涉及范围：

- 产品内核：agent runtime 新增 `context_loaded` 事件，commander/worker cognition 都能收到商品上下文 facts。
- 后端：新增 `service::api::agent_context` 聚合产品与知识；SSE DTO 支持 `metadata.context.productIds/contextRefs`；web-server 在启动 agent 前装配上下文 metadata。
- API：更新 `API_DOCS/sse.md`。
- 前端：agent-chat composer 显示后端商品上下文选择；请求发送真实 `metadata.context`；过程面板展示后端 `context_loaded` 事件和上下文卡片。

用户可感知变化：

- 登录后，问答输入框上方会显示当前后端 active 商品，可点选作为本轮问答上下文。
- 发送问题后，右侧过程面板会出现“已从后端读取商品/知识”的真实事件。
- 绑定商品后，模型拿到的是后端读取的商品名称、平台、类目、品牌、价格、描述和商品知识摘要；未绑定时不会假装知道商品详情。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service agent_context`
- [x] `cargo test -p web-server sse`
- [x] `cargo test -p agent --test default_roles context`
- [x] `cargo check -p service`
- [x] `cargo check -p agent`
- [x] `cargo check -p web-server`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web build`

## 4/8：工作区沉淀与交付物

状态：done

本次目标：

- 让 Agent SSE 问答先绑定真实工作区运行记录。
- 把 task graph 节点沉淀为 `workspace_tasks` 和 `workspace_run_tasks`。
- 把 worker report、evaluation 和最终输出写回工作区运行数据。
- 把最终回答沉淀为可查询、可更新、可归档的 `workspace_artifacts.final_report`。

涉及范围：

- 产品内核：Agent session 从临时流式过程升级为可追踪的 workspace run，并形成独立交付物记录。
- 后端：新增 `workspace_artifacts` 表和 repo/service/web API；`WorkspaceApi` 新增 agent run/task/report/evaluation/artifact 持久化方法；SSE handler 把 agent 事件转为 service 层写入。
- API：更新 `API_DOCS/sse.md` 和 `API_DOCS/app.md`。
- 前端：同步 workspace SSE event 类型，在右侧运行态展示真实 workspace/run/task/artifact 沉淀状态。

用户可感知变化：

- 发送问答后，后端会自动创建或复用一个 `agent_chat` 工作区。
- 复杂问答产生任务图时，每个虚拟员工节点会落成可查询的工作区任务快照。
- 问答完成后，最终回答会同时保存到 `workspace_run.report.output` 和 `workspace_artifacts`，右侧面板会提示交付物已经生成。

当前验证：

- [x] `cargo fmt`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo test -p web-server sse`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`

## 8/8：完整商业化体验

状态：done

本次目标：

- 让新产品入口更像“电商多智能体/虚拟员工”，而不是普通聊天。
- 后端提供商业场景 playbook，前端按后端契约展示，不再硬编码样例 prompt。
- 点击场景后进入已有问答、策略、RAG、工作区、质量和审批链路。

涉及范围：

- 产品内核：问答入口从前端样例升级为后端商业场景目录，场景自带推荐策略、所需上下文、预期产出和虚拟员工角色。
- 后端：新增 `CommercePlaybookApi`、`GET /api/app/agent/playbooks`、6 个内置电商场景和 service 单测。
- API：更新 `API_DOCS/app.md`，记录 playbook 字段、鉴权和当前内置场景。
- 前端：新增 playbook API/domain/hook，问答 composer 改为消费后端 playbook，点击场景自动填充任务并切换推荐策略。

用户可感知变化：

- 问答页输入区上方出现 6 个后端场景入口：新品冷启动、投放复盘、客诉危机、选品定价、素材优化、日报复盘。
- 用户点击任一场景后，输入框自动填入完整电商任务，不需要自己从零描述。
- 场景会自动选择合适回答策略，例如复杂协作类走 `collab`，解释判断类走 `learning`，危机/日报类走 `execution`。
- 商品上下文选择、RAG、工作区沉淀、质量检查、审批告警等前面已迁移链路继续保留。

当前验证：

- [x] `cargo fmt`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `cargo test -p service commerce_playbook`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `git diff --check`

## 7/8：搜索/RAG/外部数据

状态：done

本次目标：
- 用新产品后端框架实现真实本地 RAG，而不是伪造外部搜索结果。
- 用户发送问题后，后端自动按当前用户检索 active 商品和 active 商品知识。
- 检索结果既注入 agent metadata，也通过 SSE 事件展示给前端。
- 外部搜索适配器未接入时，前端展示明确降级原因。

涉及范围：
- 产品内核：问答链路从“用户手动选择商品上下文”扩展为“问题自动触发本地知识检索”；RAG 摘要追加到 `context.summary`，agent runtime 无需改动即可使用。
- 后端：`product_knowledge` repo 增加当前用户全局知识检索；`AgentContextApi` 增加 `build_retrieval_context`；SSE 增加 `rag_context_loaded`，并写入 `rag.*` metadata。
- API：更新 `API_DOCS/sse.md` 和 `API_DOCS/app.md`，记录本地 RAG 来源、降级状态和外部搜索边界。
- 前端：`AgentRuntimeEvent` 增加 `rag_context_loaded`；时间线和右侧过程卡片展示知识检索来源、检索词、知识数量和降级原因。

用户可感知变化：

- 不手动选择商品时，用户直接提问也会触发后端本地知识检索。
- 命中商品知识时，右侧过程区会显示“知识检索”，并展示来源数量和部分来源标题。
- 没有命中本地知识时，右侧过程区会明确显示本地知识未命中且外部搜索适配器未接入，不会假装查到了外部资料。
- 模型回答会更容易围绕已维护的商品知识展开，而不是只依赖通用模型常识。

当前验证：
- [x] `cargo fmt`
- [x] `cargo check -p service`
- [x] `cargo test -p service agent_context`
- [x] `cargo check -p web-server`
- [x] `cargo test -p web-server sse`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`

## 5/8：质量、信任、学习闭环

状态：done

本次目标：

- Agent SSE 回答完成后，由后端自动检查回答质量。
- 质量结果更新 agent 信任分，并沉淀为学习记录。
- 下一次问答前，后端读取学习记录和信任摘要并注入 agent runtime metadata。
- 前端只展示后端真实 `quality_checked` 事件，不伪造质量状态。

涉及范围：

- 产品内核：回答质量从一次性输出扩展为“检查 -> 信任分 -> 学习记录 -> 下次上下文”的闭环。
- 后端：新增 `agent_quality_checks`、`agent_trust_scores`、`agent_learning_records`；新增 repo/service/web API；SSE 自动质量检查和 `quality_checked` 事件。
- API：更新 `API_DOCS/app.md` 与 `API_DOCS/sse.md`。
- 前端：`AgentRuntimeEvent`、SSE 字段归一化、时间线、右侧工作区卡片、中英文文案。

用户可感知变化：

- 问答完成后，右侧过程区会看到“质量检查完成”事件。
- 工作区卡片会显示质量分、信任分和本轮学习记录数量。
- 如果质量不足，卡片会明确显示“需要改进”，并由后端沉淀改进经验。
- 用户反馈后端接口已具备，后续可以在前端接评分或反馈按钮。

当前验证：

- [x] `cargo check -p migration`
- [x] `cargo xtask migrate up`
- [x] `sea-orm-cli generate entity -o crates/repo/src/entity --with-serde both --date-time-crate time`
- [x] `cargo check -p repo`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `cargo test -p service`
- [x] `cargo test -p service quality_checker`
- [x] `cargo test -p service trust_ema`
- [x] `cargo test -p web-server sse`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] 前端 dev server HTTP smoke：`http://localhost:5173` 返回 200

## 6/8：审批、告警、平台写操作

状态：done

本次目标：

- 把 Agent 高风险平台写 skill 的 `pending_approval` 状态接入真实审批队列。
- 后端自动创建或复用审批、审计、告警记录。
- 前端只展示后端真实 `approval_required` 事件，不伪造审批状态。

涉及范围：

- 产品内核：高风险 skill result 从过程事件升级为治理入口，平台写操作默认阻断到审批队列。
- 后端：SSE handler 调用 `ApprovalAuditApi` 和 `BriefingOpsApi`，创建审批、审计和告警；新增 helper 和单元测试覆盖触发条件、风险等级映射、去重 key 长度。
- API：更新 `API_DOCS/sse.md` 与 `API_DOCS/app.md`。
- 前端：`AgentRuntimeEvent` 新增 `approval_required`；时间线和右侧工作区卡片展示审批 ID、风险等级、原因和告警消息。

用户可感知变化：

- 平台写操作不会显示成已经执行，而是显示“需要审批”。
- 问答页右侧会看到审批编号、能力名、审批状态和风险等级。
- 如果告警创建成功，右侧卡片会同步显示告警消息。

当前验证：

- [x] `cargo fmt`
- [x] `cargo check -p web-server`
- [x] `cargo test -p web-server sse`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`

## Post 8-1：工作区交付物刷新可见

状态：done

本次目标：

- 把 Phase 4 已经沉淀到后端的 workspace artifact 接入完整工作区页面。
- 工作区页面刷新后仍能查看 Agent 交付物列表和详情。
- 前端只消费真实后端 artifact API，不展示假交付物。

涉及范围：

- 产品内核：问答交付物从“过程事件可见”补强为“业务工作区可沉淀、可刷新、可复用”。
- 后端：未新增接口，复用已有 workspace artifact 查询和详情接口。
- 前端：新增 artifact API client、查询状态、工作区“交付物”tab、交付物列表、详情预览和指标卡。
- 文档：更新 Phase 4 验收记录、前端体验检查清单，并新增本次补强归档。

用户可感知变化：

- 用户进入“执行工作区”后，选中工作区可以打开“交付物”tab。
- 问答生成的 final report 等交付物，刷新页面后仍能在工作区里看到。
- 点击交付物后，右侧能看到标题、类型、来源、校验状态、格式、存储、文件信息、运行 ID 和正文/预览。

当前验证：

- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`
- [x] 测试账号 `test123456` 登录后真实 HTTP smoke：创建临时工作区、创建 `final_report` artifact、查询列表、查询详情均通过；临时工作区已归档清理。

运行注意：

- smoke 前本地 `19878` 上运行的是旧后端进程，对 artifact 路由返回 404。
- 重启本地后端并加载当前代码后，artifact 路由正常。

## Post 8-2：问答运行反馈前端接入

状态：done

本次目标：

- 把 Phase 5 已经完成的用户反馈后端接口接入问答工作台。
- 用户可以对真实 workspace run 评分并提交文字反馈。
- 前端只在后端返回真实运行记录后开放提交，不做假状态。

涉及范围：

- 产品内核：质量闭环补强为“自动质量检查 + 用户反馈学习记录”的双输入闭环。
- 后端：未新增接口，复用 `POST /api/app/workspaces/{workspaceId}/runs/{runId}/feedback`。
- 前端：新增反馈 API client、运行目标推导、反馈 hook、右侧工作台反馈面板和中英文文案。
- 文档：新增 Post 8-2 归档，更新 Phase 5 验收和前端体验清单。

用户可感知变化：

- 问答形成后端运行记录后，右侧虚拟员工工作区出现“本轮反馈”。
- 用户可以点 1 到 5 分星级，并填写可选反馈说明。
- 提交成功后，页面显示“已提交”，反馈写入后端学习记录。
- 未登录、生成中、无运行记录或已提交时，按钮会明确禁用并说明原因。

当前验证：

- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`
- [x] 测试账号真实 HTTP smoke：创建临时 workspace/run，提交反馈返回 `feedback_positive` learning record，临时 workspace 已归档。

测试注意：

- 冒烟测试确认 workspace run 状态应使用 `succeeded`，不能使用 `completed`。
- 后端反馈响应中的学习记录嵌套字段可能是 snake_case，前端 API client 已归一化。

## 12/12：创意内容类 Skill Native 迁移

状态：done

本次目标：

- 把旧产品创意岗位的短视频脚本、种草文案、IP 定位、内容排期、趋势捕捉、直播脚本、产品软文迁移为新产品 Agent runtime 的 Rust native skill。
- 解决这些 skill 只返回 `deferred` 骨架、用户看不到可用内容产物的问题。
- 扩展 creative 岗位自动补偿规则，让模型没显式输出 `skill_requests` 时，后端也能按创意意图命中对应 skill。

涉及范围：

- 产品内核：创意虚拟员工从“标题 CTR 评分为主”升级为“可直接生成内容方案、脚本、话术、排期和软文”的岗位能力。
- 后端：`backend/crates/agent/src/skills/creative.rs`、`backend/crates/agent/src/agent/worker/runner.rs`、`backend/crates/agent/src/skills/runtime.rs`。
- API：不新增接口，不修改 SSE 请求结构；沿用 `skill_requested` / `skill_result`。
- 前端：本阶段不改前端，继续展示后端真实 SSE 事件和最终回答。
- 文档：新增 Phase 12 后端设计和迁移归档，更新 skill 迁移矩阵。

用户可感知变化：

- 用户在问答里说“写抖音短视频脚本”“生成小红书种草文案”“做品牌 IP 定位”“排一周内容日历”“写直播带货话术”“写产品软文”“分析内容趋势”时，后端会更稳定触发对应 creative skill。
- 过程区能看到真实 skill 调用结果，结果状态是 success/native，而不是“内容引擎未接入”的 deferred。
- 当前结果是结构化文本产物：脚本结构、文案标题、正文草稿、直播阶段话术、内容排期和趋势方向；暂不生成图片、视频或 docx 文件。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent executor_runs_native_creative_content_skills`
- [x] `cargo test -p agent autosuggests_creative_content_skill_by_intent`
- [x] `cargo test -p agent`
- [x] `cargo check -p agent`

后续缺口：

- 外部搜索实时趋势 adapter 未接入。
- creative skill 尚未按真实 `product_id` 自动读取商品详情。
- 素材文件、视频、图片、导出报告仍是后续阶段。

## Post 8-3：问答短答链路与前端 chat_bypass 对齐

状态：done

本次目标：

- 修复新产品核心问答中“短答问题被默认 Agent 集群带偏”的问题。
- 让前端尊重后端 `route_decided.route_mode=chat_bypass`，短答场景不再展示多角色业务执行流程。
- 用 Edge 同时抽样新产品和旧产品，确认用户真实操作差异。

涉及范围：

- 产品内核：直接问答/短答成为一等路径；明确短答不启动任务图，不伪装成多智能体业务执行。
- 后端：chat 角色允许短摘要通过质量门禁；fast worker structured output 优先取 `role_output.summary`；显式短答在默认 `collab/manual` 下可走 `chat_bypass`；质量检查接受明确短答；含“转化、漏斗、曝光、点击、加购、下单、付款”等数字业务题仍走业务任务图。
- 前端：`apps/web` 问答 view-model 识别 `chat_bypass`，隐藏右侧虚拟员工工作区和内联 6 阶段流程；补充 `thread-inline-view-model` 单测和 web 测试 setup。
- 旧产品对照：旧产品真实存在模型选择、回答风格、自动/单岗/多岗协作、附件、商品上下文、导出原文/报告、左侧折叠等入口；这些仍是后续迁移和前端适配重点。

用户可感知变化：

- 用户登录新产品 `/commerce` 后，使用默认 “Agent 集群” 策略提问 `只输出 2+3 的计算结果，不要解释。`，页面直接回答 `5`。
- 页面不会再把这种短答展示成“数据诊断、预算评估、千川投放、创意产出、运营方案、交付文档”的多角色流程。
- 不再出现 raw JSON、`role_output`、`summary is too thin` 或 “请问您需要我帮您做什么？”。
- 真正的业务分析题仍会显示多智能体过程；短答和业务任务在体验上被区分开。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service quality_checker`
- [x] `cargo test -p agent --test default_roles`
- [x] `cargo test -p agent`
- [x] `cargo check`
- [x] `cargo clippy -p agent`
- [x] `cargo clippy -p service`
- [x] 默认前端 metadata 的真实 SSE smoke：`route_mode=chat_bypass`，输出 `5`，质量检查通过。
- [x] Edge 新产品真实登录 smoke：`/commerce` 发送短答问题，页面只显示欢迎语、用户问题、回答 `5` 三条消息，无右侧流程面板。
- [x] Edge 旧产品抽样：确认策略面板、模型选择、附件、商品上下文、导出、左侧折叠入口存在；旧产品短答仍会输出较重的策略补齐文本。
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm -C apps/web lint`

后续缺口：

- 旧产品的显式模型选择、回答策略细分、附件上传、商品上下文弹层、导出原文/报告等入口，新产品还没有完整迁移到同等可用水平。
- 外部搜索 adapter 未接入，短答 smoke 仍会记录本地 RAG 未命中和外部搜索不可用。
- 新产品需要继续以后端能力为准，逐步把旧产品 skill 生态、平台执行、教学/导出/素材能力迁移到 Rust 框架里。

## Post 8-4：模型、策略和岗位运行契约迁移

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 4 个问答补强项：`Post 8-4`。

本次目标：

- 把旧产品问答工作台里的模型选择、回答策略、学习层级、协作模式和岗位选择迁移为新产品后端运行契约。
- 前端只从后端读取可用选项并随 SSE 传参，不再靠前端文案或 prompt 拼接伪造策略能力。
- 保持前面已经迁移的 Agent skill、工作区沉淀、质量反馈、短答链路继续可用。

涉及范围：

- 产品内核：问答运行从“前端固定策略按钮”升级为“后端声明运行能力，前端提交结构化选择，Agent runtime 按选择调度”。
- 后端：新增 `GET /api/app/agent/runtime-options`，从 `services.toml` 暴露安全模型选项，从 `RoleCatalog` 暴露岗位选项；SSE metadata 支持 `model.name`、`learningLevel`、`primaryRole`、`selectedRoles`；Agent manager 支持按模型名选择不同 LLM manager；manual/single 协作模式会真实影响角色路由和任务图。
- 前端：问答 composer 新增运行配置条，展示模型、层级、协作模式和岗位；发送消息时把当前选择写入 SSE metadata 和会话消息 JSON。
- API：同步更新 `API_DOCS/app.md` 和 `API_DOCS/sse.md`。
- 设计文档：新增 `temp/DEVELOPMENT_DOCS/backend/agent_runtime_options_and_strategy_contract.md` 和 `temp/DEVELOPMENT_DOCS/frontend/agent_chat_runtime_options_adaptation.md`。

用户可感知变化：

- 用户登录 `/commerce` 后，输入框上方可以看到后端当前支持的模型、讲解层级、协作模式和虚拟员工岗位。
- 用户切换模型后，本轮 SSE 会把 `metadata.model.name` 发给后端，后端会使用对应的 `llm.name`，非法模型会直接返回 400，不会创建半截运行记录。
- 用户选择“指定单岗”时，本轮只让主岗位参与；选择“指定多岗”时，后端会按用户选中的岗位生成多角色任务图。
- 右侧过程事件里的 `route_decided` 会带出学习层级和实际模型名，便于用户确认本轮调度是否按选择执行。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent --test default_roles manual_strategy_metadata_routes_requested_roles`
- [x] `cargo test -p agent --test default_roles single_strategy_metadata_routes_only_requested_primary_role`
- [x] `cargo test -p agent --test default_roles`
- [x] `cargo test -p web-server sse`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm -C apps/web lint`
- [x] `pnpm -C apps/web build`
- [x] 真实 HTTP smoke：`GET /api/app/agent/runtime-options` 返回默认模型 `ollama-gpt-oss-20b`，模型列表包含 `ollama-gpt-oss-20b`、`ollama-gemma4`，岗位列表包含 `chat`、`ops`、`data`、`service`、`creative`。
- [x] 真实 HTTP smoke：非法 `metadata.model.name=not-configured-model` 返回 `400`，未启动 SSE。
- [x] 真实 SSE smoke：指定 `model.name=ollama-gpt-oss-20b`、`collaborationMode=manual`、`primaryRole=data`、`selectedRoles=[data,ops]` 后，`route_decided` 返回 `model_name=ollama-gpt-oss-20b`、主岗位 `data`、支持岗位 `ops`，并正常输出和完成。
- [x] Edge 页面 smoke：登录后进入 `/commerce`，页面可见“运行配置”、模型名和岗位选项。

后续缺口：

- 本轮解决的是问答运行契约，不是附件上传、导出报告、外部搜索 adapter 或真实平台写执行。
- 外部搜索仍未接入，实时趋势、竞品外部检索等旧产品能力还需要后端 adapter 继续迁移。
- 附件、素材文件和导出能力还需要继续以后端文件/交付物能力为主设计，再让前端适配。

## Post 8-5：显式岗位选择优先级与前端运行配置稳定性

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 5 个问答补强项：`Post 8-5`。

本次目标：

- 修复用户在前端选择“指定多岗/指定单岗”后，后端仍可能因为短答判断走 `chat_bypass` 的断点。
- 保留前一轮已经修好的轻问答能力：没有显式岗位选择的短答仍然可以快速直接回答。
- 让前端的岗位操作更接近后端真实契约：主岗位和参与岗位分开控制，避免用户以为选中了岗位但 metadata 不稳定。

涉及范围：

- 产品内核：显式岗位选择的优先级高于短答旁路；用户主动选择虚拟员工时，后端按岗位进入 commander/任务图。
- 后端：`SessionRuntime` 增加显式非 `chat` 岗位选择识别；`manual/single + primaryRole/selectedRoles` 会阻止 `chat_bypass` 抢先返回；`collab/manual` 且未选具体岗位的短答仍允许 `chat_bypass`。
- 前端：多岗模式下新增独立主岗位选择，岗位按钮只控制参与/不参与；手动多岗时不允许把仍被其他岗位依赖的主岗位误切掉。
- API：`API_DOCS/sse.md` 补充显式岗位选择优先级说明。

用户可感知变化：

- 用户在 `/commerce` 的问答输入区选择“指定多岗”，把主岗位设为“数据”，参与岗位选“运营”后，本轮后端会按 `data + ops` 生成任务图，而不是显示成普通聊天旁路。
- 用户仍然可以问简单短答，例如“只输出 2+3 的结果”，如果没有明确指定非 `chat` 岗位，页面会继续直接给出短答，不会强行启动多智能体流程。
- 运行配置条里“主岗位”和“岗位参与”含义更清楚，右侧过程区的 `route_decided` 更容易和用户刚才的选择对应起来。

当前验证：

- [x] `cargo fmt`（在 `backend/` 下执行；仓库根目录无 `Cargo.toml`）
- [x] `cargo test -p agent --test default_roles explicit_manual_roles_do_not_use_direct_answer_chat_bypass`
- [x] `cargo test -p agent --test default_roles collab_strategy_direct_answer_bypasses_task_graph`
- [x] `cargo test -p agent --test default_roles`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm -C apps/web lint`
- [x] `pnpm -C apps/web build`
- [x] 真实 HTTP SSE smoke：`collaborationMode=manual`、`primaryRole=data`、`selectedRoles=[data,ops]`、业务漏斗问题返回 `route_mode=task_graph`，任务图角色为 `data,ops`，并触发 `data_funnel_analysis` 与 `ops_promo_planning` 两个 native skill。
- [x] Edge 页面 smoke：登录 `/commerce` 后可见独立“主”岗位选择；页面真实发出的 SSE body 包含 `model.name=ollama-gpt-oss-20b`、`strategy.id=collab`、`collaborationMode=manual`、`primaryRole=data`，`selectedRoles` 包含 `data` 和 `ops`。

后续缺口：

- 外部搜索、附件上传、导出报告、平台执行 adapter 仍未在本轮迁移。
- 旧产品更完整的回答策略选择和教学/导出控制面仍需继续以后端能力为主迁移。

## Post 8-6：设计与 Web 增长 Skill Native 迁移

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 6 个问答补强项：`Post 8-6`。

本次目标：

- 把旧产品设计岗位和 Web 增长岗位里仍返回 deferred 的主图、详情页、配色、海报、SEO、标题、关键词、店铺装修、详情文案能力迁为新产品 Rust Agent runtime 的 native 结构化文本产物。
- 保持前端不造假：仍沿用真实 `skill_requested` / `skill_result` SSE 事件展示。
- 用户自然提问时，后端自动补偿能按意图命中对应 skill。

涉及范围：

- 产品内核：设计/Web 岗位从“能力壳 + 待接入提示”升级为“可直接输出电商设计 brief、SEO 方案和页面增长建议”的 native 能力。
- 后端：`backend/crates/agent/src/skills/design.rs`、`backend/crates/agent/src/skills/web.rs`、`backend/crates/agent/src/agent/worker/runner.rs`、`backend/crates/agent/src/skills/runtime.rs`。
- API：不新增接口，不修改 SSE 请求结构；沿用 `skill_requested` / `skill_result`。
- 前端：本阶段不改前端，继续展示后端真实 SSE 事件和最终回答。
- 文档：新增 Post 8-6 后端设计归档，更新 skill 迁移矩阵。

用户可感知变化：

- 用户问“做主图设计方案”“规划详情页长图”“做品牌配色”“做活动海报”“生成标题”“做 SEO 优化”“关键词研究”“店铺装修”“详情页文案”“页面转化优化”时，后端更稳定触发对应设计/Web skill。
- 过程区看到的 `skill_result` 状态应为 `success/native`，不再是“设计内容引擎未接入”或“Web 技能骨架已返回”的 deferred。
- 当前结果是结构化文本产物：设计 brief、平台规格、配色 HEX、SEO 提排路径、关键词矩阵、标题候选、店铺首页模块、详情页文案和 AB 测试清单。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent executor_runs_native_design_and_web_growth_skills`
- [x] `cargo test -p agent autosuggests_web_growth_skill_by_intent`
- [x] `cargo test -p agent autosuggests_design_skill_by_intent`
- [x] `cargo test -p agent`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`

后续缺口：

- 图片、海报、长图文件生成未做。
- 实时搜索量、CPC、竞品排名和店铺真实指标仍待外部搜索/平台数据 adapter。
- 素材上传、导出报告和 workspace artifact 文件化仍需后续阶段补齐。

## Post 8-7：客服处理 Skill Native 迁移

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 7 个问答补强项：`Post 8-7`。

本次目标：

- 把旧产品客服岗位中仍停留在 deferred 壳的工单处理、FAQ 话术库、升级流程迁为新产品 Rust Agent runtime 的 native 结构化能力。
- 保持客服能力不伪造实时平台政策：本地 SOP 能给具体处理方案和话术，赔付标准、平台投诉口径、特殊类目规则仍明确需要后续 adapter 校验。
- 扩展 service 岗位自动补偿规则，让自然语言客服任务能命中更准确的客服 skill。

涉及范围：

- 产品内核：客服虚拟员工从“DSR/退换货为主”升级为“能处理工单、FAQ、投诉升级、DSR、NPS和情绪识别”的岗位能力。
- 后端：`backend/crates/agent/src/skills/service.rs`、`backend/crates/agent/src/agent/worker/runner.rs`、`backend/crates/agent/src/skills/runtime.rs`。
- API：不新增接口，不修改 SSE 请求结构；沿用 `skill_requested` / `skill_result`。
- 前端：本阶段不改前端，继续展示后端真实 SSE 事件和最终回答。
- 文档：新增 Post 8-7 后端设计归档，更新 skill 迁移矩阵。

用户可感知变化：

- 用户问“客户投诉物流延迟很生气，生成工单处理话术”时，后端会触发 `service_ticket_handler`，输出优先级、SLA、处理策略、客服话术和升级条件。
- 用户问“生成客服常见 FAQ 话术库”时，后端会触发 `service_faq_playbook`，输出简短版、详细版和处理指引。
- 用户问“客户威胁投诉平台，需要主管升级处理流程”时，后端会触发 `service_escalation_flow`，输出严重度、升级路径、主管接管话术、赔偿建议和复核要求。
- DSR 提升不再只是“骨架”，会给出对应维度的具体改善动作；真实店铺指标和实时平台政策仍会明确标注需要后续 adapter。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent executor_runs_native_service_support_skills`
- [x] `cargo test -p agent autosuggests_service_support_skill_by_intent`
- [x] `cargo test -p agent`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `cargo check`
- [x] `cargo clippy`

后续缺口：

- `service_query_product` 仍需要真实商品数据库/商品画像 adapter。
- 平台最新退换货、投诉、赔付规则仍需要 `search_platform_policy` 或 `shop_policy` adapter。
- 客服处理方案当前仍是结构化文本产物，尚未生成可下载客服 SOP 文档或工单 artifact 文件。

## Post 8-8：外部搜索 Adapter 契约化

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 8 个补强项：`Post 8-8`。

本次目标：

- 把旧产品趋势、竞品、市场行情、商品评价、平台政策、行业基准搜索能力迁成新产品 Agent runtime 中稳定的后端契约。
- 当前真实 search provider 尚未接入，因此不伪造联网结果、不返回假 sources、不声明假搜索引擎。
- 为后续外部搜索/实时数据 adapter 接入提前固定输出结构。

涉及范围：

- 产品内核：搜索 skill 从简单 deferred 占位升级为统一 `adapter_required` fallback。
- 后端：`backend/crates/agent/src/skills/search.rs`、`backend/crates/agent/src/skills/runtime.rs`。
- API：本轮不新增 HTTP API，不修改 SSE 事件结构；沿用 `skill_result.status=deferred`。
- 前端：本轮不改前端，不增加假来源展示。
- 文档：新增 Post 8-8 后端设计归档和迁移归档，更新 skill 迁移矩阵。

用户可感知变化：

- 用户问“搜索咖啡趋势”“查竞品价格”“看平台最新规则”“找行业 ROI 基准”等问题时，后端会明确识别为搜索类 skill。
- 当前由于外部搜索 adapter 未接入，过程区仍显示 `deferred`，不会让用户误以为已经获取了实时互联网来源。
- 后端输出会保留计划搜索词、topic、时间范围、空来源、0 置信度和明确降级原因，后续接入真实 provider 后可直接替换为 live 数据。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent executor_returns_deferred_for_search`
- [x] `cargo test -p agent executor_returns_structured_deferred_for_search_skills`
- [x] `cargo test -p agent`
- [x] `cargo check -p agent`
- [x] `cargo clippy -p agent`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `git diff --check`

后续缺口：

- 真实 search provider 配置和执行器未接入。
- 尚未新增 Rust 外部搜索 API 或 service adapter。
- 前端尚未展示 search output 中的 `query/sources/provenance/confidence/degraded_reason` 细节。

## Post 8-9：Rust 外部搜索 Adapter API

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 9 个补强项：`Post 8-9`。

本次目标：

- 在新产品 Rust 后端补出外部搜索 adapter 的真实 API 入口。
- 配置未开启时返回标准降级结构，不伪造外部来源。
- 配置开启且 provider 可用时，按统一参数请求 provider，并归一化来源、provenance、confidence 和降级原因。

涉及范围：

- 产品内核：外部搜索从 Agent 内部 deferred 契约，向 service/web-server 可调用 adapter 入口推进。
- 后端：新增 `ExternalSearchApi`、外部搜索 DTO、web-server 静态配置、handler 和 routes。
- API：新增 `GET /api/app/external-search/status` 与 `GET /api/app/external-search/search`，同步 `API_DOCS/app.md`。
- 配置：`backend/config/services-example.toml` 新增 `[external_search]` 示例，默认关闭。
- 前端：本阶段不改前端，不新增假展示。

用户可感知变化：

- 当前产品后端已经可以回答“外部搜索 adapter 是否配置”。
- 未配置时，用户不会看到假的互联网搜索来源；后端会返回 `status=degraded`、`sources/items=[]` 和明确降级原因。
- 后续配置真实 provider 后，前端和 Agent skill 可以复用该 API 的 `data.items`、`provenance` 和 `confidence` 字段展示实时来源。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service external_search`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `git diff --check`

后续缺口：

- Agent `search_*` skill 尚未直接调用该 API 或共享 adapter client。
- 前端尚未接入 `/api/app/external-search/*` 展示来源、置信度和降级原因。
- 真实 provider 需要后续在部署环境配置 `[external_search]`。

## Post 8-10：Agent 搜索 Skill 接入外部搜索 Adapter

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 10 个补强项：`Post 8-10`。

本次目标：

- 让新产品 Agent runtime 内部的 `search_*` skill 使用 `[external_search]` 配置尝试真实搜索。
- 有真实 provider 和可归一化来源时返回 `status=live`。
- provider 未配置、请求失败或无来源时返回 `status=deferred`、`adapter_status=degraded` 或 `adapter_required`，不伪造来源。

涉及范围：

- 产品内核：搜索能力从静态降级契约升级为可配置 adapter 执行。
- 后端：新增 Agent `SearchAdapterConfig`、搜索 provider 请求、来源/provenance/confidence 归一化、worker skill executor 配置注入。
- Web-server：启动时把同一份 `[external_search]` 配置传入 Agent runtime。
- API/SSE：不新增路由；更新 `API_DOCS/sse.md`，说明搜索类 `skill_result` 的 live/degraded 语义。
- 前端：本阶段不改前端，不新增假搜索展示。

用户可感知变化：

- 用户问“搜索咖啡趋势”“查竞品价格”“看平台最新规则”“找行业 ROI 基准”等问题时，后端现在具备真实调用外部搜索 provider 的路径。
- 如果部署环境配置了 provider 且返回可信来源，过程区的 `skill_result` 可显示真实来源、provenance 和 confidence。
- 如果 provider 未配置或失败，过程区仍显示降级原因和空来源，不会让用户误以为已经查到实时资料。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent executor_returns_deferred_for_search`
- [x] `cargo test -p agent executor_returns_structured_deferred_for_search_skills`
- [x] `cargo test -p agent executor_reports_degraded_when_configured_search_provider_fails`
- [x] `cargo test -p agent executor_runs_representative_native_categories`
- [x] `cargo test -p agent`
- [x] `cargo check -p agent`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `git diff --check`

后续缺口：

- 真实部署仍需要配置可访问的 `[external_search]` provider。
- 前端尚未把搜索来源、置信度和降级原因整理成专门来源面板。
- App API 和 Agent adapter 目前共享字段语义，但实现仍各自归一化，后续可抽共享 crate。

## Post 8-11：Agent 搜索证据沉淀为 Workspace Artifact

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 11 个补强项：`Post 8-11`。

本次目标：

- 让 Agent `search_*` skill 的真实 live 搜索结果不只停留在 SSE 过程事件里。
- 当搜索输出有可信来源时，把完整搜索输出沉淀为 `artifactType=search_evidence` 的工作区交付物。
- 未配置、降级或无来源时不创建假证据。

涉及范围：

- 产品内核：搜索能力从“可实时调用”继续推进到“可沉淀、可追溯、可被后续任务复用”。
- Agent：`skill_result` 事件增加可选结构化 `output` 字段。
- 后端 service：`WorkspaceApi::record_agent_search_evidence` 复用 `workspace_artifacts` 创建搜索证据交付物。
- Web-server：SSE 收到 live 搜索结果后自动写入 `search_evidence`，并发送 `workspace_artifact_created`。
- API/SSE：更新 `API_DOCS/sse.md` 与 `API_DOCS/app.md`，说明 `search_evidence` artifact 契约。
- 前端：本阶段不改前端；前端后续可直接查询 artifact 或监听 SSE 事件展示来源面板。

用户可感知变化：

- 用户发起需要实时搜索的电商问题时，如果 provider 返回真实来源，工作区里会多出“搜索证据”交付物。
- 前端后续可在右侧来源面板、工作区历史或任务详情里展示这些来源、域名、置信度和 freshness。
- 如果搜索 adapter 不可用，用户不会看到假的搜索证据。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service search_evidence_artifact`
- [x] `cargo test -p web-server skill_result_output_is_camelized_for_sse_consumers`
- [x] `cargo test -p agent successful_skill_results_become_report_evidence`
- [x] `cargo test -p agent`
- [x] `cargo check -p agent`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `git diff --check`

后续缺口：

- 前端尚未把 `search_evidence` artifact 做成 Kimi Agent 风格的来源面板。
- 真实 provider 仍需要部署环境配置 `[external_search]`。
- 后续可把 search adapter 归一化逻辑抽到共享 crate，减少 App API 与 Agent 内部 adapter 的重复。

## Post 8-12：附件与素材上下文摘要 API

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 12 个补强项：`Post 8-12`。

本次目标：

- 让旧产品里的附件、素材、报告交付物，不再只是分散在不同接口中。
- 先由后端聚合成可被 Agent 和前端共同消费的“上下文就绪摘要”。
- 为后续问答前上下文注入、右侧来源面板、导出/教学能力打基础。

涉及范围：

- 产品内核：从单独文件/素材 CRUD 继续推进到“Agent 可判断哪些上下文可用”。
- 后端 service：`WorkspaceApi::context_summary` 聚合 `conversation_files`、`product_assets`、`product_materials`、`workspace_artifacts`。
- Web-server：新增 `GET /api/app/workspaces/{workspaceId}/context-summary`。
- API：更新 `API_DOCS/app.md`，记录请求参数、返回结构、权限和降级语义。
- 前端：本阶段不改前端，不新增假状态。

用户可感知变化：

- 后续前端可以在问答工作台显示“当前有多少附件、商品素材、交付物可用于回答”。
- 当没有上传附件或没有素材时，产品可以明确提示缺失来源，而不是让用户误以为 Agent 已经读取了资料。
- 后续 Agent 问答可以基于 `agentContext.refs` 注入真实来源引用。

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p service context_summary`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `git diff --check`

后续缺口：

- 还需要把该摘要接入 Agent 请求 metadata，让问答真正自动读取可用附件/素材引用。
- 前端尚未基于该接口做来源面板、附件状态提示和导出/教学入口。

## Post 8-13：工作区上下文摘要接入 Agent 问答主链路

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 13 个补强项：`Post 8-13`。

本次目标：

- 把 `WorkspaceApi::context_summary` 从独立查询 API 接到 Agent SSE 主链路。
- 让会话附件、商品资产、商品素材、工作区交付物在问答时进入后端 runtime metadata。
- 前端只做事件消费和过程展示，不造假按钮。

涉及范围：

- 产品内核：问答主链路开始感知工作区附件/素材/交付物来源。
- 后端 web-server：`metadata.context.conversationId` 进入 SSE DTO；绑定 workspace run 后调用 `context_summary`。
- Agent metadata：新增 `workspaceContext.*`，并把 workspace refs 追加到 `context.refs`。
- SSE：新增 `workspace_context_loaded` 事件。
- 前端：现有右侧上下文卡片和时间线识别 `workspace_context_loaded`。
- API：同步 `API_DOCS/sse.md` 和 `API_DOCS/app.md`。

用户可感知变化：

- 问答页发送任务后，右侧过程区会显示工作区上下文来源是否已加载。
- 如果会话附件、商品素材或工作区交付物为空，用户能看到明确 empty/partial 状态。
- Agent 回答前的上下文来源不再只停留在后台接口里，而是进入问答链路。

当前验证：

- [x] `cargo fmt`
- [x] `cargo fmt --check`
- [x] `cargo test -p web-server workspace_context`
- [x] `cargo test -p web-server metadata_parses_conversation_id_context_selection`
- [x] `cargo test -p web-server metadata_rejects_too_long_conversation_id`
- [x] `cargo test -p web-server sse`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm -C apps/web lint`
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web build`
- [x] `pnpm rules`
- [x] `git diff --check`

## Post 8-14：工作区上下文内容包接入 Agent 问答链路

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 14 个补强项：`Post 8-14`。

本次目标：

- 把旧产品附件/素材/报告作为问答上下文的能力继续迁入新产品。
- 后端不只给 Agent refs 和来源摘要，还把数据库已有可读文本整理成内容包。
- 严格避免伪造文件读取：不读 object storage，不做 OCR，不解析 PDF/图片/视频正文。

涉及范围：

- 产品内核：工作区上下文从“来源可见”推进到“可读文本可用于回答”。
- 后端 service：新增 `WorkspaceApi::context_pack`。
- Web-server：新增 `GET /api/app/workspaces/{workspaceId}/context-pack`，SSE 新增 `workspace_context_pack_loaded`。
- Agent metadata：新增 `workspaceContextPack.*`，可读文本追加到 `context.summary`。
- 前端：同步 SSE 类型、时间线和右侧上下文卡片。
- API：同步 `API_DOCS/app.md` 与 `API_DOCS/sse.md`。

用户可感知变化：

- 用户发送问答后，右侧过程区会显示“可读上下文”是否接入。
- 商品素材正文、资产描述、工作区交付物正文/预览、附件 metadata 已抽取文本会真实进入回答链路。
- 二进制附件没有抽取文本时只显示 metadata-only 和警告，不会让用户误以为 Agent 已经读取正文。

当前验证：

- [x] `cargo fmt`
- [x] `cargo fmt --check`
- [x] `cargo test -p service context_pack`
- [x] `cargo test -p web-server workspace_context_pack`
- [x] `cargo test -p web-server sse`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm -C apps/web lint`
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web build`
- [x] `pnpm rules`
- [x] `git diff --check`

后续缺口：

- 附件正文抽取/OCR/文档解析还没有实现，需要后续作为独立 adapter 接入。
- 可读内容包已经进入 Agent metadata，但还可以继续做前端来源详情面板和 artifact 复用入口。

## Post 8-15：工作区交付物导出能力

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 15 个补强项：`Post 8-15`。

本次目标：

- 把旧产品“回答原文 / 报告导出”迁到新产品 `workspace_artifacts` 后端体系。
- 导出结果必须来自后端真实交付物，不由前端临时拼假文件。

涉及范围：

- 产品内核：交付物从“可查询详情”推进到“可复制、可下载、可治理导出”。
- 后端 service：新增 `WorkspaceApi::export_artifact`。
- Web-server：新增 `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}/export`。
- API：同步 `API_DOCS/app.md`。
- 前端：工作区交付物详情接入导出 API，新增复制导出和下载导出。

用户可感知变化：

- 用户进入“执行工作区 -> 交付物”，选中报告或搜索证据后，可以复制后端导出的完整内容。
- 用户可以下载 `.md`、`.txt` 或 `.json` 交付文件。
- 如果交付物只有预览或对象存储引用，前端会显示后端 warning，说明当前导出限制。

当前验证：

- [x] `cargo fmt`
- [x] `cargo fmt --check`
- [x] `cargo test -p service artifact_export`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm -C apps/web lint`
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web build`
- [x] `pnpm rules`
- [x] `git diff --check`

后续缺口：

- 本轮不读取 object storage，不做 OCR、PDF、图片或视频解析。
- 后续需要继续做附件正文抽取 adapter，以及导出历史/批量导出/正式文件服务。

## Post 8-16：会话附件抽取文本登记 API

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 16 个补强项：`Post 8-16`。

本次目标：

- 为旧产品“附件正文进入问答上下文”的能力补齐新产品后端契约。
- 后端提供可信抽取文本登记 API，不在本轮读取 object storage，不做 OCR/PDF/图片解析。

涉及范围：

- 产品内核：会话附件从“只有文件元数据”推进到“可登记可读正文，并被上下文包读取”。
- 后端 repo：新增 `ConversationFileService::update_metadata`。
- 后端 service：新增 `ConversationApi::update_file_extraction`。
- Web-server：新增 `PUT /api/app/conversation-files/{fileId}/extraction`。
- API：同步 `API_DOCS/app.md`。
- 前端：同步 `ConversationFileResponse.metadata` 和 `updateConversationFileExtractionApi`。

用户可感知变化：

- 用户附件被后续解析器登记正文后，Agent 问答能真实使用附件文本。
- 工作区上下文内容包会显示附件从 `metadata_only` 变为 `extracted_text`。
- 当前仍不会假装读取未解析的二进制文件。

当前验证：

- [x] `cargo fmt`
- [x] `cargo fmt --check`
- [x] `cargo test -p service conversation_file_extraction`
- [x] `cargo check -p repo`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm -C apps/web lint`
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web build`
- [x] `pnpm rules`
- [x] `git diff --check`

后续缺口：

- 还需要实现 OCR/文档解析 adapter，把文件读取、解析、状态队列和失败重试接入本 API。
- 还需要做前端附件解析入口和解析状态展示。

## Post 8-17：竞品分析读取 API

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 17 个补强项：`Post 8-17`。

本次目标：

- 把旧产品竞品分析能力迁成新产品后端聚合读取 API。
- 让前端和 Agent 不再只能消费竞品散列表，而是能读取价格位置、平台分布、评分、近期变化和上下文引用。

涉及范围：

- 产品内核：竞品数据从 CRUD 推进到结构化分析能力。
- 后端 service：新增 `ProductCompetitorApi::competitor_analysis`。
- Web-server：新增 `GET /api/app/products/{productId}/competitors/analysis`。
- API：同步 `API_DOCS/app.md`。
- 前端：同步 `getProductCompetitorAnalysisApi` 和响应类型。

用户可感知变化：

- 后续前端接入后，用户可在商品竞品区看到分析摘要、价格位置、平台分布和近期变化。
- Agent 后续可用 `agentContext` 回答竞品和定价问题。
- 当前不会假装已自动搜索竞品，也不会自动生成报告。

当前验证：

- [x] `cargo fmt`
- [x] `cargo fmt --check`
- [x] `cargo test -p service competitor_analysis`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm -C apps/web lint`
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web build`
- [x] `pnpm rules`
- [x] `git diff --check`

后续缺口：

- 还需要实现竞品自动搜索/监控 adapter。
- 还需要将竞品分析结果沉淀为 workspace artifact。

## Post 8-18：Campaign draft 创建 API

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 18 个补强项：`Post 8-18`。

本次目标：

- 把旧产品中 Agent 可以创建营销活动草稿的能力迁成新产品后端 API。
- 让工作区交付物、Agent 输出和商品上下文可以沉淀为真实 campaign draft。

涉及范围：

- 产品内核：营销活动从普通 CRUD 推进到 Agent/工作区可沉淀草稿。
- 后端 service：新增 `CampaignApi::create_campaign_draft`。
- Web-server：新增 `POST /api/app/campaigns/draft`。
- API：同步 `API_DOCS/app.md`。
- 前端：同步 `createCampaignDraftApi` 和响应类型。

用户可感知变化：

- 后续前端接入后，用户可在 Agent 交付物或工作区里把投放方案保存为营销活动草稿。
- 草稿会保留商品、工作区、交付物和策略来源，便于后续继续编辑、审批和执行。
- 当前不会自动发布到抖音、淘宝等平台。

当前验证：

- [x] `cargo fmt`
- [x] `cargo fmt --check`
- [x] `cargo test -p service campaign_draft`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm -C apps/web lint`
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web build`
- [x] `pnpm rules`
- [x] `git diff --check`

后续缺口：

- 还需要前端交付物卡片接入创建活动草稿入口。
- 还需要草稿审批、平台执行 adapter 和审计日志闭环。

## Post 8-19：产品生命周期状态 API

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 19 个补强项：`Post 8-19`。

本次目标：

- 把旧产品 `draft -> listing -> active -> archived` 商品生命周期迁成新产品后端状态机。
- 让前端和 Agent 后续从后端读取生命周期阶段、允许流转和上下文，而不是前端硬编码状态。

涉及范围：

- 产品内核：商品从 `active / archived` 粗粒度状态扩展为选品、上架、运营、归档四阶段。
- Repo：扩展 `ProductStatus`，并让创建商品可写入指定状态。
- 后端 service：新增 lifecycle options、detail、transition 逻辑和状态机校验。
- Web-server：新增 `GET /api/app/products/lifecycle/options`、`GET/PATCH /api/app/products/{productId}/lifecycle`。
- API：同步 `API_DOCS/app.md`。
- 前端：同步 lifecycle API adapter 和响应类型。

用户可感知变化：

- 后续前端接入后，用户可在商品页看到商品处于选品中、上架中、运营中或已归档。
- 用户只能按后端允许的路径推进生命周期，例如 `draft -> listing -> active -> archived`。
- Agent 后续可根据商品当前阶段给出更贴合阶段的电商运营建议。

当前验证：

- [x] `cargo fmt`
- [x] `cargo fmt --check`
- [x] `cargo test -p service product_lifecycle`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

后续缺口：

- 还需要把商品页 UI 接到 lifecycle API，展示阶段和可执行下一步。
- 还需要让 Agent task graph 在商品相关任务中消费 lifecycle agentContext。

## Post 8-20：Skill run 记录查询 API

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 20 个补强项：`Post 8-20`。

本次目标：

- 把旧产品 skill run audit 能力迁成新产品后端可持久化、可查询的 Agent skill run 记录。
- 让 `skill_requested` / `skill_result` 不再只停留在 SSE 流式过程里，而是进入 workspace run 的审计轨迹。

涉及范围：

- 产品内核：Skill 调用从“过程可见”推进到“历史可查、可审计、可与审批/搜索证据关联”。
- 后端 migration/repo：新增 `agent_skill_runs` 表和单表 repo service。
- 后端 service：新增 skill run 查询、请求落库、结果落库能力。
- Web-server：SSE 收到 `SkillRequested` / `SkillResult` 时自动写记录；新增 `GET /api/app/workspaces/{workspaceId}/runs/{runId}/skill-runs`。
- API：同步 `API_DOCS/app.md`。
- 前端：同步 `listAgentSkillRunsApi` 和响应类型，但本阶段不新增页面入口。

用户可感知变化：

- 后续工作区运行详情页接入后，用户可以看到某次 Agent 运行调用过哪些 skill。
- 用户能看到 skill 的角色、状态、执行模式、风险等级、是否需要审批、摘要、错误和输出结构。
- 失败、降级、待审批和成功状态都来自后端真实 Agent runtime，不由前端伪造。

当前验证：

- [x] `cargo fmt`
- [x] `cargo fmt --check`
- [x] `cargo test -p service skill_run`
- [x] `cargo check -p repo`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

后续缺口：

- 还需要在工作区运行详情页接入 skill run 历史列表。
- 还需要把 skill run 与搜索证据 artifact、审批记录、告警做前端关联展示。
