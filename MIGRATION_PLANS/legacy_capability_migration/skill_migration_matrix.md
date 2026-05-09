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
| `ops_promo_planning` | `ops_promo_planning` | pending_review | `backend/crates/agent/src/skills/ops.rs` | 活动方案，后续可联动 campaign |
| `ops_pricing_strategy` | `ops_pricing_strategy` | pending_review | `backend/crates/agent/src/skills/ops.rs` | 优先核对 native |
| `ops_channel_strategy` | `ops_channel_strategy` | pending_review | `backend/crates/agent/src/skills/ops.rs` | 需平台/人群上下文 |
| `ops_inventory_planning` | `ops_inventory_planning` | pending_review | `backend/crates/agent/src/skills/ops.rs` | 后续接库存/商品数据 |
| `ops_listing_copy` | `ops_listing_copy` | pending_review | `backend/crates/agent/src/skills/ops.rs` | 可和 web/creative 协同 |
| `ops_assortment_planning` | `ops_assortment_planning` | pending_review | `backend/crates/agent/src/skills/ops.rs` | 选品组合 |
| `ops_execution_plan` | `ops_execution_plan` | pending_review | `backend/crates/agent/src/skills/ops.rs` | 工作区任务沉淀候选 |

## 数据 data

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `data_funnel_analysis` | `data_funnel_analysis` | pending_review | `backend/crates/agent/src/skills/data.rs` | 第一批代表 skill |
| `data_anomaly_diagnosis` | `data_anomaly_diagnosis` | pending_review | `backend/crates/agent/src/skills/data.rs` | 指标序列异常 |
| `data_trend_forecast` | `data_trend_forecast` | pending_review | `backend/crates/agent/src/skills/data.rs` | 趋势预测 |
| `data_customer_segmentation` | `data_customer_segmentation` | pending_review | `backend/crates/agent/src/skills/data.rs` | RFM 分层 |
| `data_competitor_analysis` | `data_competitor_analysis` | pending_review | `backend/crates/agent/src/skills/data.rs` | 接 product_competitor |
| `data_dashboard` | `data_dashboard` | pending_review | `backend/crates/agent/src/skills/data.rs` | 数据源需明确 |

## 客服 service

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `service_ticket_handler` | `service_ticket_handler` | pending_review | `backend/crates/agent/src/skills/service.rs` | 工单处理 |
| `service_faq_playbook` | `service_faq_playbook` | pending_review | `backend/crates/agent/src/skills/service.rs` | FAQ 话术 |
| `service_escalation_flow` | `service_escalation_flow` | pending_review | `backend/crates/agent/src/skills/service.rs` | 升级流程 |
| `service_dsr_improvement` | `service_dsr_improvement` | pending_review | `backend/crates/agent/src/skills/service.rs` | DSR 提升 |
| `service_return_handler` | `service_return_handler` | pending_review | `backend/crates/agent/src/skills/service.rs` | 第一批代表 skill |

## 设计 design

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `design_main_image` | `design_main_image` | pending_review | `backend/crates/agent/src/skills/design.rs` | 主图方案 |
| `design_detail_page` | `design_detail_page` | pending_review | `backend/crates/agent/src/skills/design.rs` | 详情页结构 |
| `design_color_scheme` | `design_color_scheme` | pending_review | `backend/crates/agent/src/skills/design.rs` | 配色 |
| `design_material_spec` | `design_material_spec` | pending_review | `backend/crates/agent/src/skills/design.rs` | 平台素材规范 |
| `design_campaign_poster` | `design_campaign_poster` | pending_review | `backend/crates/agent/src/skills/design.rs` | 活动海报 |

## 财务 accounting

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `accounting_cost_calc` | `accounting_cost_calc` | pending_review | `backend/crates/agent/src/skills/accounting.rs` | 成本核算 |
| `accounting_profit_analysis` | `accounting_profit_analysis` | pending_review | `backend/crates/agent/src/skills/accounting.rs` | 利润分析 |
| `accounting_budget_plan` | `accounting_budget_plan` | pending_review | `backend/crates/agent/src/skills/accounting.rs` | 预算 |
| `accounting_roi_calc` | `accounting_roi_calc` | pending_review | `backend/crates/agent/src/skills/accounting.rs` | 第一批代表 skill |
| `accounting_compliance_check` | `accounting_compliance_check` | pending_review | `backend/crates/agent/src/skills/accounting.rs` | 合规 |

## 工程 engineering

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `engineering_arch_review` | `engineering_arch_review` | pending_review | `backend/crates/agent/src/skills/engineering.rs` | 架构评估 |
| `engineering_bug_analysis` | `engineering_bug_analysis` | pending_review | `backend/crates/agent/src/skills/engineering.rs` | 排障 |
| `engineering_perf_optimize` | `engineering_perf_optimize` | pending_review | `backend/crates/agent/src/skills/engineering.rs` | 性能 |
| `engineering_tech_selection` | `engineering_tech_selection` | pending_review | `backend/crates/agent/src/skills/engineering.rs` | 技术选型 |

## 网站增长 web

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `web_seo_optimize` | `web_seo_optimize` | pending_review | `backend/crates/agent/src/skills/web.rs` | SEO |
| `web_title_generator` | `web_title_generator` | pending_review | `backend/crates/agent/src/skills/web.rs` | 第一批代表 skill |
| `web_page_conversion` | `web_page_conversion` | pending_review | `backend/crates/agent/src/skills/web.rs` | 页面转化 |
| `web_store_design` | `web_store_design` | pending_review | `backend/crates/agent/src/skills/web.rs` | 店铺装修 |
| `web_keyword_research` | `web_keyword_research` | pending_review | `backend/crates/agent/src/skills/web.rs` | 关键词 |

## 创意 creative

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `creative_video_script` | `creative_video_script` | pending_review | `backend/crates/agent/src/skills/creative.rs` | 短视频脚本 |
| `creative_seeding_copy` | `creative_seeding_copy` | pending_review | `backend/crates/agent/src/skills/creative.rs` | 种草文案 |
| `creative_ip_branding` | `creative_ip_branding` | pending_review | `backend/crates/agent/src/skills/creative.rs` | IP 定位 |
| `creative_content_calendar` | `creative_content_calendar` | pending_review | `backend/crates/agent/src/skills/creative.rs` | 内容排期 |
| `creative_trend_catch` | `creative_trend_catch` | pending_review | `backend/crates/agent/src/skills/creative.rs` | 趋势，可能需要 search |
| `creative_live_script` | `creative_live_script` | pending_review | `backend/crates/agent/src/skills/creative.rs` | 第一批代表 skill |

## 协同 coordination

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `coordination_agent_handoff` | `coordination_agent_handoff` | pending_review | `backend/crates/agent/src/skills/coordination.rs` | 交接 |
| `coordination_task_orchestration` | `coordination_task_orchestration` | pending_review | `backend/crates/agent/src/skills/coordination.rs` | 任务编排 |
| `coordination_status_query` | `coordination_status_query` | pending_review | `backend/crates/agent/src/skills/coordination.rs` | 状态查询，需真实化 |
