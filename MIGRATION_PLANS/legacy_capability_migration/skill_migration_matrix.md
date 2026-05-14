# Skill 迁移矩阵

状态说明：

- `pending_review`：尚未核对 Rust 实现。
- `spec_exists`：已有 Rust `SkillSpec`，但执行能力待核对。
- `native_ready`：Rust native 执行可用。
- `adapter_required`：需要搜索、LLM、DB、平台等 adapter。
- `approval_required`：高风险平台写操作，需要审批。
- `missing`：新产品尚无对应能力。

## 运营 ops

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `ops_promo_planning` | `ops_promo_planning` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 当前已可 native 执行；已补强活动/促销自然问法触达；Post 8-35：成功输出会沉淀为 `ops_promo_plan` artifact |
| `ops_pricing_strategy` | `ops_pricing_strategy` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已纳入岗位自动补偿规则，可由 ops 定价意图触发；Post 8-35：成功输出会沉淀为 `ops_pricing_strategy` artifact |
| `ops_channel_strategy` | `ops_channel_strategy` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已补强渠道/渠道分配/投放渠道等自然问法触达；Post 8-35：成功输出会沉淀为 `ops_channel_strategy` artifact |
| `ops_inventory_planning` | `ops_inventory_planning` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已纳入岗位自动补偿规则，当前基于输入数字做本地规划；Post 8-35：成功输出会沉淀为 `ops_inventory_plan` artifact |
| `ops_listing_copy` | `ops_listing_copy` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已补强上架文案/标题卖点/上新文案自然问法触达；Post 8-35：成功输出会沉淀为 `ops_listing_copy` artifact，缺商品上下文时 deferred；Post 8-70：自动补偿 input 优先读取本轮显式 `context.selected_summary` 的商品名、平台和卖点，未选商品时不从历史摘要注入默认商品；Post 8-71：支持 `product_ref=product:{uuid}`，旧 `product_id` 仅兼容旧整数参数 |
| `ops_assortment_planning` | `ops_assortment_planning` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已补强货盘/SKU组合/选品组合自然问法触达；Post 8-35：成功输出会沉淀为 `ops_assortment_plan` artifact |
| `ops_execution_plan` | `ops_execution_plan` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已纳入岗位自动补偿规则，并有单测覆盖；Post 8-35：成功输出会沉淀为 `ops_execution_plan` artifact；Post 8-71：支持 `product_ref=product:{uuid}` 透传主推商品引用 |
| `ops_smart_pricing` | `ops_smart_pricing` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已可 native 执行；Post 8-36：成功输出会沉淀为 `ops_smart_pricing_report` artifact，缺竞品价格时明示 `calculated_from_defaults` |
| `ops_inventory_optimizer` | `ops_inventory_optimizer` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已可 native 执行；Post 8-36：成功输出会沉淀为 `ops_inventory_optimization_report` artifact，缺 SKU 明细时 deferred |
| `ops_ad_fatigue_detector` | `ops_ad_fatigue_detector` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已可 native 执行；Post 8-36：成功输出会沉淀为 `ops_ad_fatigue_report` artifact，缺渠道数据时 deferred |
| `ops_abc_xyz_classifier` | `ops_abc_xyz_classifier` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已可 native 执行；Post 8-36：成功输出会沉淀为 `ops_abc_xyz_report` artifact，使用确定性演示 SKU 时明示 `calculated_from_defaults` |

## 数据 data

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `query_store_metrics` | `query_store_metrics` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` / `backend/crates/service/src/api/workspace.rs` | Post 8-62：已迁为 Rust native 显式指标查询/汇总；有本轮 GMV、订单、UV、广告费、退款、毛利或 `platforms[]` 输入时返回 `manual_input/explicit_metrics`，无真实指标时返回 `store_metrics_required`，不伪造平台实时数据。Post 8-63：已补自然语言触达，可从“查询/加载店铺指标”和显式指标文本自动补偿该 skill；未明确平台时为 `all`。Post 8-67：成功输出会沉淀为 `data_store_metrics_report` artifact，deferred 不创建假报告 |
| `query_product_metrics` | `query_product_metrics` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` / `backend/crates/service/src/api/workspace.rs` | Post 8-65：已迁为 Rust native 商品/SKU指标查询；有本轮显式商品指标时返回 `manual_input/explicit_product_metrics`，包含 `totals`、`derived_metrics`、`week_over_week`、`risks` 和 `next_actions`；无显式指标时返回 `product_metrics_required`，不默认读取任何预设商品，不伪造平台实时商品数据。已补“查询商品指标/商品数据/单品数据/SKU指标/商品表现”等自然触达。Post 8-66：成功输出会沉淀为 `data_product_metrics_report` artifact，deferred 不创建假报告；Post 8-75：支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，未选商品时不写空引用 |
| `query_campaign_metrics` | `query_campaign_metrics` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` / `backend/crates/service/src/api/workspace.rs` | Post 8-65：已迁为 Rust native 活动/投放活动指标查询；有本轮显式活动指标时返回 `manual_input/explicit_campaign_metrics`，包含预算使用率、CTR、转化率、ROAS、ROI、风险和动作建议；无显式指标时返回 `campaign_metrics_required`，不伪造淘宝、抖音、千川或其它平台实时活动数据。已补“活动指标/活动数据/活动复盘/活动表现/投放活动数据”等自然触达。Post 8-66：成功输出会沉淀为 `data_campaign_metrics_report` artifact，deferred 不创建假报告 |
| `data_funnel_analysis` | `data_funnel_analysis` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | 已纳入岗位自动补偿规则，并有 stream 级测试覆盖；Post 8-31：有用户数据时成功输出会沉淀为 `data_funnel_report` artifact；Post 8-77：支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，未选商品时不写空引用 |
| `data_channel_roi` | `data_channel_roi` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | Post 8-31：有用户渠道数据时成功输出会沉淀为 `data_channel_roi_report` artifact。Post 8-68：runtime 契约修正为 native deterministic；渠道 ROI 问法可从本轮文本解析 `channels[]`，如抖音/淘宝 GMV、广告费、订单、UV；缺渠道数据时仍 deferred，不伪造平台投放数据。Post 8-78：支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，成功输出回显 `商品引用`，未选商品时不写空引用 |
| `data_refund_decomposition` | `data_refund_decomposition` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | Post 8-31：有用户退款项数据时成功输出会沉淀为 `data_refund_report` artifact。Post 8-68：runtime 契约修正为 native deterministic；退款帕累托 / 退款拆解问法可从本轮文本解析 `items[]`，如质量问题、物流破损、色差等退款金额；缺退款项时仍 deferred，不伪造售后平台数据。Post 8-79：支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，成功输出回显 `商品引用`，未选商品时不写空引用 |
| `data_anomaly_diagnosis` | `data_anomaly_diagnosis` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已纳入岗位自动补偿规则；Post 8-37：成功输出会沉淀为 `data_anomaly_report` artifact，时序点不足时 deferred |
| `data_trend_forecast` | `data_trend_forecast` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已纳入岗位自动补偿规则；Post 8-37：成功输出会沉淀为 `data_trend_forecast_report` artifact，时序点不足时 deferred |
| `data_customer_segmentation` | `data_customer_segmentation` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已纳入岗位自动补偿规则；Post 8-37：成功输出会沉淀为 `data_customer_segmentation_report` artifact，使用默认估算时明示 `calculated_from_defaults` |
| `data_competitor_analysis` | `data_competitor_analysis` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | 已补强竞品/对标/竞对自然问法触达；Post 8-37：成功输出会沉淀为 `data_competitor_report` artifact，缺竞品列表时明示 `calculated_from_defaults`；Post 8-76：支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，未选商品时不写空引用 |
| `data_dashboard` | `data_dashboard` | native_ready | `backend/crates/agent/src/skills/data.rs` | Post 8-29：已迁为 Rust native 经营看板；有入参时按真实入参计算，无入参时明确 `store_metrics_required`，不伪造平台实时数据。Post 8-30：成功输出会沉淀为 `data_dashboard` workspace artifact |
| `data_ab_test_analyzer` | `data_ab_test_analyzer` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已可 native 执行；Post 8-37：成功输出会沉淀为 `data_ab_test_report` artifact，样本/转化数不合法时返回 validation error |
| `data_multi_period_trend` | `data_multi_period_trend` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_multi_period_trend_report` artifact，时序点不足时 deferred |
| `data_attribution_analysis` | `data_attribution_analysis` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | Post 8-58：已迁为 Rust native 多模型渠道归因；成功输出会沉淀为 `data_attribution_report` artifact，缺多渠道输入时 deferred，不伪造平台数据。Post 8-80：支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，成功输出回显 `商品引用`；`渠道归因/多触点归因/归因模型` 自然问法可触达，未选商品时不写空引用 |
| `data_ltv_calculator` | `data_ltv_calculator` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_ltv_report` artifact，使用默认估算时明示 `calculated_from_defaults`。Post 8-81：支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，成功输出回显 `商品引用`；`LTV/客户价值/生命周期价值/获客成本回收` 自然问法可触达，未选商品时不写空引用 |
| `data_cohort_analysis` | `data_cohort_analysis` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_cohort_report` artifact，使用默认新客数时明示 `calculated_from_defaults`。Post 8-84：支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，成功输出回显 `商品引用`；`队列留存/同期群/留存分析/复购留存/新客留存/流失节点` 自然问法可触达，未选商品时不写空引用 |
| `data_seasonal_decompose` | `data_seasonal_decompose` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_seasonal_decompose_report` artifact，时序点不足时 deferred。Post 8-85：支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，成功输出回显 `商品引用`；`季节性/周期性/周期分解/季节分解/周内波动/周期波动` 自然问法可触达，未选商品时不写空引用 |
| `data_price_elasticity` | `data_price_elasticity` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_price_elasticity_report` artifact，使用默认价格点时明示 `calculated_from_defaults`。Post 8-82：支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，成功输出回显 `商品引用`；`价格弹性/价格敏感/需求弹性/调价测试/涨价影响/降价影响` 自然问法可触达，未选商品时不写空引用 |
| `data_demand_forecast` | `data_demand_forecast` | native_ready | `backend/crates/agent/src/skills/data.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_demand_forecast_report` artifact，时序点不足时 deferred。Post 8-83：支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，成功输出回显 `商品引用`；`需求预测/销量预测/订单预测/备货预测/补货预测/库存预测` 自然问法可触达，未选商品时不写空引用 |
| `data_comprehensive_diagnosis` | `data_comprehensive_diagnosis` | native_ready | `backend/crates/agent/src/skills/data.rs` | Post 8-59：已迁为 Rust native 综合诊断编排器，组合经营看板、漏斗、异常、客户、退款、预测、归因和 LTV 分项；成功输出会沉淀为 `data_comprehensive_diagnosis_report` artifact，整体无有效用户数据时 deferred |

## 搜索 search

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `search_trends` | `search_trends` | adapter_required | `backend/crates/agent/src/skills/search.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | Post 8-8：已形成统一 search adapter 降级契约；Post 8-60：输出补齐 `evidence`、去重来源和 fresh-data/bypass-cache 语义；Post 8-64：已补“联网/实时/最新/搜索趋势”自然触达；真实实时趋势 provider 待配置 |
| `search_competitor` | `search_competitor` | adapter_required | `backend/crates/agent/src/skills/search.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | Post 8-8：已形成统一 search adapter 降级契约；Post 8-60：输出补齐 `evidence`、去重来源和 fresh-data/bypass-cache 语义；Post 8-64：已补“搜竞品公开资料”自然触达；竞品外部来源待配置 |
| `search_market_info` | `search_market_info` | adapter_required | `backend/crates/agent/src/skills/search.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | Post 8-8：已形成统一 search adapter 降级契约；Post 8-60：输出补齐 `evidence`、去重来源和 fresh-data/bypass-cache 语义；Post 8-64：已补“搜索市场报告/公开来源”自然触达；市场新闻/行情 provider 待配置 |
| `search_product_reviews` | `search_product_reviews` | adapter_required | `backend/crates/agent/src/skills/search.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | Post 8-8：已形成统一 search adapter 降级契约；Post 8-60：输出补齐 `evidence`、去重来源和 fresh-data/bypass-cache 语义；Post 8-64：已补“搜用户评价/口碑/差评”自然触达；平台评价来源待配置 |
| `search_platform_policy` | `search_platform_policy` | adapter_required | `backend/crates/agent/src/skills/search.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | Post 8-8：已形成统一 search adapter 降级契约；Post 8-60：输出补齐 `evidence`、去重来源和 fresh-data/bypass-cache 语义；Post 8-64：已补“查平台最新规则/政策”自然触达；平台官方规则/公告来源待配置 |
| `search_industry_benchmarks` | `search_industry_benchmarks` | adapter_required | `backend/crates/agent/src/skills/search.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | Post 8-8：已形成统一 search adapter 降级契约；Post 8-60：输出补齐 `evidence`、去重来源和 fresh-data/bypass-cache 语义；Post 8-64：已补“查行业基准/均值”自然触达；行业基准数据来源待配置 |

Post 8-9 补充：Rust 后端已新增 `/api/app/external-search/status` 和 `/api/app/external-search/search` 作为统一外部搜索 adapter API。

Post 8-10 补充：上述 Agent `search_*` skill 已接入同一份 `[external_search]` 配置。provider 配置完整且返回可信来源时可返回 `live`；未配置、请求失败或无来源时仍返回 `deferred/degraded`，不伪造结果。

Post 8-60 补充：App API 和 Agent `search_*` skill 均已回显 `bypassCache/query.bypass_cache`，兼容旧产品 `_needs_fresh_data` 实时性意图；live/degraded 输出均包含 `evidence` 结构，live 来源会去重并生成可供前端来源面板和 Agent 复用的证据摘要。

## 平台执行 / 审批治理

| 旧能力 | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `platform_update_price` | `platform_update_price` | approval_required | `backend/crates/service/src/api/platform_execution.rs` / `backend/crates/service/src/api/approval_audit.rs` / `backend/crates/service/src/api/platform_connect.rs` | Post 8-25：审批通过后进入执行队列并写真实失败结果；Post 8-44：审批创建时已有 readiness 诊断；Post 8-45：平台连接页已有只读 adapter 能力目录；Post 8-46：创建审批前已有 preflight 预检；Post 8-47：preflight、审批 readiness 和执行失败详情共用同一后端规则模块；Post 8-48：动作展示、参数、幂等字段和 adapter 文案已进入共享动作规格注册表；Post 8-49：审批后失败结果会记录 adapter request envelope 和幂等键；Post 8-50：失败结果已纳入 adapter outcome 契约；Post 8-51：审批后执行规划已调用统一 dispatch outcome。真实平台 adapter 未实现时返回 `adapter_not_implemented`，不会伪造成成功 |
| `platform_update_inventory` | `platform_update_inventory` | approval_required | `backend/crates/service/src/api/platform_execution.rs` / `backend/crates/service/src/api/approval_audit.rs` / `backend/crates/service/src/api/platform_connect.rs` | Post 8-25：审批通过后进入执行队列并写真实失败结果；Post 8-44：审批创建时已有 readiness 诊断；Post 8-45：平台连接页已有只读 adapter 能力目录；Post 8-46：创建审批前已有 preflight 预检；Post 8-47：缺连接、禁用连接、缺参数或 adapter 未实现都由共享规则统一判断；Post 8-48：动作规格已统一到共享注册表；Post 8-49：审批后失败结果会记录规范化 adapter request envelope；Post 8-50：已具备统一 adapter outcome；Post 8-51：已具备统一 dispatch outcome |
| `platform_sync_products` | `platform_sync_products` | approval_required | `backend/crates/service/src/api/platform_execution.rs` / `backend/crates/service/src/api/approval_audit.rs` / `backend/crates/service/src/api/platform_connect.rs` | Post 8-44：已纳入平台执行 readiness 诊断；Post 8-45：已纳入平台执行 adapter 能力目录；Post 8-46：已纳入创建审批前 preflight 预检；Post 8-47：已纳入共享规则内核；Post 8-48：已纳入共享动作规格注册表；Post 8-49：已具备 adapter request envelope；Post 8-50：已具备 adapter outcome 契约；Post 8-51：审批后执行调度已集中到平台执行内核；真实商品同步 adapter 仍待实现 |

## 附件 / 素材 / 上下文 ingestion

| 旧能力 | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `conversation_file_context` | `workspace_context_pack` | native_ready | `backend/crates/service/src/api/workspace.rs` | Post 8-14：工作区内容包已读取数据库已有素材正文、资产描述、交付物正文和附件抽取文本；Post 8-43：附件解析状态新增 `pending/extracted/failed/unsupported`，只有 `extracted` 会进入 Agent 可读上下文 |
| `conversation_file_extraction` | `conversation_file_extraction_status` | adapter_required | `backend/crates/service/src/api/conversation.rs` | Post 8-16：已有可信抽取文本登记 API；Post 8-43：已补状态契约。真实 OCR、PDF/Office 解析、图片识别、object storage 文件读取和后台 parser worker 仍待 adapter |

## 客服 service

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `service_ticket_handler` | `service_ticket_handler` | native_ready | `backend/crates/agent/src/skills/service.rs` | Post 8-7：已迁为 native 工单处理、优先级、SLA 和话术包；Post 8-33：成功输出会沉淀为 `service_ticket_playbook` artifact；Post 8-73：支持 `product_ref=product:{uuid}` 记录本轮显式选品关联，未选商品时不注入空引用 |
| `service_faq_playbook` | `service_faq_playbook` | native_ready | `backend/crates/agent/src/skills/service.rs` | Post 8-7：已迁为 native FAQ 话术库和处理指引；Post 8-33：成功输出会沉淀为 `service_faq_playbook` artifact；Post 8-73：支持 `product_ref=product:{uuid}` 作为 FAQ 适用商品范围记录 |
| `service_escalation_flow` | `service_escalation_flow` | native_ready | `backend/crates/agent/src/skills/service.rs` | Post 8-7：已迁为 native 升级路径、主管话术和复核模板；Post 8-33：成功输出会沉淀为 `service_escalation_playbook` artifact；Post 8-73：支持 `product_ref=product:{uuid}` 记录本轮显式选品关联 |
| `service_dsr_improvement` | `service_dsr_improvement` | native_ready | `backend/crates/agent/src/skills/service.rs` | Post 8-7：已补强 native 改善方案；已纳入岗位自动补偿规则；Post 8-33：成功输出会沉淀为 `service_dsr_improvement_plan` artifact |
| `service_return_handler` | `service_return_handler` | native_ready | `backend/crates/agent/src/skills/service.rs` | 已纳入岗位自动补偿规则；Post 8-34：成功输出会沉淀为 `service_return_plan` artifact；Post 8-73：支持 `product_ref=product:{uuid}` 记录本轮显式选品关联 |
| `service_query_product` | `service_query_product` | native_ready | `backend/crates/agent/src/skills/service.rs` | Post 8-56：从旧产品客服商品查询迁为新产品 Agent native skill；只读取本轮显式注入的商品上下文 metadata，不在 agent crate 直连数据库，不默认注入商品；无商品上下文或筛选不命中时返回 deferred。Post 8-72：支持 `product_ref=product:{uuid}` 精确匹配显式选品，客服岗位“查询商品 / 商品信息 / 商品资料 / 引用商品”自然问法可触发该 skill，未选商品时不留下空引用字段 |
| `service_nps_analyzer` | `service_nps_analyzer` | native_ready | `backend/crates/agent/src/skills/service.rs` | 已可 native 执行并可由满意度/NPS自然问法触发；Post 8-34：成功输出会沉淀为 `service_nps_report` artifact |
| `service_sentiment_analyzer` | `service_sentiment_analyzer` | native_ready | `backend/crates/agent/src/skills/service.rs` | 已可 native 执行并可由差评/情绪/投诉自然问法触发；Post 8-34：成功输出会沉淀为 `service_sentiment_report` artifact |
| `service_nps_driver_analysis` | `service_nps_driver_analysis` | native_ready | `backend/crates/agent/src/skills/service.rs` | 已补强满意度驱动因素/投诉分类/NPS拖累自然问法触达；Post 8-34：成功输出会沉淀为 `service_nps_driver_report` artifact |

## 设计 design

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `design_main_image` | `design_main_image` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已纳入岗位自动补偿规则；Post 8-41：成功输出会沉淀为 `design_main_image_brief` artifact，缺商品上下文时明示 `calculated_from_defaults`；Post 8-70：显式选品时自动补偿 input 使用选中商品和平台；Post 8-71：支持 `product_ref=product:{uuid}`；当前不生成图片文件 |
| `design_detail_page` | `design_detail_page` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已迁为 Rust native 详情页模块方案；Post 8-41：成功输出会沉淀为 `design_detail_page_brief` artifact，缺商品上下文时明示 `calculated_from_defaults`；Post 8-70：显式选品时自动补偿 input 使用选中商品和平台；Post 8-71：支持 `product_ref=product:{uuid}`；尚不生成长图文件 |
| `design_color_scheme` | `design_color_scheme` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已迁为 Rust native HEX 配色、比例和变量命名；Post 8-41：成功输出会沉淀为 `design_color_scheme` artifact，缺品牌调性时 validation error；Post 8-71：支持 `product_ref=product:{uuid}` |
| `design_material_spec` | `design_material_spec` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已纳入岗位自动补偿规则；Post 8-41：成功输出会沉淀为 `design_material_spec` artifact，缺平台/商品上下文时明示 `calculated_from_defaults`；Post 8-71：支持 `product_ref=product:{uuid}` |
| `design_campaign_poster` | `design_campaign_poster` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已迁为 Rust native 活动海报 brief；Post 8-41：成功输出会沉淀为 `design_campaign_poster_brief` artifact，缺活动名时 validation error；Post 8-71：支持 `product_ref=product:{uuid}`；尚不生成图片文件 |

## 财务 accounting

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `accounting_cost_calc` | `accounting_cost_calc` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则；Post 8-32：成功输出会沉淀为 `accounting_cost_report` artifact |
| `accounting_profit_analysis` | `accounting_profit_analysis` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则；Post 8-32：成功输出会沉淀为 `accounting_profit_report` artifact，缺少 revenue 时 deferred |
| `accounting_budget_plan` | `accounting_budget_plan` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则；Post 8-32：成功输出会沉淀为 `accounting_budget_plan` artifact，使用默认参数时明示 `calculated_from_defaults` |
| `accounting_roi_calc` | `accounting_roi_calc` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则；Post 8-32：成功输出会沉淀为 `accounting_roi_report` artifact，缺少投入产出时 deferred |
| `accounting_break_even_calc` | `accounting_break_even_calc` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | Post 8-57：补齐统一输出契约；成功输出会沉淀为 `accounting_break_even_report` artifact，贡献利润为负时 deferred |
| `accounting_compliance_check` | `accounting_compliance_check` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已补强税务/开票/纳税/现金收款等自然问法触达；Post 8-57：成功输出会沉淀为 `accounting_compliance_report` artifact，缺少 monthly_revenue 时 deferred |
| `accounting_pl_statement` | `accounting_pl_statement` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | Post 8-57：成功输出会沉淀为 `accounting_pl_statement` artifact，缺少 gmv 时 deferred |
| `accounting_budget_vs_actual` | `accounting_budget_vs_actual` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | Post 8-57：成功输出会沉淀为 `accounting_budget_vs_actual_report` artifact，缺少 actual_gmv 时 deferred |
| `accounting_gmv_waterfall` | `accounting_gmv_waterfall` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | Post 8-57：成功输出会沉淀为 `accounting_gmv_waterfall_report` artifact，缺少 gmv 时 deferred |
| `accounting_scenario_analysis` | `accounting_scenario_analysis` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | Post 8-57：成功输出会沉淀为 `accounting_scenario_report` artifact，缺少 base_gmv 时 deferred |
| `accounting_cash_flow_forecast` | `accounting_cash_flow_forecast` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | Post 8-57：成功输出会沉淀为 `accounting_cash_flow_forecast_report` artifact，缺少输入时以默认假设明示 `calculated_from_defaults` |
| `accounting_financial_narrative` | `accounting_financial_narrative` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | Post 8-57：成功输出会沉淀为 `accounting_financial_narrative_report` artifact，缺少输入时以默认样例明示 `calculated_from_defaults` |

## 工程 engineering

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `engineering_arch_review` | `engineering_arch_review` | native_ready | `backend/crates/agent/src/skills/engineering.rs` | Post 8-42：已迁为 Rust native 架构评估报告；成功输出会沉淀为 `engineering_arch_review_report` artifact；真实代码仓库/日志/APM 读取仍待 adapter |
| `engineering_bug_analysis` | `engineering_bug_analysis` | native_ready | `backend/crates/agent/src/skills/engineering.rs` | Post 8-42：已迁为 Rust native Bug 排查方案；成功输出会沉淀为 `engineering_bug_analysis_report` artifact；真实日志和 trace 读取仍待 adapter |
| `engineering_perf_optimize` | `engineering_perf_optimize` | native_ready | `backend/crates/agent/src/skills/engineering.rs` | Post 8-42：已迁为 Rust native 性能优化方案；成功输出会沉淀为 `engineering_perf_plan` artifact；真实压测/APM 指标读取仍待 adapter |
| `engineering_tech_selection` | `engineering_tech_selection` | native_ready | `backend/crates/agent/src/skills/engineering.rs` | Post 8-42：已迁为 Rust native 技术选型报告；成功输出会沉淀为 `engineering_tech_selection_report` artifact；实时 benchmark / 外部资料仍待 search adapter |
| `engineering_system_check` | `engineering_system_check` | approval_required | `backend/crates/agent/src/skills/engineering.rs` / `backend/crates/agent/src/agent/worker/runner.rs` | Post 8-69：已补平台 API 连通性 / 接口健康 / 授权 / 同步状态自然触达；真实测试连接时返回 `pending_approval`、`execution_mode=adapter`、`risk_level=high`，不伪造平台健康检查结果；真实平台配置读取和官方 adapter 仍待实现 |
| `engineering_sla_monitor` | `engineering_sla_monitor` | native_ready | `backend/crates/agent/src/skills/engineering.rs` | 已纳入岗位自动补偿规则；Post 8-42：补齐统一输出契约并沉淀为 `engineering_sla_report` artifact，缺监控输入时明示 `calculated_from_defaults` |

## 网站增长 web

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `web_seo_optimize` | `web_seo_optimize` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native SEO 优化方案；Post 8-39：成功输出会沉淀为 `web_seo_plan` artifact；Post 8-71：支持 `product_ref=product:{uuid}`；实时搜索量仍待 adapter |
| `web_title_generator` | `web_title_generator` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native 多标题候选，并纳入自动补偿；Post 8-39：成功输出会沉淀为 `web_title_candidates` artifact；Post 8-70：显式选品时自动补偿 input 使用选中商品和平台；Post 8-71：支持 `product_ref=product:{uuid}` |
| `web_page_conversion` | `web_page_conversion` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已纳入岗位自动补偿规则；Post 8-39：成功输出会沉淀为 `web_conversion_plan` artifact，使用基准默认值时明示 `calculated_from_defaults` |
| `web_store_design` | `web_store_design` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native 店铺装修方案；Post 8-39：成功输出会沉淀为 `web_store_design_brief` artifact；真实店铺指标仍待 adapter |
| `web_keyword_research` | `web_keyword_research` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native 关键词矩阵；Post 8-39：成功输出会沉淀为 `web_keyword_matrix` artifact；Post 8-70：显式选品时以选中商品作为 seed keyword；Post 8-71：支持 `product_ref=product:{uuid}`；实时热度/CPC仍待 adapter |
| `web_product_description` | `web_product_description` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native 详情页文案；Post 8-39：成功输出会沉淀为 `web_product_description` artifact，缺商品名时明示默认主推商品；Post 8-70：显式选品时自动补偿 input 使用选中商品和平台，未选商品时不从历史摘要注入默认商品；Post 8-71：支持 `product_ref=product:{uuid}` |
| `web_title_seo_scorer` | `web_title_seo_scorer` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native 标题 SEO 评分；Post 8-39：成功输出会沉淀为 `web_title_seo_score` artifact，缺标题时 validation error |

## 创意 creative

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `creative_video_script` | `creative_video_script` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 结构化脚本产物，并纳入创意岗位自动补偿；Post 8-40：成功输出会沉淀为 `video_script` artifact；Post 8-70：显式选品时自动补偿 input 使用选中商品和平台，未选商品时不从历史摘要注入默认商品；Post 8-71：支持 `product_ref=product:{uuid}` |
| `creative_seeding_copy` | `creative_seeding_copy` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 种草文案产物，并纳入创意岗位自动补偿；Post 8-40：成功输出会沉淀为 `seeding_copy` artifact；Post 8-70：显式选品时自动补偿 input 使用选中商品、平台和知识卖点；Post 8-71：支持 `product_ref=product:{uuid}` |
| `creative_ip_branding` | `creative_ip_branding` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native IP 定位方案，并纳入创意岗位自动补偿；Post 8-40：成功输出会沉淀为 `ip_branding` artifact，缺品牌/类目时 validation error |
| `creative_content_calendar` | `creative_content_calendar` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 内容排期产物，并纳入创意岗位自动补偿；Post 8-40：成功输出会沉淀为 `content_calendar` artifact，默认排期会明示 `calculated_from_defaults` |
| `creative_trend_catch` | `creative_trend_catch` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 趋势方向产物；Post 8-40：成功输出会沉淀为 `creative_trend_report` artifact；实时外部搜索仍待后续 adapter |
| `creative_live_script` | `creative_live_script` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 直播话术产物，并纳入岗位自动补偿规则；Post 8-40：成功输出会沉淀为 `live_script` artifact，默认直播参数会明示 `calculated_from_defaults`；Post 8-70：显式选品时 product_count 和平台来自选中商品上下文；Post 8-71：支持 `product_refs=[product:{uuid}]` |
| `creative_product_article` | `creative_product_article` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 产品软文产物，并纳入创意岗位自动补偿；Post 8-40：成功输出会沉淀为 `product_article` artifact；Post 8-70：显式选品时自动补偿 input 使用选中商品和平台；Post 8-71：支持 `product_ref=product:{uuid}` |
| `creative_title_ctr_scorer` | `creative_title_ctr_scorer` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已纳入岗位自动补偿规则；Post 8-40：成功输出会沉淀为 `creative_title_ctr_score` artifact，缺标题时 validation error |

## 协同 coordination

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `coordination_agent_handoff` | `coordination_agent_handoff` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 已可 native 输出角色交接建议 |
| `coordination_task_orchestration` | `coordination_task_orchestration` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 已可 native 输出多角色任务编排骨架 |
| `coordination_status_query` | `coordination_status_query` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 本轮已真实化为 session/task/workspace 可见状态查询，并补强 chat/engineering 自然问法触达 |
| `coordination_create_campaign` | `coordination_create_campaign` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 本轮已真实化为营销活动草稿 skill，并通过 SSE/service 落成真实 campaign draft 与 campaign_draft artifact；Post 8-74：支持 `product_ref=product:{uuid}` 作为新产品商品引用，并继续回填旧 `product_id` 兼容现有草稿持久化链路 |
| `coordination_create_task` | `coordination_create_task` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 本轮已真实化为工作区任务草案 skill，并通过 SSE/service 落成真实 workspace task |
| `coordination_save_memory` | `coordination_save_memory` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 本轮已真实化为工作记忆草案 skill，并通过 SSE/service 落成真实 agent learning record / workspace memory |
| `coordination_skill_chain_planner` | `coordination_skill_chain_planner` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | Post 8-61：已补强为 Rust native 结构化技能链规划，输出阶段、负责角色、执行模式、风险、前置依赖、不可用技能、adapter提示和审批提示；只做规划，不执行外部 Agent 或平台写操作 |
| `coordination_external_agent_call` | `coordination_external_agent_call` | approval_required | `backend/crates/agent/src/skills/coordination.rs` | 保持 pending approval；真实外部 Agent/A2A 调用需要白名单、审计和安全策略后再接，不在当前阶段伪造执行 |
