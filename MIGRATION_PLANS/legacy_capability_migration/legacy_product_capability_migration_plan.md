# 旧产品能力迁移到新产品的全量规划

## 0. 文档定位

本文件用于后续持续开发读取和执行，目标是把旧产品 `D:\github\commerce-agents崩了之后重新做` 中已经形成的电商多智能体能力、skill、业务模块、前端体验和验证方法，按新产品 `D:\github\atlas\commerce-agents-fullstack-main` 的 Rust 后端、React 前端、Postgres 数据模型和协议体系重新实现。

这不是代码搬迁方案。旧产品是 Python FastAPI + SQLite + Vanilla JS 的大管道式系统，新产品是 Rust workspace + agent runtime + service/web-server 分层 + React workspace。迁移原则是“保留能力与产品思想，重建工程实现”。

本轮约束：

- 不再沿用之前“不能改 `backend/crates/agent/`”的限制。agent 内核可以改，但必须按新产品的 agent runtime 边界修改。
- 不允许把旧产品 `chat_pipeline.py` 的大函数结构搬进新产品。
- 不允许在 `web-server` handler 中直接创建 LLM client。LLM 仍由 `agent` crate 统一持有。
- 不允许前端假造能力状态。前端展示必须来自后端 API、SSE event、业务数据或明确的本地静态说明。
- 迁移执行必须以后端为主线。先让 Rust 后端能力、数据、SSE/API、测试和 smoke 达标，再让前端按后端契约适配。
- 新增或修改 API 必须同步更新根目录 `API_DOCS/`。

## 1. 当前事实基线

### 1.1 旧产品基线

旧产品核心定位：电商多 Agent 协同智能平台，让用户用自然语言驱动运营、数据、客服、设计、财务、工程、web、创意等虚拟员工完成电商经营工作。

已确认旧产品能力：

- 8 个主角色：`ops`、`data`、`service`、`design`、`accounting`、`engineering`、`web`、`creative`。
- 46 个 skill，分 9 类：运营、数据分析、客服、设计、财务、工程、网站增长、创意内容、协同编排。
- 主链路：用户消息 -> intent 规则路由 -> context 富化 -> feature budget -> prompt 构建 -> LLM 流式输出 -> tool use -> 多 Agent 协作 -> 质量检查 -> 学习/信任/告警/指标。
- 业务模块：产品管理、产品知识、素材资产、竞品、活动、工作区、日报、告警、审批、包治理、历史会话。
- 前端体验：聊天主界面、Agent 激活态、ToolCallCard、QualityCard、TrustBadge、AlertCard、IntelligencePanel、工作区产物、历史会话。

旧产品典型工程位置：

- `server/src/core/chat_pipeline.py`：旧主流程，不能直接迁移结构。
- `server/src/core/intent.py`：规则意图识别，可迁移规则思想。
- `server/src/core/multi_agent.py`：主 Agent + 支持 Agent 协同，可迁移协同策略。
- `server/src/core/feature_budget.py`：top skill 选择，可迁移为 Rust role skill budget。
- `server/src/core/prompt_builder.py`、`prompt_injectors.py`：prompt 分层思想可迁移。
- `server/src/core/quality_checker.py`、`trust_scorer.py`、`agent_memory.py`：质量、信任、学习闭环可迁移。
- `server/src/skills/*.py`：旧 skill 业务语义和计算逻辑来源。
- `server/packages/roles/*/manifest.yaml`、`server/packages/skills/*/manifest.yaml`：角色/skill package 元数据来源。
- `client/src/core/sse-event-router.js`、`client/src/components/*`：前端事件展示思想来源。

### 1.2 新产品基线

新产品当前已经具备的承载点：

- `backend/crates/agent/`：
  - `RoleCatalog`、`RoleProfile`、`RoleRouter`
  - `Commander`、`Worker`、`TaskGraphRuntime`
  - `SessionRuntime`、`SessionEvent`
  - `SkillSpec`、`SkillRegistry`、`SkillExecutorSet`
  - staged skill specs 当前测试显示 93 个
  - `config/json/agent/roles.json` 已有 9+角色，包括 `chat` 和 `douyin_ads`
- `backend/crates/service/`：
  - conversation、product、product_asset、product_knowledge、product_material、product_competitor
  - workspace、briefing、approval_audit、package_governance、platform_connect 等业务 API
- `backend/crates/web-server/`：
  - `/api/sse/agent/session/stream`
  - `/api/app/*` 业务路由
  - JWT 鉴权、SSE 输出、agent_manager 静态入口
- `frontend/apps/web/`：
  - agent-chat 主问答页面
  - 策略选择 UI
  - 过程面板、右侧工作区、历史会话
  - commerce 各业务页面

新产品当前已验证：

- 云端 Ollama OpenAI-compatible endpoint 可访问：`http://119.29.232.53:16814/v1`
- `gpt-oss:20b` 模型可用。
- 前端发送问题 -> `/api/sse/agent/session/stream` -> agent runtime -> LLM -> 页面显示回答，主链路已通。

## 2. 总体迁移原则

### 2.1 思想迁移，不做旧代码复刻

旧产品的价值是：

- 电商岗位虚拟员工体系
- 自动路由和协同
- skill 能力矩阵
- 业务数据和产品知识注入
- 回答质量、信任、学习闭环
- 可视化过程和交付物

这些要迁移。旧产品的大 Python pipeline、SQLite 表结构、Vanilla JS 组件体系不直接迁移。

### 2.2 新产品分层职责

迁移后职责边界如下：

- `backend/crates/agent/`：智能体生命周期、角色路由、任务图、skill 调用、记忆、质量事件、SSE agent event。
- `backend/crates/service/`：业务数据编排、事务、权限、跨表业务逻辑。
- `backend/crates/repo/`：单表数据访问。
- `backend/crates/web-server/`：HTTP DTO、路由、鉴权、SSE/WS 接入、错误映射。
- `frontend/apps/web/`：业务 app 装配、问答页面、过程展示、业务页面对齐。
- `API_DOCS/`：唯一 API 文档入口。
- `temp/DEVELOPMENT_DOCS/`：后续开发设计与迁移台账。

### 2.3 后端优先，前端跟随后端适配

本迁移不是先做页面再补接口。产品核心能力在后端，尤其是 agent runtime、skill 执行、业务数据读取、任务图编排、质量检查、审批和 SSE 事件。前端只负责把这些真实能力以用户能理解的方式呈现出来。

每个能力都按这个顺序推进：

1. 后端能力定义：确认能力属于 `agent`、`service`、`repo`、`web-server` 哪一层。
2. 后端设计文档：需要 API、DB 或跨模块逻辑时，先补后端设计。
3. Agent/skill 实现：角色路由、skill spec/executor、任务图、上下文注入、质量事件先在 Rust 后端闭环。
4. Service/API 实现：业务数据读写、权限、事务、错误码和 SSE/HTTP DTO 明确。
5. API 文档同步：所有新增或修改接口进入根目录 `API_DOCS/`。
6. 后端验证：`cargo fmt`、`cargo check`、`cargo clippy`、相关测试、agent smoke、真实 SSE 请求通过。
7. 前端适配：只在后端契约稳定后，改页面、事件类型、过程卡片、工作区和业务入口。
8. 端到端验收：用户从前端提问，后端真实执行，前端显示真实结果。

前端可以提前做信息架构草图和交互方案，但不能把未完成的后端能力做成假状态、假流程或静态演示。凡是后端事件、skill result、quality check、approval、workspace artifact 未提供真实数据，前端只能显示“能力未接入/等待后端事件”，不能伪造过程。

### 2.4 能力验收必须以“用户能用”为准

每个能力迁移完成不以“代码存在”为准，而以以下结果为准：

- 用户能在前端自然语言触发。
- 后端能正确路由角色。
- 如需 skill，SSE 能展示真实 skill request/result。
- 业务数据能读写真实表。
- 最终回答能把工具结果和业务数据合成到用户可理解的答案里。
- 失败时有清楚的错误、风险、缺口提示。
- 对应 API 文档、测试和验证命令齐全。

## 3. 迁移对象总目录

### 3.1 认知层

迁移对象：

- 意图识别
- 角色路由
- 回答策略
- feature budget
- prompt 分层
- 任务复杂度判断
- 澄清问题判断
- 质量评分
- 信任度
- 学习记忆

新产品落点：

- `backend/crates/agent/src/roles/router.rs`
- `backend/crates/agent/src/agent/commander/*`
- `backend/crates/agent/src/cognition/*`
- `backend/crates/agent/src/agent_session/memory/*`
- `backend/config/prompts/zh/cognition/*`
- `backend/config/json/agent/*`

### 3.2 编排层

迁移对象：

- TIER_SINGLE
- TIER_MULTI
- TIER_CHAIN
- 主角色 + 支持角色
- role handoff
- skill chain
- Kimi Agent 集群式任务拆解
- worker report 合成

新产品落点：

- `TaskGraphRuntime`
- `Commander`
- `Worker`
- `RoleCollaborationPlan`
- `SessionEvent`
- 前端 runtime event mapping

### 3.3 Skill 工具层

迁移对象：

- 旧产品 46 个基础 skill
- search skill
- coordination skill
- 高风险 platform skill
- skill feedback
- skill run audit
- skill success rate

新产品落点：

- `backend/crates/agent/src/skills/*`
- `backend/crates/agent/src/roles/skill_catalog.rs`
- `backend/crates/service/src/api/package_governance.rs`
- `backend/crates/service/src/api/approval_audit.rs`
- `API_DOCS/sse.md`
- `API_DOCS/app.md`

### 3.4 业务执行层

迁移对象：

- 产品生命周期
- 产品素材
- 产品知识
- 竞品
- 营销活动
- 工作区任务
- 日报
- 告警
- 审批回滚
- 平台连接
- 能力包治理

新产品落点：

- `backend/crates/service/src/api/product*.rs`
- `backend/crates/service/src/api/workspace.rs`
- `backend/crates/service/src/api/briefing_ops.rs`
- `backend/crates/service/src/api/approval_audit.rs`
- `backend/crates/service/src/api/platform_connect.rs`
- `backend/crates/service/src/api/package_governance.rs`
- `backend/crates/web-server/src/handlers/app/*`
- `frontend/apps/web/src/pages/site/commerce/*`

### 3.5 知识与记忆层

迁移对象：

- 历史对话
- 产品三层知识
- 附件摘要
- 商品上下文绑定
- 长期记忆
- role trust
- 用户反馈
- RAG/vector 检索

新产品落点：

- conversation 表与 API
- product_knowledge 表与 API
- product_asset/product_material 表与 API
- agent session memory
- 后续 vector store 或 Postgres pgvector 设计

### 3.6 前端体验层

迁移对象：

- 问答为核心的第一屏
- 回答策略选择
- Agent 激活态
- ToolCallCard
- QualityCard
- TrustBadge
- AlertCard
- 工作区交付物
- 左右侧可折叠布局
- 历史会话
- 业务页面联动

新产品落点：

- `frontend/apps/web/src/pages/site/agent-chat/*`
- `frontend/apps/web/src/shell/AgentChatShell.tsx`
- `frontend/apps/web/src/pages/site/commerce/*`
- `frontend/apps/web/src/api/*`
- `@workspace/app-kit` / `@workspace/ui-components`

## 4. 旧 Skill 全量迁移计划

### 4.1 运营 ops

旧 skill：

- `ops_promo_planning`
- `ops_pricing_strategy`
- `ops_channel_strategy`
- `ops_inventory_planning`
- `ops_listing_copy`
- `ops_assortment_planning`
- `ops_execution_plan`

迁移要求：

- 先核对 Rust `ops.rs` 是否已有同名或等价 spec。
- 对本地计算类直接 Rust native 实现。
- 涉及商品/活动/库存真实数据时，通过 service query adapter 读取业务数据。
- 涉及创建活动或任务时走 coordination/business API，不允许 agent 直接写数据库。

验收：

- 用户问“新品冷启动/定价/库存/活动”时能路由到 ops。
- ops worker 能输出可执行计划。
- 如调用 skill，SSE 展示 skill result。
- 前端回答区能显示最终策略，右侧能显示执行步骤。

### 4.2 数据 data

旧 skill：

- `data_funnel_analysis`
- `data_anomaly_diagnosis`
- `data_trend_forecast`
- `data_customer_segmentation`
- `data_competitor_analysis`
- `data_dashboard`

迁移要求：

- 漏斗、异常、趋势、RFM 优先 native。
- 竞品分析要与 product_competitor service 打通。
- 数据看板需要明确数据源：假数据、用户输入、业务表、平台连接数据必须区分。
- 后续补 `query_store_metrics`、`query_campaign_metrics`、`query_product_metrics` adapter。

验收：

- 用户提供指标序列时可直接分析。
- 用户绑定产品/活动时能读取业务指标。
- 回答必须标明数据来源和置信度。

### 4.3 客服 service

旧 skill：

- `service_ticket_handler`
- `service_faq_playbook`
- `service_escalation_flow`
- `service_dsr_improvement`
- `service_return_handler`

迁移要求：

- FAQ、工单、升级、DSR、退换货可先 native。
- 后续接入 conversation、product、platform policy 和售后记录。
- 客诉高风险时触发 alert/approval 事件。

验收：

- 用户问售后/差评/投诉/退款时路由到 service。
- 输出必须包含话术、处理步骤、风险、升级条件。

### 4.4 设计 design

旧 skill：

- `design_main_image`
- `design_detail_page`
- `design_color_scheme`
- `design_material_spec`
- `design_campaign_poster`

迁移要求：

- 先做设计方案文本输出，不做图像生成。
- 平台规格、尺寸、素材格式要结构化返回。
- 与 product_asset/product_material 关联，能基于已上传素材给建议。

验收：

- 用户问主图/详情页/海报/素材规格时路由到 design。
- 输出可被前端展示为素材清单或设计 brief。

### 4.5 财务 accounting

旧 skill：

- `accounting_cost_calc`
- `accounting_profit_analysis`
- `accounting_budget_plan`
- `accounting_roi_calc`
- `accounting_compliance_check`

迁移要求：

- 成本、利润、ROI、预算优先 native。
- 财务数据缺失时要求用户补字段，不允许编造。
- 预算计划可和 campaign/workspace 创建任务联动。

验收：

- 用户问成本、毛利、ROI、预算时路由到 accounting。
- 输出必须包含公式、关键假设、风险。

### 4.6 工程 engineering

旧 skill：

- `engineering_arch_review`
- `engineering_bug_analysis`
- `engineering_perf_optimize`
- `engineering_tech_selection`

迁移要求：

- 作为电商技术岗位能力保留。
- 不能越权执行本机命令或改代码，除非明确是开发任务。
- 在业务产品里主要用于平台连接、数据管道、SLA、接口稳定性建议。

验收：

- 用户问系统接口、性能、故障、平台对接时路由到 engineering。
- 输出检查清单和排障路径。

### 4.7 网站增长 web

旧 skill：

- `web_seo_optimize`
- `web_title_generator`
- `web_page_conversion`
- `web_store_design`
- `web_keyword_research`

迁移要求：

- SEO、标题、关键词、页面转化先 native。
- 后续与 product_material、product_knowledge、search adapter 联动。

验收：

- 用户问标题/关键词/页面转化时路由到 web。
- 可输出多版本标题、关键词意图、转化优化建议。

### 4.8 创意 creative

旧 skill：

- `creative_video_script`
- `creative_seeding_copy`
- `creative_ip_branding`
- `creative_content_calendar`
- `creative_trend_catch`
- `creative_live_script`

迁移要求：

- 文案、脚本、直播流程、内容排期 native。
- 趋势捕捉涉及搜索时先 deferred/search adapter。
- 与 design、ops、web 协同。

验收：

- 用户问短视频/直播/小红书/种草/内容排期时路由到 creative。
- 输出结构化脚本或内容日历。

### 4.9 协同 coordination

旧 skill：

- `coordination_agent_handoff`
- `coordination_task_orchestration`
- `coordination_status_query`

迁移要求：

- 不能只返回静态模板。
- 要结合当前 task graph、worker 状态、workspace/task 状态。
- 支持创建 campaign、task、memory、skill chain plan。

验收：

- 复杂任务能拆成角色任务。
- 前端能看到任务图和各岗位状态。

### 4.10 Search 与外部数据

旧产品有 `search.py` 和 search providers。

迁移要求：

- 第一阶段 search skill 保持 deferred，明确提示需要 adapter。
- 第二阶段接入可用搜索 provider 或内部知识库。
- 第三阶段接平台数据、行业 benchmark、竞品公开资料。

验收：

- 没接外部搜索时不能伪造“已搜索”。
- 接入后必须显示来源、时间、置信度。

## 5. 回答策略迁移计划

旧产品强调问答核心，新产品当前前端已有策略选项，但策略主要以 prompt 文本方式注入。需要升级为后端可识别的 request metadata。

### 5.1 策略类型

- `execution`：先给可用答案，再列动作。
- `learning`：解释逻辑路径，适合用户学习。
- `collab`：多角色协同，先拆岗位，再综合。

### 5.2 后端改造

新增或扩展 Agent SSE request：

- `strategy_id`
- `response_mode`
- `collaboration_mode`
- `preferred_roles`
- `product_ids`
- `workspace_id`
- `context_refs`

落点：

- `backend/crates/web-server/src/dto/sse.rs`
- `backend/crates/agent/src/agent_manager/manager.rs`
- `backend/crates/agent/src/agent_session/session_runtime.rs`
- `backend/crates/agent/src/agent/commander/routing.rs`

### 5.3 前端改造

落点：

- `frontend/apps/web/src/pages/site/agent-chat/domain/agent-chat-strategy.ts`
- `frontend/apps/web/src/pages/site/agent-chat/view-model/agent-chat-send-message.ts`
- `frontend/apps/web/src/api/agent.ts`

验收：

- 前端切换策略后，后端事件和最终回答行为有可观察差异。
- `collab` 策略必须更容易触发多 worker。
- `learning` 策略必须展示推理路径但不能冗长。

## 6. 多 Agent 编排迁移计划

### 6.1 单角色

适用：

- 普通知识解释
- 简单问答
- 单岗位任务

要求：

- 能直接返回回答。
- 不展示无意义的完整任务图。
- 前端右侧可折叠为轻量过程。

### 6.2 多角色

适用：

- 新品冷启动
- 投放复盘
- 活动策划
- 客诉危机
- 选品/定价/库存/预算综合问题

要求：

- Commander 明确 primary role 和 support roles。
- 每个 worker 产出结构化 `RoleWorkOutput`。
- 最终 synthesize 合成一份用户可用答案。

### 6.3 链式任务

适用：

- 数据诊断 -> 运营方案 -> 创意脚本 -> 预算测算 -> 交付文档
- Kimi Agent 集群式长任务

要求：

- task graph 有依赖关系。
- 每个节点状态可通过 SSE 展示。
- 节点失败能重试或降级。
- 最终产物可写入 workspace。

## 7. 业务模块迁移计划

### 7.1 产品生命周期

旧能力：

- draft -> listing -> active -> archived
- 产品基础信息
- 商品知识
- 素材资产
- PIN 删除保护

新落点：

- `product.rs`
- `product_knowledge.rs`
- `product_asset.rs`
- `product_material.rs`
- 需要补生命周期状态机和前端状态展示。

任务：

- 核对当前 product 表字段是否覆盖 lifecycle。
- 补状态变更 API。
- Agent 回答时可引用产品上下文。
- 前端产品页显示生命周期、知识、素材、操作入口。

### 7.2 产品知识与附件

旧能力：

- L1 基础信息
- L2 结构化知识
- L3 文件摘要
- 上传文档/图片/视频摘要

新落点：

- `product_knowledge`
- `product_asset`
- `storage`
- agent context adapter

任务：

- 先支持文本知识和文件元数据。
- 再支持附件摘要。
- 再支持向量检索。
- 最后支持多模态图片理解。

### 7.3 竞品

旧能力：

- 竞品信息
- 价格/活动/销量/策略对比

新落点：

- `product_competitor`
- data skill
- search adapter

任务：

- 支持手工录入竞品。
- 支持 Agent 基于竞品记录分析。
- search 接入后支持外部竞品补充。

### 7.4 营销活动

旧能力：

- campaign CRUD
- campaign items
- 活动节奏
- 预算和 KPI

新落点：

- `campaign.rs`
- `workspace.rs`
- ops/accounting/data skill

任务：

- 让 Agent 可根据对话创建 campaign draft。
- 高风险写入先 pending approval。
- 前端活动页展示来自后端的真实活动。

### 7.5 工作区与交付物

旧能力：

- workspace
- workspace task
- workspace memory
- 项目任务
- 交付文件

新落点：

- `workspace.rs`
- `long_task.rs`
- 前端 workspace 页面
- agent task graph

任务：

- 对齐 agent session 和 workspace run。
- 把复杂问答沉淀为 workspace run。
- worker output 可转为 workspace task/report/material。

### 7.6 日报、告警、主动通知

旧能力：

- briefing daily
- alert engine
- proactive engine

新落点：

- `briefing_ops.rs`
- `approval_audit.rs`
- `notifications`
- WebSocket 小铃铛

任务：

- 先做手动生成日报。
- 再做业务规则告警。
- 最后做主动通知和定时任务。

### 7.7 平台连接

旧能力：

- 外部平台连接
- 平台数据同步
- 可选自动执行

新落点：

- `platform_connect.rs`
- high-risk platform skill
- approval audit

任务：

- 只保存连接元数据和状态。
- 真实同步必须审计。
- 改价/库存等写操作必须 approval。

### 7.8 能力包治理

旧能力：

- role/domain/skill package 扫描
- 启停
- 能力矩阵

新落点：

- `package_governance.rs`
- `agent roles/skills registry`
- 前端包治理页面

任务：

- 展示当前 role 和 skill catalog。
- 支持启停策略配置。
- 后续可做 package manifest 导入。

## 8. 数据与 API 规划

### 8.1 需要核对的现有表

先核对当前 repo entity/migration 是否已有：

- app_users
- conversations
- conversation_messages
- products
- product_assets
- product_materials
- product_knowledge
- product_competitors
- campaigns
- workspaces
- workspace_tasks
- workspace_runs
- long_running_tasks
- approval/audit 相关表
- package_registry
- platform_connectors

### 8.2 预计新增或增强表

待后端设计时确认：

- `agent_skill_runs`：记录 skill 调用、输入摘要、状态、输出摘要、risk level。
- `agent_quality_checks`：记录 9维评分、失败原因、重试建议。
- `agent_trust_scores`：按 user/role/skill 记录信任等级。
- `agent_learning_records`：用户反馈和成功案例沉淀。
- `agent_context_refs`：对话绑定的 product/campaign/workspace/material 引用。
- `agent_runtime_events`：可选持久化关键 SSE 事件。
- `agent_route_decisions`：保存 commander 路由结果，便于复盘。

是否新增表必须按后端文档流程单独确认，不能直接开发。

### 8.3 API 文档计划

需要维护：

- `API_DOCS/sse.md`：Agent SSE request/response/event。
- `API_DOCS/app.md`：业务接口。
- `API_DOCS/common.md`：LLM 配置、agent runtime、错误模型。
- 可能新增 `API_DOCS/agent-capabilities.md`：角色、skill、质量、学习、审批事件总览。

## 9. 前端迁移规划

前端迁移是后端能力达标后的适配工作，不作为能力迁移的起点。所有前端任务必须先确认对应后端事件、API、DTO、错误码和权限规则已经存在或已经在后端设计文档中确认。前端不承担业务事实判断，不在本地补一套 agent 状态机，不把 prompt、角色路由、skill 调用逻辑写进页面。

### 9.1 问答首页

目标：

- 第一屏就是电商多智能体问答。
- 未登录可看体验，但发送真实问题需要登录或明确 guest 策略。
- 登录后历史会话、产品上下文、工作区沉淀可用。

任务：

- 修正问答页面信息密度。
- 策略选择与后端 metadata 打通。
- 用户输入区保留样例、附件、上下文 chip。
- 回答区支持结构化区块：结论、证据、风险、下一步。

### 9.2 左右侧折叠

目标：

- 左侧历史/导航可折叠。
- 右侧虚拟员工工作区可折叠。
- 折叠状态稳定，不遮挡主问答。

任务：

- 使用现有 shell/sidebar 组件能力。
- 不在 app 内重写通用 sidebar primitive。
- 移动端默认收起侧栏。

### 9.3 Runtime 事件展示

需要展示：

- route decision
- task graph planned
- worker started/finished
- skill requested/result
- quality checked
- memory persisted
- alert triggered
- approval required
- workspace artifact created

任务：

- 扩展 `AgentRuntimeEvent` 类型。
- 扩展 event mapping。
- 新增或改造过程卡片，但优先使用 `app-kit`/`ui-components`。

### 9.4 业务页面联动

目标：

- 用户在聊天中生成的产品、活动、任务、日报、审批可在对应页面看到。
- 业务页面可以把当前对象作为上下文发回聊天。

任务：

- 产品页增加“带入问答”。
- 活动页增加“让 Agent 复盘/优化”。
- 工作区页展示 agent run。
- 审批页展示高风险 tool 操作。

## 10. 分阶段开发计划

### 10.0 阶段执行门禁

每个 Phase 都拆成后端门禁和前端适配两段：

后端门禁必须先通过：

- 能力边界确认：明确落在 `agent`、`service`、`repo`、`web-server`、`API_DOCS` 的哪些文件和模块。
- 后端设计完成：涉及新 API、DB、SSE、错误码、权限或 LLM 调用时，先写后端设计。
- Rust 实现完成：agent runtime、skill executor、service 逻辑、DTO、route、event 都按新产品分层实现。
- API_DOCS 更新：前端只按根目录 `API_DOCS/` 对接。
- 后端验证完成：`cargo fmt`、`cargo check`、`cargo clippy`、相关测试、agent smoke、真实 SSE/HTTP 请求。

前端适配只能在后端门禁之后开始：

- 同步 TypeScript event/API 类型。
- 页面按真实数据展示 route、worker、skill、quality、approval、artifact。
- 使用 `app-kit` 和 `ui-components`，不新增第二套组件体系。
- 跑 `pnpm rules` 和相关前端验证。
- 最后做浏览器真实问答和业务场景验收。

如果某个能力后端没有达标，该能力不得进入前端“视觉完成”状态，只能在台账中标为 `backend_pending`、`api_pending`、`sse_pending` 或 `data_pending`。

### Phase 0：迁移台账与能力核对

目标：建立可执行台账，不盲目开发。

任务：

- 生成旧 46 skill 与新 93 staged spec 的映射表。
- 标记每个 skill 状态：`ported_spec_only`、`native_ready`、`adapter_required`、`approval_required`、`missing`。
- 核对 8+角色的关键词、能力、协作对象。
- 输出 `skill_migration_matrix.md`。

验收：

- 每个旧 skill 都有新落点和状态。
- 每个旧角色都有新 role profile。

### Phase 1：问答策略与路由增强

目标：先让后端问答路由真正体现电商多智能体，再让前端策略选择跟随后端行为。

后端任务：

- Agent request 增加策略 metadata。
- Commander 使用策略影响路由。
- `chat` 快速问答和 `collab` 多角色任务分流。
- SSE 输出 route decision、strategy applied、worker planned 等真实事件。
- 更新 `API_DOCS/sse.md`。
- 通过 agent smoke 和真实 SSE 请求验证。

前端适配：

- 策略选择传给后端 metadata。
- 页面只展示后端返回的真实路由和协同差异。

验收：

- execution/learning/collab 三种策略有可观察差异。
- 简单问答不展示冗余流程。
- 复杂电商任务能触发多角色。

### Phase 2：Skill 执行与 SSE 可视化

目标：先让后端 skill 执行闭环，再让用户看到真实 skill 调用。

后端任务：

- 规范 `skill_requests` 输出格式。
- Worker 执行 skill 后发 `skill_requested`、`skill_result` event。
- 低风险 native skill 完成可用闭环。
- skill 失败、deferred、approval_required 都形成结构化事件。
- 更新 `API_DOCS/agent-capabilities.md` 或 `API_DOCS/sse.md`。
- 覆盖至少 6 个代表 skill 的后端测试或 smoke。

前端适配：

- ToolCallCard 消费真实 `skill_requested`、`skill_result`。
- 失败、deferred、approval 状态按后端事件展示。

验收：

- 至少覆盖 ops/data/accounting/service/creative/web 各 1 个真实 skill。
- skill 失败、deferred、approval 都能在前端正确显示。

### Phase 3：产品知识与上下文绑定

目标：先让后端能稳定读取产品、知识、素材上下文，再让前端提供选择入口。

后端任务：

- SSE request 支持 `product_ids`、`context_refs`。
- 后端从 service 读取产品知识，注入 agent context。
- 回答标明引用的产品/知识来源。
- 权限、空数据、缺失素材、超长上下文降级规则明确。
- 更新 API 文档并完成后端验证。

前端适配：

- 支持选择 product/context chip。
- 展示回答引用的产品、知识和素材来源。

验收：

- 绑定产品后，回答能引用真实产品信息。
- 未绑定产品时不假装知道商品详情。

### Phase 4：工作区沉淀与交付物

目标：先让后端把复杂问答沉淀为真实任务和产物，再让前端工作区展示。

后端任务：

- Agent run 与 workspace_run 对齐。
- Worker report 可生成 workspace task。
- 最终方案可沉淀为 briefing/material/report。
- artifact 创建、查询、权限和错误状态闭环。
- 更新 workspace/API 文档并完成后端验证。

前端适配：

- 前端右侧工作区展示真实后端数据。

验收：

- 一次新品冷启动问答可生成 workspace run。
- 页面刷新后仍能看到任务与结果。

### Phase 5：质量、信任、学习闭环

目标：旧产品质量体系迁移。

后端任务：

- 设计 9维质量检查结构。
- Agent 输出后生成 quality event。
- 用户反馈写入 learning/trust。
- 下一次 prompt 可读取相关经验。
- 明确质量分不参与自动高风险操作。
- 更新 API 文档并完成后端验证。

前端适配：

- 展示质量评分、反馈入口、历史偏好命中提示。

验收：

- 前端可展示质量评分。
- 用户反馈能保存。
- 后续回答能引用历史偏好或成功案例。

### Phase 6：审批、告警、平台写操作

目标：高风险能力可控上线。

任务：

- platform skill 全部默认 pending approval。
- 审批页承接 tool approval。
- 告警规则接业务数据。
- WebSocket 推送通知。

验收：

- LLM 不能直接改价/改库存。
- 用户审批后才能执行高风险动作。
- 审计记录可查。

### Phase 7：搜索/RAG/外部数据

目标：从内部智能升级到外部信息增强。

任务：

- search adapter 接入真实 provider。
- product knowledge 向量化检索。
- 行业 benchmark、竞品公开资料、平台政策可引用。

验收：

- 搜索结果有来源。
- RAG 命中可解释。
- 外部不可用时优雅降级。

### Phase 8：完整商业化体验

目标：旧产品能力完整落地为新产品体验。

任务：

- 完整端到端脚本。
- 全角色场景用例。
- 性能、错误、权限、安全检查。
- 文档和演示数据整理。

验收：

- 典型场景全通：新品冷启动、投放复盘、客诉危机、选品定价、素材优化、日报复盘。

## 11. 测试与验证策略

### 11.1 后端验证

后端验证是每个能力进入前端适配前的硬门禁。只要后端未通过，前端不得把该能力标为完成。

每次涉及 Rust 后端：

- `cargo fmt`
- `cargo check`
- `cargo clippy`
- 相关 crate 测试
- 涉及 agent/LLM 时运行 agent smoke

重点 smoke：

- `cargo run -p agent --bin ollama_task_graph_smoke`
- `cargo run -p agent --bin role_skill_project_smoke`
- web-server SSE 真实请求

### 11.2 前端验证

前端验证只在对应后端 API/SSE 契约稳定后执行。前端失败时优先判断是展示映射问题，还是后端能力、DTO 或事件缺口。

每次涉及前端：

- `pnpm -C apps/web rules`
- `pnpm -C apps/web lint`
- `pnpm -C apps/web test`
- `pnpm -C apps/web build`
- Playwright/浏览器级真实问答

### 11.3 端到端场景

固定场景集：

- 新品冷启动：ops + data + design + accounting
- 投放复盘：douyin_ads + data + creative + accounting
- 客诉危机：service + ops + data
- 商品定价：accounting + ops + data
- SEO 标题：web + creative + data
- 工作区交付：ops + data + creative + delivery/report

每个场景验收：

- route decision 正确。
- 至少一个 skill 可被调用。
- 最终回答有结论、证据、风险、下一步。
- 前端过程面板不空。
- 相关业务数据能保存或明确说明未保存。

## 12. 风险与处理

### 12.1 旧产品能力名已迁移但执行未迁移

风险：Rust 中有 spec，但只是壳。

处理：

- 通过 skill matrix 标记 spec/native/adapter 状态。
- 只有真实输出可用才算完成。

### 12.2 LLM 选择 skill 不稳定

风险：模型不按 JSON 输出 skill_requests。

处理：

- 强化 worker prompt。
- 增加 JSON decoder/repair。
- 对常见电商场景做 commander fast route + deterministic skill suggestion。

### 12.3 前端显示过程但后端无真实事件

风险：体验像假演示。

处理：

- 前端只消费真实 SSE event。
- 静态说明必须标明“待生成/未执行”。

### 12.4 高风险平台写操作误执行

风险：LLM 直接改价、库存、同步商品。

处理：

- platform skill 永远 `PendingApproval` 起步。
- approval audit 必须记录 input、operator、result。
- 前端审批后再执行。

### 12.5 业务数据权限错乱

风险：用户看到或操作别人的产品/任务。

处理：

- 所有 service API 使用 JWT user_id。
- repo/service 过滤 user_id。
- E2E 覆盖多用户隔离。

### 12.6 prompt 膨胀

风险：把所有角色、skill、产品知识全塞进 prompt。

处理：

- 保留 feature budget。
- role prompt 只注入当前角色 top skills。
- 产品知识只注入命中摘要。

### 12.7 新旧架构混乱

风险：为了迁移快，在 web-server/service/agent 重复实现逻辑。

处理：

- LLM/agent 逻辑只进 agent crate。
- 业务数据只进 service/repo。
- HTTP 只进 web-server。
- 前端只做展示和交互装配。

## 13. 开发台账要求

后续每完成一个能力，需要在本目录新增或更新台账文件：

- `skill_migration_matrix.md`
- `role_migration_matrix.md`
- `api_migration_checklist.md`
- `frontend_experience_checklist.md`
- `phase_acceptance_log.md`

每个能力记录：

- 旧产品来源文件
- 新产品落点文件
- 是否改 agent crate
- 是否新增 API
- 是否新增 DB
- 是否更新 API_DOCS
- 测试命令
- 浏览器验证结果
- 未完成风险

## 14. 第一批建议立即执行的开发任务

第一批不要追求全量业务模块，先让后端问答主链路形成“旧产品精髓”，再做前端适配：

1. 建立 skill migration matrix。
2. 后端扩展 Agent SSE request metadata，接入回答策略。
3. 后端扩展 agent SessionEvent：skill_requested、skill_result、quality_checked、route_decided。
4. 后端让 collab 策略稳定触发多角色任务图。
5. 后端核对并补齐 6 个代表性 native skill：
   - `ops_pricing_strategy`
   - `data_funnel_analysis`
   - `accounting_roi_calc`
   - `service_return_handler`
   - `creative_live_script`
   - `web_title_generator`
6. 更新 `API_DOCS/`，完成 cargo 验证、agent smoke 和真实 SSE 请求。
7. 前端 ToolCallCard、策略选择、过程面板再消费真实后端 event。
7. 做一个端到端场景：新品冷启动。

完成标准：

- 用户在前端提问新品冷启动。
- 后端路由到 ops/data/design/accounting 中至少 2 个角色。
- 至少 1 个 skill 真实执行。
- 页面显示过程、工具结果和最终答案。
- conversation 保存。
- 无 agent 假事件。

## 15. 后续决策点

开发前需要逐项确认：

- 是否允许新增 agent quality/trust/learning 相关表。
- 是否优先做本地 native skill，还是先接搜索/RAG。
- 平台连接是否只做模拟审批，还是接真实外部平台。
- 附件摘要第一版支持哪些格式。
- 旧产品 SQLite 历史数据是否需要迁移，还是只迁移能力不迁移数据。
- 是否需要保留旧产品一键脚本作为参考，还是完全使用新产品 cargo/pnpm 流程。

## 16. 总结

迁移的核心不是“把旧产品搬过来”，而是把旧产品已经证明有价值的电商多智能体产品能力，按新产品更清晰的 Rust agent runtime、service/web-server 分层、React 前端体验重建。

正确完成后，新产品应该具备：

- 以问答为核心的电商多智能体入口。
- 真实可观察的多角色协作。
- 真实 skill 调用和风险控制。
- 产品、素材、知识、活动、工作区等业务数据进入回答。
- 质量、信任、学习和告警形成闭环。
- 前端呈现不再是静态演示，而是后端能力的实时映射。
