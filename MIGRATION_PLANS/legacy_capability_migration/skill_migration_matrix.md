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
| `ops_listing_copy` | `ops_listing_copy` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已补强上架文案/标题卖点/上新文案自然问法触达；Post 8-35：成功输出会沉淀为 `ops_listing_copy` artifact，缺商品上下文时 deferred |
| `ops_assortment_planning` | `ops_assortment_planning` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已补强货盘/SKU组合/选品组合自然问法触达；Post 8-35：成功输出会沉淀为 `ops_assortment_plan` artifact |
| `ops_execution_plan` | `ops_execution_plan` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已纳入岗位自动补偿规则，并有单测覆盖；Post 8-35：成功输出会沉淀为 `ops_execution_plan` artifact |
| `ops_smart_pricing` | `ops_smart_pricing` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已可 native 执行；Post 8-36：成功输出会沉淀为 `ops_smart_pricing_report` artifact，缺竞品价格时明示 `calculated_from_defaults` |
| `ops_inventory_optimizer` | `ops_inventory_optimizer` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已可 native 执行；Post 8-36：成功输出会沉淀为 `ops_inventory_optimization_report` artifact，缺 SKU 明细时 deferred |
| `ops_ad_fatigue_detector` | `ops_ad_fatigue_detector` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已可 native 执行；Post 8-36：成功输出会沉淀为 `ops_ad_fatigue_report` artifact，缺渠道数据时 deferred |
| `ops_abc_xyz_classifier` | `ops_abc_xyz_classifier` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已可 native 执行；Post 8-36：成功输出会沉淀为 `ops_abc_xyz_report` artifact，使用确定性演示 SKU 时明示 `calculated_from_defaults` |

## 数据 data

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `data_funnel_analysis` | `data_funnel_analysis` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已纳入岗位自动补偿规则，并有 stream 级测试覆盖；Post 8-31：有用户数据时成功输出会沉淀为 `data_funnel_report` artifact |
| `data_anomaly_diagnosis` | `data_anomaly_diagnosis` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已纳入岗位自动补偿规则；Post 8-37：成功输出会沉淀为 `data_anomaly_report` artifact，时序点不足时 deferred |
| `data_trend_forecast` | `data_trend_forecast` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已纳入岗位自动补偿规则；Post 8-37：成功输出会沉淀为 `data_trend_forecast_report` artifact，时序点不足时 deferred |
| `data_customer_segmentation` | `data_customer_segmentation` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已纳入岗位自动补偿规则；Post 8-37：成功输出会沉淀为 `data_customer_segmentation_report` artifact，使用默认估算时明示 `calculated_from_defaults` |
| `data_competitor_analysis` | `data_competitor_analysis` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已补强竞品/对标/竞对自然问法触达；Post 8-37：成功输出会沉淀为 `data_competitor_report` artifact，缺竞品列表时明示 `calculated_from_defaults` |
| `data_dashboard` | `data_dashboard` | native_ready | `backend/crates/agent/src/skills/data.rs` | Post 8-29：已迁为 Rust native 经营看板；有入参时按真实入参计算，无入参时明确 `store_metrics_required`，不伪造平台实时数据。Post 8-30：成功输出会沉淀为 `data_dashboard` workspace artifact |
| `data_ab_test_analyzer` | `data_ab_test_analyzer` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已可 native 执行；Post 8-37：成功输出会沉淀为 `data_ab_test_report` artifact，样本/转化数不合法时返回 validation error |
| `data_multi_period_trend` | `data_multi_period_trend` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_multi_period_trend_report` artifact，时序点不足时 deferred |
| `data_ltv_calculator` | `data_ltv_calculator` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_ltv_report` artifact，使用默认估算时明示 `calculated_from_defaults` |
| `data_cohort_analysis` | `data_cohort_analysis` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_cohort_report` artifact，使用默认新客数时明示 `calculated_from_defaults` |
| `data_seasonal_decompose` | `data_seasonal_decompose` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_seasonal_decompose_report` artifact，时序点不足时 deferred |
| `data_price_elasticity` | `data_price_elasticity` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_price_elasticity_report` artifact，使用默认价格点时明示 `calculated_from_defaults` |
| `data_demand_forecast` | `data_demand_forecast` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已可 native 执行；Post 8-38：成功输出会沉淀为 `data_demand_forecast_report` artifact，时序点不足时 deferred |

## 搜索 search

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `search_trends` | `search_trends` | adapter_required | `backend/crates/agent/src/skills/search.rs` | Post 8-8：已形成统一 search adapter 降级契约；真实实时趋势 provider 待接入 |
| `search_competitor` | `search_competitor` | adapter_required | `backend/crates/agent/src/skills/search.rs` | Post 8-8：已形成统一 search adapter 降级契约；竞品外部来源待接入 |
| `search_market_info` | `search_market_info` | adapter_required | `backend/crates/agent/src/skills/search.rs` | Post 8-8：已形成统一 search adapter 降级契约；市场新闻/行情 provider 待接入 |
| `search_product_reviews` | `search_product_reviews` | adapter_required | `backend/crates/agent/src/skills/search.rs` | Post 8-8：已形成统一 search adapter 降级契约；平台评价来源待接入 |
| `search_platform_policy` | `search_platform_policy` | adapter_required | `backend/crates/agent/src/skills/search.rs` | Post 8-8：已形成统一 search adapter 降级契约；平台官方规则/公告来源待接入 |
| `search_industry_benchmarks` | `search_industry_benchmarks` | adapter_required | `backend/crates/agent/src/skills/search.rs` | Post 8-8：已形成统一 search adapter 降级契约；行业基准数据来源待接入 |

Post 8-9 补充：Rust 后端已新增 `/api/app/external-search/status` 和 `/api/app/external-search/search` 作为统一外部搜索 adapter API。

Post 8-10 补充：上述 Agent `search_*` skill 已接入同一份 `[external_search]` 配置。provider 配置完整且返回可信来源时可返回 `live`；未配置、请求失败或无来源时仍返回 `deferred/degraded`，不伪造结果。

## 客服 service

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `service_ticket_handler` | `service_ticket_handler` | native_ready | `backend/crates/agent/src/skills/service.rs` | Post 8-7：已迁为 native 工单处理、优先级、SLA 和话术包；Post 8-33：成功输出会沉淀为 `service_ticket_playbook` artifact |
| `service_faq_playbook` | `service_faq_playbook` | native_ready | `backend/crates/agent/src/skills/service.rs` | Post 8-7：已迁为 native FAQ 话术库和处理指引；Post 8-33：成功输出会沉淀为 `service_faq_playbook` artifact |
| `service_escalation_flow` | `service_escalation_flow` | native_ready | `backend/crates/agent/src/skills/service.rs` | Post 8-7：已迁为 native 升级路径、主管话术和复核模板；Post 8-33：成功输出会沉淀为 `service_escalation_playbook` artifact |
| `service_dsr_improvement` | `service_dsr_improvement` | native_ready | `backend/crates/agent/src/skills/service.rs` | Post 8-7：已补强 native 改善方案；已纳入岗位自动补偿规则；Post 8-33：成功输出会沉淀为 `service_dsr_improvement_plan` artifact |
| `service_return_handler` | `service_return_handler` | native_ready | `backend/crates/agent/src/skills/service.rs` | 已纳入岗位自动补偿规则；Post 8-34：成功输出会沉淀为 `service_return_plan` artifact |
| `service_nps_analyzer` | `service_nps_analyzer` | native_ready | `backend/crates/agent/src/skills/service.rs` | 已可 native 执行并可由满意度/NPS自然问法触发；Post 8-34：成功输出会沉淀为 `service_nps_report` artifact |
| `service_sentiment_analyzer` | `service_sentiment_analyzer` | native_ready | `backend/crates/agent/src/skills/service.rs` | 已可 native 执行并可由差评/情绪/投诉自然问法触发；Post 8-34：成功输出会沉淀为 `service_sentiment_report` artifact |
| `service_nps_driver_analysis` | `service_nps_driver_analysis` | native_ready | `backend/crates/agent/src/skills/service.rs` | 已补强满意度驱动因素/投诉分类/NPS拖累自然问法触达；Post 8-34：成功输出会沉淀为 `service_nps_driver_report` artifact |

## 设计 design

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `design_main_image` | `design_main_image` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已纳入岗位自动补偿规则，成功输出会沉淀为 `design_main_image_brief` artifact；当前不生成图片文件 |
| `design_detail_page` | `design_detail_page` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已迁为 Rust native 详情页模块方案，成功输出会沉淀为 `design_detail_page_brief` artifact；尚不生成长图文件 |
| `design_color_scheme` | `design_color_scheme` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已迁为 Rust native HEX 配色、比例和变量命名，成功输出会沉淀为 `design_color_scheme` artifact |
| `design_material_spec` | `design_material_spec` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已纳入岗位自动补偿规则，成功输出会沉淀为 `design_material_spec` artifact |
| `design_campaign_poster` | `design_campaign_poster` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已迁为 Rust native 活动海报 brief，成功输出会沉淀为 `design_campaign_poster_brief` artifact；尚不生成图片文件 |

## 财务 accounting

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `accounting_cost_calc` | `accounting_cost_calc` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则；Post 8-32：成功输出会沉淀为 `accounting_cost_report` artifact |
| `accounting_profit_analysis` | `accounting_profit_analysis` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则；Post 8-32：成功输出会沉淀为 `accounting_profit_report` artifact，缺少 revenue 时 deferred |
| `accounting_budget_plan` | `accounting_budget_plan` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则；Post 8-32：成功输出会沉淀为 `accounting_budget_plan` artifact，使用默认参数时明示 `calculated_from_defaults` |
| `accounting_roi_calc` | `accounting_roi_calc` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则；Post 8-32：成功输出会沉淀为 `accounting_roi_report` artifact，缺少投入产出时 deferred |
| `accounting_compliance_check` | `accounting_compliance_check` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 本轮补强了税务/开票/纳税/现金收款等自然问法触达 |

## 工程 engineering

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `engineering_arch_review` | `engineering_arch_review` | adapter_required | `backend/crates/agent/src/skills/engineering.rs` | 当前为 deferred 工程分析骨架；本轮补强了架构/重构/扩展性自然问法触达 |
| `engineering_bug_analysis` | `engineering_bug_analysis` | adapter_required | `backend/crates/agent/src/skills/engineering.rs` | 当前为 deferred 工程分析骨架；本轮补强了故障/报错/排查自然问法触达 |
| `engineering_perf_optimize` | `engineering_perf_optimize` | adapter_required | `backend/crates/agent/src/skills/engineering.rs` | 当前为 deferred 工程分析骨架；本轮补强了性能/延迟/QPS自然问法触达 |
| `engineering_tech_selection` | `engineering_tech_selection` | adapter_required | `backend/crates/agent/src/skills/engineering.rs` | 当前为 deferred 工程分析骨架；本轮补强了技术选型/框架选择自然问法触达 |
| `engineering_sla_monitor` | `engineering_sla_monitor` | native_ready | `backend/crates/agent/src/skills/engineering.rs` | 已纳入岗位自动补偿规则 |

## 网站增长 web

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `web_seo_optimize` | `web_seo_optimize` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native SEO 优化方案；Post 8-39：成功输出会沉淀为 `web_seo_plan` artifact，实时搜索量仍待 adapter |
| `web_title_generator` | `web_title_generator` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native 多标题候选，并纳入自动补偿；Post 8-39：成功输出会沉淀为 `web_title_candidates` artifact |
| `web_page_conversion` | `web_page_conversion` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已纳入岗位自动补偿规则；Post 8-39：成功输出会沉淀为 `web_conversion_plan` artifact，使用基准默认值时明示 `calculated_from_defaults` |
| `web_store_design` | `web_store_design` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native 店铺装修方案；Post 8-39：成功输出会沉淀为 `web_store_design_brief` artifact；真实店铺指标仍待 adapter |
| `web_keyword_research` | `web_keyword_research` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native 关键词矩阵；Post 8-39：成功输出会沉淀为 `web_keyword_matrix` artifact；实时热度/CPC仍待 adapter |
| `web_product_description` | `web_product_description` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native 详情页文案；Post 8-39：成功输出会沉淀为 `web_product_description` artifact，缺商品名时明示默认主推商品 |
| `web_title_seo_scorer` | `web_title_seo_scorer` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已迁为 Rust native 标题 SEO 评分；Post 8-39：成功输出会沉淀为 `web_title_seo_score` artifact，缺标题时 validation error |

## 创意 creative

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `creative_video_script` | `creative_video_script` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 结构化脚本产物，并纳入创意岗位自动补偿 |
| `creative_seeding_copy` | `creative_seeding_copy` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 种草文案产物，并纳入创意岗位自动补偿 |
| `creative_ip_branding` | `creative_ip_branding` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native IP 定位方案，并纳入创意岗位自动补偿 |
| `creative_content_calendar` | `creative_content_calendar` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 内容排期产物，并纳入创意岗位自动补偿 |
| `creative_trend_catch` | `creative_trend_catch` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 趋势方向产物；实时外部搜索仍待后续 adapter |
| `creative_live_script` | `creative_live_script` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 直播话术产物，并纳入岗位自动补偿规则 |
| `creative_product_article` | `creative_product_article` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已迁为 Rust native 产品软文产物，并纳入创意岗位自动补偿 |
| `creative_title_ctr_scorer` | `creative_title_ctr_scorer` | native_ready | `backend/crates/agent/src/skills/creative.rs` | 已纳入岗位自动补偿规则 |

## 协同 coordination

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `coordination_agent_handoff` | `coordination_agent_handoff` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 已可 native 输出角色交接建议 |
| `coordination_task_orchestration` | `coordination_task_orchestration` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 已可 native 输出多角色任务编排骨架 |
| `coordination_status_query` | `coordination_status_query` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 本轮已真实化为 session/task/workspace 可见状态查询，并补强 chat/engineering 自然问法触达 |
| `coordination_create_campaign` | `coordination_create_campaign` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 本轮已真实化为营销活动草稿 skill，并通过 SSE/service 落成真实 campaign draft 与 campaign_draft artifact |
| `coordination_create_task` | `coordination_create_task` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 本轮已真实化为工作区任务草案 skill，并通过 SSE/service 落成真实 workspace task |
| `coordination_save_memory` | `coordination_save_memory` | native_ready | `backend/crates/agent/src/skills/coordination.rs` | 本轮已真实化为工作记忆草案 skill，并通过 SSE/service 落成真实 agent learning record / workspace memory |
