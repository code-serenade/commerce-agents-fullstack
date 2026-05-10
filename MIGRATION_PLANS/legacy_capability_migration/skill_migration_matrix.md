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
| `ops_pricing_strategy` | `ops_pricing_strategy` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已纳入岗位自动补偿规则，可由 ops 定价意图触发 |
| `ops_channel_strategy` | `ops_channel_strategy` | pending_review | `backend/crates/agent/src/skills/ops.rs` | 需平台/人群上下文 |
| `ops_inventory_planning` | `ops_inventory_planning` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已纳入岗位自动补偿规则，当前基于输入数字做本地规划 |
| `ops_listing_copy` | `ops_listing_copy` | pending_review | `backend/crates/agent/src/skills/ops.rs` | 可和 web/creative 协同 |
| `ops_assortment_planning` | `ops_assortment_planning` | pending_review | `backend/crates/agent/src/skills/ops.rs` | 选品组合 |
| `ops_execution_plan` | `ops_execution_plan` | native_ready | `backend/crates/agent/src/skills/ops.rs` | 已纳入岗位自动补偿规则，并有单测覆盖 |

## 数据 data

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `data_funnel_analysis` | `data_funnel_analysis` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已纳入岗位自动补偿规则，并有 stream 级测试覆盖 |
| `data_anomaly_diagnosis` | `data_anomaly_diagnosis` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已纳入岗位自动补偿规则，输入不足时按本地保守诊断 |
| `data_trend_forecast` | `data_trend_forecast` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已纳入岗位自动补偿规则 |
| `data_customer_segmentation` | `data_customer_segmentation` | native_ready | `backend/crates/agent/src/skills/data.rs` | 已纳入岗位自动补偿规则，后续可接真实客户数据 |
| `data_competitor_analysis` | `data_competitor_analysis` | pending_review | `backend/crates/agent/src/skills/data.rs` | 接 product_competitor |
| `data_dashboard` | `data_dashboard` | pending_review | `backend/crates/agent/src/skills/data.rs` | 数据源需明确 |

## 客服 service

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `service_ticket_handler` | `service_ticket_handler` | pending_review | `backend/crates/agent/src/skills/service.rs` | 工单处理 |
| `service_faq_playbook` | `service_faq_playbook` | pending_review | `backend/crates/agent/src/skills/service.rs` | FAQ 话术 |
| `service_escalation_flow` | `service_escalation_flow` | pending_review | `backend/crates/agent/src/skills/service.rs` | 升级流程 |
| `service_dsr_improvement` | `service_dsr_improvement` | native_ready | `backend/crates/agent/src/skills/service.rs` | 已纳入岗位自动补偿规则 |
| `service_return_handler` | `service_return_handler` | native_ready | `backend/crates/agent/src/skills/service.rs` | 已纳入岗位自动补偿规则 |

## 设计 design

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `design_main_image` | `design_main_image` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已纳入岗位自动补偿规则，当前生成主图方案不生成图片文件 |
| `design_detail_page` | `design_detail_page` | pending_review | `backend/crates/agent/src/skills/design.rs` | 详情页结构 |
| `design_color_scheme` | `design_color_scheme` | pending_review | `backend/crates/agent/src/skills/design.rs` | 配色 |
| `design_material_spec` | `design_material_spec` | native_ready | `backend/crates/agent/src/skills/design.rs` | 已纳入岗位自动补偿规则 |
| `design_campaign_poster` | `design_campaign_poster` | pending_review | `backend/crates/agent/src/skills/design.rs` | 活动海报 |

## 财务 accounting

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `accounting_cost_calc` | `accounting_cost_calc` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则 |
| `accounting_profit_analysis` | `accounting_profit_analysis` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则 |
| `accounting_budget_plan` | `accounting_budget_plan` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则 |
| `accounting_roi_calc` | `accounting_roi_calc` | native_ready | `backend/crates/agent/src/skills/accounting.rs` | 已纳入岗位自动补偿规则 |
| `accounting_compliance_check` | `accounting_compliance_check` | pending_review | `backend/crates/agent/src/skills/accounting.rs` | 合规 |

## 工程 engineering

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `engineering_arch_review` | `engineering_arch_review` | pending_review | `backend/crates/agent/src/skills/engineering.rs` | 架构评估 |
| `engineering_bug_analysis` | `engineering_bug_analysis` | pending_review | `backend/crates/agent/src/skills/engineering.rs` | 排障 |
| `engineering_perf_optimize` | `engineering_perf_optimize` | pending_review | `backend/crates/agent/src/skills/engineering.rs` | 性能 |
| `engineering_tech_selection` | `engineering_tech_selection` | pending_review | `backend/crates/agent/src/skills/engineering.rs` | 技术选型 |
| `engineering_sla_monitor` | `engineering_sla_monitor` | native_ready | `backend/crates/agent/src/skills/engineering.rs` | 已纳入岗位自动补偿规则 |

## 网站增长 web

| 旧 Skill | 新产品目标 ID | 状态 | 新落点 | 备注 |
| --- | --- | --- | --- | --- |
| `web_seo_optimize` | `web_seo_optimize` | pending_review | `backend/crates/agent/src/skills/web.rs` | SEO |
| `web_title_generator` | `web_title_generator` | native_ready | `backend/crates/agent/src/skills/web.rs` | Rust native 已存在；本次自动补偿优先走 SEO scorer |
| `web_page_conversion` | `web_page_conversion` | native_ready | `backend/crates/agent/src/skills/web.rs` | 已纳入岗位自动补偿规则 |
| `web_store_design` | `web_store_design` | pending_review | `backend/crates/agent/src/skills/web.rs` | 店铺装修 |
| `web_keyword_research` | `web_keyword_research` | pending_review | `backend/crates/agent/src/skills/web.rs` | 关键词 |

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
| `coordination_agent_handoff` | `coordination_agent_handoff` | pending_review | `backend/crates/agent/src/skills/coordination.rs` | 交接 |
| `coordination_task_orchestration` | `coordination_task_orchestration` | pending_review | `backend/crates/agent/src/skills/coordination.rs` | 任务编排 |
| `coordination_status_query` | `coordination_status_query` | pending_review | `backend/crates/agent/src/skills/coordination.rs` | 状态查询，需真实化 |
