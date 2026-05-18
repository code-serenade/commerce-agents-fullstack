# Post 8-134：千川投放 native skills

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 134 个补强项：`Post 8-134`。

本次目标：

- 将旧产品中千川投流、赛马机制、计划结构、人群包、预算节奏、ROI/ROAS 止损和直播间漏斗诊断能力，迁成新产品 `douyin_ads` 角色下的 Rust Agent native skills。
- 让已经存在的“千川投放专员”不再只停留在角色 prompt 和任务图，而是能通过真实 `skill_requested` / `skill_result` 返回结构化结果。

涉及范围：

- 产品内核：新增千川投放 native skill 能力，仍使用新产品 Agent skill registry、runtime、role skill catalog、worker autosuggest 和 workspace artifact 体系。
- 后端 Agent：新增 `douyin_ads_plan_structure`、`douyin_ads_audience_package`、`douyin_ads_budget_pacing`、`douyin_ads_live_funnel_diagnosis`。
- 角色权限：`douyin_ads` 可以使用自身 native skills，并可申请 `search_trends`、`search_market_info`、`search_platform_policy`、`search_industry_benchmarks`。
- 自动触发：千川计划、冷启动、赛马、人群包、定向、预算节奏、ROI/ROAS、直播间漏斗、进房、商品点击、付款等问法可触发对应 native skill。
- 工作区交付物：成功输出可沉淀为 `douyin_ads_plan_structure`、`douyin_ads_audience_package`、`douyin_ads_budget_pacing`、`douyin_ads_live_funnel_report`。
- API 文档：同步更新根目录 `API_DOCS/app.md` 和 `API_DOCS/sse.md`。

用户可感知变化：

- 用户问“给这个商品做千川冷启动计划结构，预算2万，目标ROAS2.8”时，过程区会看到 `douyin_ads_plan_structure`，并得到计划结构、预算拆分、KPI 和止损规则。
- 用户问“做千川人群包和定向设计”时，过程区会看到 `douyin_ads_audience_package`，并得到核心转化、相似扩展、内容兴趣和再营销人群包。
- 用户问“复盘千川预算节奏：预算5万，消耗1.2万，GMV4.2万”时，过程区会看到 `douyin_ads_budget_pacing`，并得到当前 ROAS、预算剩余、放量 / 稳投 / 收缩 / 止损判断。
- 用户问“诊断直播间漏斗：曝光、点击、进房、商品点击、下单、付款...”时，过程区会看到 `douyin_ads_live_funnel_diagnosis`，并得到各环节转化率、主要瓶颈和动作建议。
- 如果用户同时要求“最新平台规则 / 行业ROI基准 / 人群趋势 / 联网资料”，后端会追加独立 `search_*` skill；搜索失败或未配置 provider 时按搜索降级，不会伪造外部来源。

边界说明：

- 本轮不接巨量千川、抖音罗盘、直播间后台、广告后台或实时数仓。
- 所有计算只基于本轮显式用户输入和显式选品上下文。
- 未显式选择商品时不会注入默认商品、历史商品或测试商品。
- 预算节奏缺少预算或 ROAS / GMV / 消耗时返回 `deferred`。
- 直播漏斗缺少曝光和点击时返回 `deferred`。

当前验证：

- [x] `cargo fmt --all`（在 `backend/` workspace 下执行；根目录没有 `Cargo.toml`）
- [x] `cargo test -p agent douyin_ads`
- [x] `cargo test -p agent keeps_roles_from_requesting_unowned_skills`
- [x] `cargo test -p service content_artifact_builds_from_douyin_ads_skill_output`
- [x] `cargo test -p agent staged_registry_loads_all_specs`
- [x] `cargo test -p agent executor_runs_representative_native_categories`
- [x] `cargo check -p agent -p service`
- [x] `cargo clippy -p agent -p service`
- [x] `git diff --check`

后续缺口：

- 真实千川 / 抖音官方数据读取仍需要后续平台 adapter 或数据 adapter。
- 前端后续需要把这些 native skill 和 search 双 skill 链路以可折叠过程卡片、右侧过程面板和工作区交付物列表展示出来。
