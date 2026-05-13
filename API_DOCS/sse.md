# SSE API

接口前缀：`/api/sse`

说明：SSE 用于把 agent 回复以文本流形式返回给前端；普通业务通知仍然使用 WebSocket `/api/ws`。

## 1. Agent 回复流

- 方法：`POST`
- 路径：`/api/sse/agent/session/stream`
- 鉴权：有效 App 用户 JWT
- 响应类型：`text/event-stream`

请求体：

```json
{
  "input": "请用一句话解释 agent session 是什么",
  "agentId": "web-agent",
  "sessionId": "session-001",
  "workspaceId": "workspace-001",
  "metadata": {
    "context": {
      "conversationId": "conversation-uuid",
      "productIds": ["product-uuid"],
      "contextRefs": ["product:product-uuid"]
    },
    "strategy": {
      "id": "collab",
      "responseMode": "execution",
      "collaborationMode": "manual",
      "learningLevel": "higher_vocational",
      "primaryRole": "data",
      "selectedRoles": ["data", "ops", "creative"]
    },
    "model": {
      "name": "ollama-gpt-oss-20b"
    }
  }
}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `input` | `string` | 是 | 用户输入，不能为空，最大 20000 字符 |
| `agentId` | `string` | 否 | agent ID，不传时使用 `web-agent` |
| `sessionId` | `string` | 否 | 会话 ID，不传时服务端生成 |
| `workspaceId` | `string` | 否 | 工作区 ID；不传时后端会创建或复用当前用户默认 `agent_chat` 工作区 |
| `metadata.context.conversationId` | `string` | 否 | 绑定到本轮问答的会话 ID；后端会读取该会话下已注册的 active 附件，并注入工作区上下文摘要 |
| `metadata.context.productIds` | `string[]` | 否 | 绑定到本轮问答的商品 ID，最多 5 个；后端会校验当前用户权限并读取真实商品/知识 |
| `metadata.context.contextRefs` | `string[]` | 否 | 上下文引用，当前支持 `product:{productId}`，最多 20 个 |
| `metadata.model.name` | `string` | 否 | 指定本轮使用的后端 `llm.name`；不传时使用后端默认模型 |
| `metadata.strategy.id` | `string` | 否 | 回答策略 ID，当前前端使用 `execution` / `learning` / `collab` |
| `metadata.strategy.responseMode` | `string` | 否 | 回答模式，当前前端使用 `execution` / `learning` |
| `metadata.strategy.collaborationMode` | `string` | 否 | 协作模式，当前前端使用 `single` / `manual`；`manual` 会要求后端优先进入多角色任务图 |
| `metadata.strategy.learningLevel` | `string` | 否 | 学习/讲解层级，例如 `beginner` / `higher_vocational` / `professional` |
| `metadata.strategy.primaryRole` | `string` | 否 | 用户指定的主岗位 runtime role，例如 `data`、`ops`、`creative` |
| `metadata.strategy.selectedRoles` | `string[]` | 否 | 用户指定参与本轮协作的岗位 runtime role，最多 6 个，服务端去重 |

`metadata.context.*` 只作为引用入口，不允许前端直接塞商品事实或附件正文。后端会通过 service 层读取当前用户自己的 active 商品、active 商品知识、会话附件、商品素材和工作区交付物，压缩成 agent runtime metadata：`context.product_ids`、`context.refs`、`context.summary`、`context.sources_json`、`workspaceContext.*` 等。商品、会话、工作区不存在、已归档、不属于当前用户或 ID 非法时返回对应业务错误码。商品上下文是显式选择能力：只有传入 `metadata.context.productIds` 或 `metadata.context.contextRefs` 中的 `product:{id}` 时，后端才会注入对应商品资料；如果本轮没有选择商品，即使用户问题包含“咖啡”等关键词，也不会自动把某个已有商品或商品知识注入模型上下文。

`metadata.model.name` 必须是 `GET /api/app/agent/runtime-options` 返回的 `models[].name`。如果模型名不存在，服务端返回 `400`，不会启动 Agent，也不会创建工作区运行记录。后端只把模型配置名和模型名写入 runtime metadata，不向前端暴露 `base_url` 或 `api_key`。

`metadata.strategy.*` 仅允许小写字母、数字、`_`、`-`，单字段最大 64 字符。后端会把这些字段转换为 agent runtime metadata：`strategy.id`、`strategy.response_mode`、`strategy.collaboration_mode`、`strategy.learning_level`、`strategy.primary_role`、`strategy.selected_roles`。前端不得把策略说明拼进 `input` 来伪造后端策略行为。

当 `metadata.strategy.collaborationMode` 为 `manual` 或 `single`，且 `primaryRole` 或 `selectedRoles` 中包含非 `chat` 岗位时，后端会优先尊重这次显式岗位选择并进入 commander/任务图路由；即使用户输入文本本身像“只输出/answer only”这类短答请求，也不会被 `chat_bypass` 吞掉。没有显式非 `chat` 岗位选择的短答请求仍可走 `route_decided.route_mode=chat_bypass`，用于普通轻问答。

每次 SSE Agent 调用都会先在 service 层绑定工作区运行记录：如果 `workspaceId` 合法且属于当前用户，则写入该工作区；如果不传 `workspaceId`，后端会创建或复用当前用户的默认 `agent_chat` 工作区。绑定成功后，agent runtime metadata 会追加 `workspace.workspace_id`、`workspace.run_id` 和 `workspace.source=agent_session`。随后后端会调用 `WorkspaceApi::context_summary` 聚合会话附件、商品资产、商品素材和工作区 artifact，追加 `workspaceContext.*` metadata，并发送 `workspace_context_loaded` 事件；再调用 `WorkspaceApi::context_pack` 把数据库里已经存在的素材正文、资产描述、交付物正文/预览、以及 `conversation_file.metadata.extractionStatus=extracted` 的附件抽取文本整理成可读内容包，追加 `workspaceContextPack.*` metadata，并发送 `workspace_context_pack_loaded` 事件。Agent SSE 场景下，工作区绑定过商品也不会被当成本轮默认商品；必须由本轮 `metadata.context.productIds/contextRefs` 显式选择后，商品资产和商品素材才会进入本轮上下文。如果绑定或上下文归属校验失败，SSE 不会启动，接口直接返回业务错误，避免前端误以为本次结果已沉淀或已读取资料。

SSE 事件分为三类：

1. 原始 agent 过程事件，事件名来自 `agent::api::SessionEvent::event_name()`。
2. 工作区沉淀事件，由 web-server 调用 service 层写入真实 `workspaces`、`workspace_runs`、`workspace_tasks`、`workspace_run_tasks`、`workspace_artifacts` 后产生。
3. 兼容旧前端的文本事件：`delta` / `done` / `error`。

原始过程事件示例：

```text
event: started
data: {"type":"started","session_id":"session-001"}

event: route_decided
data: {"type":"route_decided","session_id":"session-001","strategy_id":"collab","response_mode":"execution","collaboration_mode":"manual","learning_level":"higher_vocational","model_name":"ollama-gpt-oss-20b","route_mode":"task_graph","primary_runtime_role":"data","support_runtime_roles":["ops","creative"]}

event: context_loaded
data: {"type":"context_loaded","session_id":"session-001","product_ids":["product-uuid"],"refs":["product:product-uuid"],"product_count":1,"knowledge_count":3,"source_count":4,"summary":"Product source product:...","sources_json":"[...]"}

event: rag_context_loaded
data: {"type":"rag_context_loaded","mode":"local_product_knowledge","queryTerms":["咖啡","转化"],"productIds":["product-uuid"],"knowledgeCount":3,"sourceCount":4,"summary":"Local commerce RAG context...","sources":[{"sourceType":"product_knowledge","id":"knowledge-uuid","productId":"product-uuid","title":"核心卖点"}],"degraded":false,"degradedReason":null,"externalSearchAvailable":false}

event: workspace_context_loaded
data: {"type":"workspace_context_loaded","workspaceId":"workspace-uuid","conversationId":"conversation-uuid","productId":"product-uuid","runId":null,"readiness":"ready","summary":"Workspace context is ready: 1 conversation files, 2 product assets, 3 product materials, 4 workspace artifacts are available for agent grounding and delivery.","counts":{"conversationFiles":1,"productAssets":2,"productMaterials":3,"workspaceArtifacts":4},"attachmentSources":[{"sourceType":"conversation_file","id":"file-uuid","title":"brief.pdf","mimeType":"application/pdf","fileSize":123456,"status":"active","refKey":"conversation_file:file-uuid","preview":"brief.pdf","metadata":{}}],"productAssetSources":[],"productMaterialSources":[],"artifactSources":[],"missing":[],"warnings":[],"agentContext":{"kind":"workspace_context_summary","refs":["conversation_file:file-uuid"]}}

event: workspace_context_pack_loaded
data: {"type":"workspace_context_pack_loaded","workspaceId":"workspace-uuid","conversationId":"conversation-uuid","productId":"product-uuid","runId":null,"readiness":"partial","summary":"Workspace context pack is partial: 2 readable item(s), 3600 text chars, 0 omitted item(s), from 1 conversation files, 1 product assets, 1 product materials, 1 workspace artifacts.","counts":{"conversationFiles":1,"productAssets":1,"productMaterials":1,"workspaceArtifacts":1},"items":[{"sourceType":"product_material","id":"material-uuid","title":"五一咖啡卖点","refKey":"product_material:material-uuid","contentKind":"content","text":"商品素材正文...","textChars":120,"truncated":false,"metadata":{}},{"sourceType":"conversation_file","id":"file-uuid","title":"brief.pdf","refKey":"conversation_file:file-uuid","contentKind":"extraction_pending","text":null,"textChars":0,"truncated":false,"metadata":{"extractionStatus":"pending","metadata":{"extraction":{"status":"pending","source":"upload","parser":"unassigned"}}}}],"omitted":[],"warnings":["1 conversation file(s) are still pending text extraction"],"agentContext":{"kind":"workspace_context_pack","refs":["product_material:material-uuid","conversation_file:file-uuid"],"contentRefs":["product_material:material-uuid"]}}

event: worker_started
data: {"type":"worker_started","session_id":"session-001","worker_id":"worker-001","task_id":"task-001"}

event: skill_requested
data: {"type":"skill_requested","session_id":"session-001","worker_id":"worker.chat","task_id":"task-001","request_id":"task-001-skill-1","skill_id":"coordination_agent_handoff","role":"chat","reason":"需要交接给更合适的虚拟员工"}

event: skill_result
data: {"type":"skill_result","session_id":"session-001","worker_id":"worker.chat","task_id":"task-001","request_id":"task-001-skill-1","skill_id":"coordination_agent_handoff","role":"chat","status":"success","summary":"Agent交接建议已生成","error":null,"output":{"status":"ready"},"execution_mode":"native","risk_level":"low","approval_required":false}

event: task_graph_planned
data: {"type":"task_graph_planned","session_id":"session-001","graph":{...}}

event: workspace_run_tasks_linked
data: {"tasks":[{"id":"run-task-uuid","runId":"run-uuid","workspaceId":"workspace-uuid","status":"pending"}]}

event: task_node_reported
data: {"type":"task_node_reported","session_id":"session-001","graph_id":"graph-001","node_id":"node-001","worker_id":"worker-001","report":{...}}

event: workspace_run_task_updated
data: {"id":"run-task-uuid","runId":"run-uuid","workspaceId":"workspace-uuid","status":"succeeded","metadata":{...}}

event: finished
data: {"type":"finished","session_id":"session-001"}

event: workspace_run_completed
data: {"id":"run-uuid","workspaceId":"workspace-uuid","status":"succeeded","source":"agent_session","report":{...}}

event: workspace_artifact_created
data: {"id":"artifact-uuid","workspaceId":"workspace-uuid","runId":"run-uuid","artifactType":"final_report","title":"Agent final report","storageKind":"inline","format":"markdown","status":"active"}

event: workspace_artifact_created
data: {"id":"search-artifact-uuid","workspaceId":"workspace-uuid","runId":"run-uuid","artifactType":"search_evidence","title":"Search evidence - 冷萃咖啡 竞品","storageKind":"inline","format":"json","source":"agent_session","status":"active","metadata":{"kind":"agent_search_evidence","skillId":"search_competitor","requestId":"task-001-skill-1","sourceCount":3,"confidence":0.82}}

event: agent_learning_record_created
data: {"type":"agent_learning_record_created","id":"learning-uuid","workspaceId":"workspace-uuid","runId":"run-uuid","role":"agent","category":"decision","content":"本品定价区间保持在 159-179 元，避免低于核心竞品均价。","confidence":0.88,"sourceAction":"workspace_memory_manual","status":"active","metadata":{"kind":"workspace_memory","memoryKey":"pricing_strategy_2024","memoryType":"decision","source":"agent_skill_result","sourceRole":"ops","requestId":"task-001-skill-2","skillId":"coordination_save_memory","upsertMode":"create"}}

event: campaign_draft_created
data: {"type":"campaign_draft_created","campaign":{"id":"campaign-uuid","userId":"user-uuid","productId":"product-uuid","name":"五一咖啡促销活动","campaignType":"promotion","description":"提升五一周末转化率","budget":5000.0,"startDate":"2026-05-01","endDate":"2026-05-05","status":"draft","metadata":{},"output":{},"actionMetadata":{},"createdAt":"2026-05-12T08:00:00Z","updatedAt":"2026-05-12T08:00:00Z"},"source":{"sourceType":"agent_skill_result","productId":"product-uuid","workspaceId":"workspace-uuid","sourceArtifactId":null,"sourceSummary":"Selected product and workspace context...","refKeys":["product:product-uuid","agent_output:coordination_create_campaign"]},"warnings":["campaign draft date range is incomplete"],"agentContext":{"type":"campaign_draft","source":{"sourceType":"agent_skill_result","productId":"product-uuid","workspaceId":"workspace-uuid","sourceArtifactId":null,"sourceSummary":"Selected product and workspace context...","refKeys":["product:product-uuid","agent_output:coordination_create_campaign"]},"planning":{"goal":"提升五一周末转化率","channels":["douyin","taobao"]},"warnings":["campaign draft date range is incomplete"]}}

event: quality_checked
data: {"qualityCheck":{"id":"quality-check-uuid","workspaceId":"workspace-uuid","runId":"run-uuid","artifactId":"artifact-uuid","role":"agent","score":0.82,"passed":true,"dimensions":{},"issues":[],"suggestions":[]},"trustUpdate":{"role":"agent","previousScore":0.5,"newScore":0.596,"previousLevel":"moderate","newLevel":"moderate","delta":0.096,"reason":"quality check passed; trust score moved toward the answer quality"},"learningRecords":[]}

event: approval_required
data: {"type":"approval_required","approval":{"id":"approval-uuid","workspaceId":"workspace-uuid","runId":"run-uuid","skillName":"platform_update_price","toolCallId":"task-001-skill-1","riskLevel":"L3","status":"pending","reason":"平台写操作需要审批","actionKey":"agent:run:...","result":{"status":"pending","source":"tool_approval_create","executionReadiness":{"status":"blocked","adapterStatus":"adapter_not_implemented","canApprove":false,"canExecuteNow":false,"willFailIfApproved":true,"platform":"douyin","skillName":"platform_update_price","message":"Platform execution adapter is not implemented yet for price update.","details":{"productId":"prod-1","skuId":null,"newPrice":99,"quantity":null,"page":null,"pageSize":null},"checkedAt":"2026-05-13T00:00:00Z"}}},"alert":{"id":"alert-uuid","workspaceId":"workspace-uuid","runId":"run-uuid","alertType":"agent_approval_required","severity":"warning","message":"Agent skill platform_update_price requires approval before execution.","status":"active"},"skillId":"platform_update_price","requestId":"task-001-skill-1","role":"ops","riskLevel":"L3","reason":"平台写操作需要审批"}
```

常见原始事件：

| 事件名 | 说明 |
| --- | --- |
| `started` | agent session 已开始 |
| `memory_loaded` / `memory_persisted` / `memory_compacted` | 记忆加载、保存或压缩 |
| `context_loaded` | 后端已读取并注入真实业务上下文，当前包含商品 ID、context refs、商品数量、知识数量、来源数量、摘要和来源 JSON |
| `rag_context_loaded` | 后端已根据用户问题检索当前用户自己的本地商品知识，返回检索模式、关键词、来源、摘要、是否降级和外部搜索可用状态 |
| `workspace_context_loaded` | 后端已聚合当前工作区的会话附件、商品资产、商品素材和工作区交付物，返回 `ready/partial/empty`、来源计数、引用 key、缺失项和警告，并已把摘要注入 agent metadata |
| `workspace_context_pack_loaded` | 后端已把可读上下文内容包注入 Agent metadata；内容只来自数据库已有文本字段或 `extractionStatus=extracted` 的附件抽取文本，不读取 object storage、不做 OCR、不伪造二进制附件正文 |
| `commander_thinking` | commander 正在规划或调度 |
| `route_decided` | 后端已识别策略和路由模式，包含策略、学习层级、实际模型、主角色和支持角色 |
| `worker_started` / `worker_finished` | 单个 worker 开始或完成 |
| `skill_requested` | worker 请求调用某个 skill，包含请求 ID、skill ID、角色和原因；请求来源可能是模型显式 `skill_requests`，也可能是后端按岗位/任务自动补偿的代表 skill |
| `skill_result` | skill 已返回执行结果，包含状态、摘要、错误、可选结构化 `output`、执行模式、风险等级和是否需要审批 |
| `collaboration_started` / `collaboration_worker_started` / `collaboration_worker_finished` / `collaboration_finished` | 多 worker 协作过程 |
| `task_graph_planned` / `task_graph_finished` | 任务图规划或完成 |
| `task_node_ready` / `task_node_started` / `task_node_reported` / `task_node_evaluated` / `task_node_rework_requested` / `task_node_passed` / `task_node_failed` | 任务节点生命周期 |
| `output` | agent 产生最终输出 |
| `failed` | agent session 失败 |
| `finished` | agent session 完成 |

`skill_requested` 的触发来源：

- 模型显式输出 `skill_requests` 时，后端优先执行模型请求，不再追加自动补偿。
- 模型没有输出 `skill_requests` 时，后端可根据当前 worker 岗位、任务文本、上下文摘要和上游结果，自动补偿一个代表 skill 请求。
- 自动补偿仍走角色白名单、`SkillExecutorSet`、真实 `skill_result` 和结果合并链路，不绕过权限，也不伪造前端事件。
- 自动补偿不会直接执行高风险平台写操作；平台写操作仍返回 `pending_approval` 并进入审批链路。
- `coordination_create_campaign` 在返回 `status=success` 且 `campaign_creation.should_persist=true` 时，web-server 会把结构化草稿沉淀成真实 `campaign draft`，并额外发送 `campaign_draft_created` 与对应 `workspace_artifact_created`。
- 搜索类 `search_*` skill 会读取后端 `[external_search]` 配置：配置完整且 provider 返回可信来源时，`skill_result` 可返回 `status=success`、`execution_mode=adapter`，`output` 内包含 `status=live`、`sources`、`data.items`、`provenance` 和 `confidence`；未配置、provider 失败或无来源时仍返回 `status=deferred`，`output` 内带 `adapter_status=adapter_required/degraded` 和 `degraded_reason`。
- 当搜索类 `skill_result` 同时满足 `status=success`、`execution_mode=adapter`、`output.status=live` 且 `output.sources` 非空时，web-server 会把完整搜索输出沉淀为 `artifactType=search_evidence`、`format=json`、`storageKind=inline` 的工作区交付物，并额外发送 `workspace_artifact_created`。未配置、降级或无来源时不会创建搜索证据 artifact。
- 当 creative / design / data / accounting / service / ops / web / engineering 内容类 `skill_result` 返回 `status=success` 且命中支持名单时，web-server 会把输出沉淀为 markdown 工作区交付物，并额外发送 `workspace_artifact_created`。当前 creative 支持：`creative_video_script`、`creative_seeding_copy`、`creative_ip_branding`、`creative_content_calendar`、`creative_live_script`、`creative_product_article`、`creative_trend_catch`、`creative_title_ctr_scorer`。当前 design 支持：`design_main_image`、`design_detail_page`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster`。当前 data 支持：`data_dashboard`、`data_funnel_analysis`、`data_channel_roi`、`data_refund_decomposition`、`data_anomaly_diagnosis`、`data_trend_forecast`、`data_customer_segmentation`、`data_competitor_analysis`、`data_ab_test_analyzer`、`data_multi_period_trend`、`data_ltv_calculator`、`data_cohort_analysis`、`data_seasonal_decompose`、`data_price_elasticity`、`data_demand_forecast`。当前 accounting 支持：`accounting_cost_calc`、`accounting_profit_analysis`、`accounting_roi_calc`、`accounting_budget_plan`。当前 service 支持：`service_ticket_handler`、`service_faq_playbook`、`service_escalation_flow`、`service_dsr_improvement`、`service_return_handler`、`service_nps_analyzer`、`service_sentiment_analyzer`、`service_nps_driver_analysis`。当前 ops 支持：`ops_promo_planning`、`ops_pricing_strategy`、`ops_channel_strategy`、`ops_inventory_planning`、`ops_listing_copy`、`ops_assortment_planning`、`ops_execution_plan`、`ops_smart_pricing`、`ops_inventory_optimizer`、`ops_ad_fatigue_detector`、`ops_abc_xyz_classifier`。当前 web 支持：`web_seo_optimize`、`web_title_generator`、`web_page_conversion`、`web_store_design`、`web_keyword_research`、`web_product_description`、`web_title_seo_scorer`。当前 engineering 支持：`engineering_arch_review`、`engineering_bug_analysis`、`engineering_perf_optimize`、`engineering_tech_selection`、`engineering_sla_monitor`。
- 其它需要内容引擎、平台数据或未接 adapter 的能力会返回 `deferred`，不会伪造外部资料或平台执行结果。

`skill_result.status` 当前使用：`success`、`deferred`、`pending_approval`、`not_found`、`validation_error`、`failed`。其中 `pending_approval` 和 `approval_required=true` 表示后端没有直接执行平台写操作，需要进入审批或人工确认链路；`deferred` 表示 skill 已被识别但依赖搜索、外部数据、数据库或平台适配器，当前不会伪造结果。搜索类 skill 在 provider 返回可信来源时会以 `success` 承载 `output.status=live`，provider 不可用时仍以 `deferred` 承载明确降级原因。

`rag_context_loaded` 当前使用本地 RAG：来源只来自当前用户自己的 active `products` 和 active `product_knowledge`。只有本轮显式选择商品时，服务端才会把检索摘要追加到 agent runtime `context.summary`，让模型可在回答中使用这些事实。如果没有选择商品，事件可返回空检索状态，但不会向模型注入商品知识或“默认商品”提示。如果已选择商品但没有命中知识，事件会返回 `degraded=true` 和 `degradedReason="No matching local product knowledge was found; external search adapter is not configured."`。当前不会伪造外部搜索结果，`externalSearchAvailable=false` 表示外部搜索适配器尚未接入。

`workspace_context_loaded` 当前只聚合来源元数据，不读取文件正文，不做 OCR，不伪造素材总结。服务端会把 `agentContext.refs` 追加到 runtime `context.refs`，把 `summary` 追加到 `context.summary`，并写入 `workspaceContext.readiness`、`workspaceContext.counts_json`、`workspaceContext.refs`、`workspaceContext.sources_json`、`workspaceContext.missing_json`、`workspaceContext.warnings_json`。`readiness=empty` 表示没有可用附件、素材或交付物，前端应展示明确空状态；这不是外部搜索或附件解析结果。

`workspace_context_pack_loaded` 会继续把可读正文追加到 runtime `context.summary`，并写入 `workspaceContextPack.readiness`、`workspaceContextPack.counts_json`、`workspaceContextPack.items_json`、`workspaceContextPack.refs`、`workspaceContextPack.omitted_json`、`workspaceContextPack.warnings_json`。可读来源包括 `product_material.content`、`product_asset.description`、`workspace_artifact.content/preview`，以及 `conversation_file.metadata.extractionStatus=extracted` 时的 `extractedText/textContent/plainText/markdown/text/content/preview/summary` 等字段。未成功抽取的附件会返回 `contentKind=metadata_only`、`extraction_pending`、`extraction_failed` 或 `extraction_unsupported`，不会被当成已读取正文。

工作区沉淀事件：

| 事件名 | 说明 |
| --- | --- |
| `workspace_run_linked` | SSE 启动前已创建或绑定 `workspace_run`，返回 `workspace`、`run` 和 `createdWorkspace` |
| `workspace_run_tasks_linked` | `task_graph_planned` 已沉淀为 `workspace_tasks` 和 `workspace_run_tasks`，返回运行任务快照列表 |
| `workspace_run_task_updated` | 节点开始、worker report 或 evaluation 已写入对应运行任务快照 |
| `workspace_run_completed` | agent session 正常完成，`workspace_run.status=succeeded`，最终输出写入 `report.output` |
| `workspace_artifact_created` | 已创建工作区交付物；正常完成时通常为 `artifactType=final_report` 的 markdown 报告，live 搜索证据会创建 `artifactType=search_evidence` 的 json 交付物，creative / design / data / accounting / service / ops / web / engineering 内容 skill 会创建对应 markdown 内容交付物 |
| `agent_learning_record_created` | `coordination_save_memory` 已真实沉淀为学习/工作记忆记录，可在后续工作区问答中复用 |
| `campaign_draft_created` | `coordination_create_campaign` 已真实沉淀为营销活动草稿，后续可在活动管理、工作区或交付物里继续编辑和治理 |
| `quality_checked` | agent session 正常完成并创建交付物后，后端已完成回答质量检查、信任分更新和学习记录沉淀 |
| `approval_required` | `skill_result.status=pending_approval` 或 `approval_required=true` 时，后端已创建或复用工具审批记录，并尝试创建告警 |
| `workspace_run_failed` | agent session 失败，`workspace_run.status=failed`，失败原因写入 `report.reason` |
| `workspace_run_persistence_error` | agent 流运行过程中某次沉淀失败，返回 `action`、`code`、`message`；该事件不截断原始 LLM 回答 |

`workspace_run_linked` 的 `run.source` 固定为 `agent_session`。`workspace_run_tasks_linked` 与 `workspace_run_task_updated` 返回的数据结构等同于 `API_DOCS/app.md` 中的 `WorkspaceRunTaskResponse`，字段使用 camelCase。`workspace_run_completed` / `workspace_run_failed` 返回的数据结构等同于 `WorkspaceRunResponse`。`workspace_artifact_created` 返回的数据结构等同于 `WorkspaceArtifactResponse`。`agent_learning_record_created` 返回的数据结构等同于 `API_DOCS/app.md` 中的 `AgentLearningRecordResponse`，字段使用 camelCase。`campaign_draft_created` 返回的数据结构等同于 `API_DOCS/app.md` 中 `POST /api/app/campaigns/draft` 的成功响应，字段使用 camelCase。搜索证据 artifact 的 `content` 是完整搜索输出 JSON 字符串，`metadata.kind=agent_search_evidence`，并包含 `skillId`、`requestId`、`role`、`query`、`sourceCount`、`domains`、`confidence`、`provenance` 和 `freshness`。creative / design / data / accounting / service / ops / web 内容 artifact 的 `metadata.kind=agent_generated_content`，并包含 `contentCategory=creative/design/data/accounting/service/ops/web`、`artifactType`、`skillId`、`requestId`、`role`、`summary`。`creative_video_script`、`creative_seeding_copy`、`creative_ip_branding`、`creative_content_calendar`、`creative_live_script`、`creative_product_article`、`creative_trend_catch`、`creative_title_ctr_scorer` 成功输出会分别创建脚本、种草文案、IP定位、内容排期、直播脚本、产品软文、趋势报告和标题CTR评分交付物；使用默认商品、排期或直播参数时会以 `data_readiness=calculated_from_defaults` 明示，缺少必填参数时返回 validation error，不会创建假创意交付物。`design_main_image`、`design_detail_page`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster` 成功输出会分别创建主图 brief、详情页 brief、配色方案、素材规范和活动海报 brief 交付物；主图、详情页或素材规范使用默认商品/平台规则时会以 `data_readiness=calculated_from_defaults` 明示，本阶段不生成真实图片文件，缺少必填参数时返回 validation error。`data_dashboard` 交付物会保留 skill 输出中的 `has_data` 和 `adapter_status`，前端应据此区分人工入参看板与缺少真实平台数据的降级看板。`data_funnel_analysis`、`data_channel_roi`、`data_refund_decomposition` 缺少用户提供数据时会返回 `deferred`，不会创建假报告 artifact。`data_anomaly_diagnosis`、`data_trend_forecast`、`data_customer_segmentation`、`data_competitor_analysis`、`data_ab_test_analyzer` 成功输出会分别创建异常诊断、趋势预测、客户分层、竞品分析和 A/B 测试报告；异常/趋势缺少必要时序数据时返回 `deferred`，客户分层或竞品分析使用默认估算时会以 `data_readiness=calculated_from_defaults` 明示。`data_multi_period_trend`、`data_ltv_calculator`、`data_cohort_analysis`、`data_seasonal_decompose`、`data_price_elasticity`、`data_demand_forecast` 成功输出会分别创建多周期趋势、LTV、队列留存、季节性分解、价格弹性和需求预测报告；多周期趋势、季节性分解或需求预测缺少必要时序数据时返回 `deferred`，LTV、队列留存或价格弹性使用默认估算时会以 `data_readiness=calculated_from_defaults` 明示。`accounting_profit_analysis`、`accounting_roi_calc` 缺少必要财务输入时会返回 `deferred`，不会创建假财务报告；`accounting_cost_calc`、`accounting_budget_plan` 如使用默认参数会在输出中以 `data_readiness=calculated_from_defaults` 明示。`service_ticket_handler`、`service_faq_playbook`、`service_escalation_flow` 成功输出会分别创建客服工单、FAQ、升级流程 playbook；`service_dsr_improvement` 如使用默认样例评分会在输出中以 `data_readiness=calculated_from_defaults` 明示。`service_return_handler`、`service_nps_analyzer`、`service_sentiment_analyzer`、`service_nps_driver_analysis` 成功输出会分别创建退换货处理、NPS/CSAT、情绪分析、满意度驱动因素报告；NPS/驱动因素如使用默认样例数据会在输出中以 `data_readiness=calculated_from_defaults` 明示。`ops_promo_planning`、`ops_pricing_strategy`、`ops_channel_strategy`、`ops_inventory_planning`、`ops_listing_copy`、`ops_assortment_planning`、`ops_execution_plan` 成功输出会分别创建运营计划类交付物；`ops_smart_pricing`、`ops_inventory_optimizer`、`ops_ad_fatigue_detector`、`ops_abc_xyz_classifier` 成功输出会分别创建智能定价、库存优化、广告疲劳、ABC-XYZ库存分类报告；缺少必要用户输入时返回 `deferred`，不会创建假运营交付物。智能定价缺少竞品价或 ABC-XYZ 使用演示 SKU 时会以 `data_readiness=calculated_from_defaults` 明示。`web_seo_optimize`、`web_title_generator`、`web_page_conversion`、`web_store_design`、`web_keyword_research`、`web_product_description`、`web_title_seo_scorer` 成功输出会分别创建 SEO 方案、标题候选、页面转化、店铺装修、关键词矩阵、详情页文案和标题 SEO 评分交付物；使用平台规则、通用主推商品或页面基准默认值时会以 `data_readiness=calculated_from_defaults` 明示，缺少必填参数时返回 validation error，不会创建假 Web 增长交付物。`quality_checked` 返回的数据结构等同于 `API_DOCS/app.md` 中 `POST /api/app/workspaces/{workspaceId}/runs/{runId}/quality-checks` 的成功响应，字段使用 camelCase。

补充：engineering 内容 artifact 也使用 `metadata.kind=agent_generated_content`，`metadata.contentCategory=engineering`。`engineering_arch_review`、`engineering_bug_analysis`、`engineering_perf_optimize`、`engineering_tech_selection`、`engineering_sla_monitor` 成功输出会分别创建架构评估、Bug排查、性能优化、技术选型和SLA监控报告；当前基于用户输入和内置工程规则生成，不读取真实代码仓库、日志、APM、监控系统或实时外部 benchmark，缺少关键上下文时会以 `data_readiness=calculated_from_defaults` 明示；`engineering_system_check` 仍需平台 adapter 和审批，不会创建假平台健康检查交付物。

`quality_checked` 当前由 web-server 在 `workspace_run_completed` 和 `workspace_artifact_created` 后自动触发。质量检查失败不会截断 LLM 回答；如果质量检查自身写入失败，服务端会继续发 `workspace_run_persistence_error`，其中 `action` 为 `evaluate_agent_run`。

`approval_required` 当前由 web-server 在收到高风险 `skill_result` 后自动触发。服务端会复用现有 `tool_approval_queue`、`audit_logs`、`alerts`，不新增第二套审批表。审批记录用 `actionKey` 去重，告警用 `dedupKey` 去重；重复的同一平台写操作会返回既有 active 记录。Agent 风险等级会映射为业务审批等级：`low -> L1`、`medium -> L2`、`high -> L3`。平台写类审批的 `approval.result.executionReadiness` 会前置返回真实执行可行性诊断，当前可能为 `validation_error`、`connection_missing`、`connection_disabled`、`unsupported_action` 或 `adapter_not_implemented`；当前真实平台 adapter 尚未实现，因此不会把已连接平台动作标记为 `ready`。如果审批写入失败，服务端会继续发 `workspace_run_persistence_error`，其中 `action` 为 `create_tool_approval`；如果审计或告警写入失败，`action` 分别为 `create_tool_approval_audit`、`create_approval_alert`。

兼容文本事件：

```text
event: delta
data: {"content":"Agent session 是..."}

event: done
data: {}
```

失败事件：

```text
event: error
data: {"message":"agent failed"}
```

前端消费规则：

- 新前端应监听所有原始过程事件，用于展示 agent、worker、任务图和任务节点过程
- 新前端应监听工作区沉淀事件，用于展示“本轮回答已保存到哪个工作区、哪些岗位任务已沉淀、最终报告和交付物是否可刷新后查看”
- 文本回复仍然监听 `delta`，把 `data.content` 追加到当前回复文本
- 收到 `delta` 后把 `data.content` 追加到当前回复文本
- 收到 `done` 后结束本次回复
- 收到 `error` 后展示错误信息并结束本次回复
- 前端使用 `POST + fetch stream` 调用，不使用 `GET EventSource`

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 当前 agent runtime 如果只产生最终 `Output`，服务端会把最终内容作为一个 `delta` 发出
- 后续 agent 支持 token 级 `LlmChunk` 后，服务端会把每个 chunk 映射为 `delta`
