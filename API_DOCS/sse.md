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

`metadata.context.*` 只作为引用入口，不允许前端直接塞商品事实或附件正文。后端会通过 service 层读取当前用户自己的 active 商品、active 商品知识、同一会话历史消息、会话附件和工作区交付物，压缩成 agent runtime metadata：`context.product_ids`、`context.refs`、`context.selected_summary`、`context.combined_summary`、`context.sources_json`、`conversation.*`、`workspaceContext.*` 等。商品、会话、工作区不存在、已归档、不属于当前用户或 ID 非法时返回对应业务错误码。商品上下文是显式选择能力：只有传入 `metadata.context.productIds` 或 `metadata.context.contextRefs` 中的 `product:{id}` 时，后端才会注入对应商品资料；如果本轮没有选择商品，即使用户问题包含“咖啡”等关键词，也不会自动把某个已有商品或商品知识注入模型上下文。

`context.selected_summary` 是后端唯一认可的“本轮手动选中商品”来源。worker 自动补偿内容、上架、详情、设计、关键词、促销、渠道等商品型 skill input 时，只会从 `context.selected_summary` 解析商品名称、平台、类目、卖点和商品引用；`conversation.history_summary`、`context.summary`、`context.combined_summary`、`rag.summary`、`workspaceContext.summary` 只能作为普通上下文，不会被当作本轮选品。未选择商品时，相关 skill 会使用本轮用户问题中的普通文本或返回缺数据 / 默认假设状态，不会把历史商品、预设商品或测试商品作为默认指定商品。

商品型 skill input 的新产品商品引用使用 `product_ref` 或 `product_refs`，值为 `product:{uuid}`。旧字段 `product_id` 仍保留为旧整数参数兼容字段，后端不会把 UUID 写入 `product_id`。

当 `metadata.context.conversationId` 存在时，服务端会使用当前登录用户身份读取该会话最近的 user/assistant 消息，并把同一会话历史压缩为 `conversation.history_summary` 后追加到 `context.combined_summary`。当前这一次 `input` 会从历史摘要中排除，避免重复；历史摘要只用于“同一对话连续追问和补充信息”场景，例如用户先说明咖啡商品、后续补充价格、规格、促销活动时，Agent 能在下一轮继续使用这些补充信息。该读取仍受会话归属校验限制，不会跨用户、跨账号或跨会话注入历史。

`metadata.model.name` 必须是 `GET /api/app/agent/runtime-options` 返回的 `models[].name`。如果模型名不存在，服务端返回 `400`，不会启动 Agent，也不会创建工作区运行记录。后端只把模型配置名和模型名写入 runtime metadata，不向前端暴露 `base_url` 或 `api_key`。

`metadata.strategy.*` 仅允许小写字母、数字、`_`、`-`，单字段最大 64 字符。后端会把这些字段转换为 agent runtime metadata：`strategy.id`、`strategy.response_mode`、`strategy.collaboration_mode`、`strategy.learning_level`、`strategy.primary_role`、`strategy.selected_roles`。前端不得把策略说明拼进 `input` 来伪造后端策略行为。

当 `metadata.strategy.collaborationMode` 为 `manual` 或 `single`，且 `primaryRole` 或 `selectedRoles` 中包含非 `chat` 岗位时，后端会优先尊重这次显式岗位选择并进入 commander/任务图路由；即使用户输入文本本身像“只输出/answer only”这类短答请求，也不会被 `chat_bypass` 吞掉。没有显式非 `chat` 岗位选择的短答请求仍可走 `route_decided.route_mode=chat_bypass`，用于普通轻问答。

每次 SSE Agent 调用都会先在 service 层绑定工作区运行记录：如果 `workspaceId` 合法且属于当前用户，则写入该工作区；如果不传 `workspaceId`，后端会创建或复用当前用户的默认 `agent_chat` 工作区。绑定成功后，agent runtime metadata 会追加 `workspace.workspace_id`、`workspace.run_id` 和 `workspace.source=agent_session`。随后后端会调用 `WorkspaceApi::context_summary` 和 `WorkspaceApi::context_pack`，但 Agent SSE 内部只按本次新建的 `workspace_run.id` 聚合工作区 artifact，避免未选择商品时把同一工作区历史交付物自动注入模型。会话附件仍只跟随本轮 `conversationId`，商品资产和商品素材仍只跟随本轮显式选择的 `productIds/contextRefs`。后端会发送 `workspace_context_loaded` 与 `workspace_context_pack_loaded` 事件，并把这两个事件中的本轮上下文 metadata 注入 Agent runtime。Agent SSE 场景下，工作区绑定过商品也不会被当成本轮默认商品；必须由本轮 `metadata.context.productIds/contextRefs` 显式选择后，商品资产和商品素材才会进入本轮上下文。如果绑定或上下文归属校验失败，SSE 不会启动，接口直接返回业务错误，避免前端误以为本次结果已沉淀或已读取资料。

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
data: {"type":"rag_context_loaded","mode":"local_product_knowledge","queryTerms":["咖啡","转化"],"productIds":["product-uuid"],"knowledgeCount":3,"sourceCount":4,"summary":"Local commerce RAG context...","sources":[{"sourceType":"product_knowledge","id":"knowledge-uuid","productId":"product-uuid","title":"核心卖点"}],"degraded":false,"degradedReason":null,"externalSearchAvailable":true}

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
- `coordination_create_campaign` 在返回 `status=success` 且 `campaign_creation.should_persist=true` 时，web-server 会把结构化草稿沉淀成真实 `campaign draft`，并额外发送 `campaign_draft_created` 与对应 `workspace_artifact_created`。该 skill 支持 `product_ref=product:{uuid}` 作为新产品商品引用；未显式传入时可从本轮 `context.refs/context.product_ids` 推断，并继续回填旧 `product_id` 字符串兼容现有草稿持久化链路。
- `coordination_skill_chain_planner` 通过既有 `skill_result` 返回结构化技能链规划；`output` 包含 `status=success`、`角色`、`动作`、`推荐执行链`、`前置依赖说明`、`前置依赖缺口`、`不可用技能`、`审批提示`、`adapter提示` 和 `可用技能过滤`。每个步骤会标明 `技能`、`阶段`、`负责角色`、`执行模式`、`风险等级`、`前置依赖` 和 `可用`，供前端过程面板展示真实计划状态；它只做规划，不执行平台写操作，不调用外部 Agent，也不把 adapter-required 步骤伪装成已完成。
- `query_store_metrics` 通过既有 `skill_result` 返回店铺指标查询结果；当本轮 skill input 明确提供 `gmv/revenue`、`orders/order_count`、`visitors/uv`、`clicks`、`ad_spend/ad_cost`、`refund_amount/refunds`、`gross_profit/profit` 或 `platforms[]` 时，返回 `status=success`、`adapter_status=manual_input`、`data_readiness=explicit_metrics`，并包含 `totals`、`derived_metrics`、`week_over_week`、`anomalies` 和 `platforms`。当没有显式指标且未接 `store_metrics` adapter 时，返回 `status=deferred`、`adapter_status=store_metrics_required`、`data_readiness=missing_user_input`；前端应展示为缺数据/待接入状态，不得当作淘宝、抖音或其它平台实时读取结果。
- 当用户自然语言明确表达“查询/加载/读取店铺指标、店铺数据、经营数据基线”等意图时，worker autosuggest 可自动产生 `skill_requested.skill_id=query_store_metrics`；如果文本中包含 `GMV18万、订单1800、UV3.6万、广告费3.6万` 等显式指标，后端会把这些值解析进 skill input。未明确平台时使用 `platform=all`，不会默认淘宝或其它平台；更具体的“店铺大盘/看板、竞品、退款、趋势、渠道ROI”等意图仍优先进入对应专门 skill。
- `query_product_metrics` 通过既有 `skill_result` 返回商品 / SKU 指标查询结果；当本轮 skill input 明确提供 `product_ref/product_id/product_name/sku_id` 和 `gmv/revenue`、`orders/order_count`、`visitors/uv`、`clicks/click_count`、`ad_spend/ad_cost`、`refund_amount/refunds`、`gross_profit/profit`、`inventory/stock` 或 `stockout_count` 等指标时，返回 `status=success`、`adapter_status=manual_input`、`data_readiness=explicit_product_metrics`，并在 `query.product_ref` 回显本轮显式商品引用，包含 `query`、`totals`、`derived_metrics`、`week_over_week`、`risks` 和 `next_actions`。当没有显式商品指标且未接 `product_metrics` adapter 时，返回 `status=deferred`、`adapter_status=product_metrics_required`、`data_readiness=missing_user_input`；前端应展示为缺商品指标 / 待接入状态，不得把未选择商品、预设商品或历史商品当作默认上下文。
- `query_campaign_metrics` 通过既有 `skill_result` 返回营销活动 / 投放活动指标查询结果；当本轮 skill input 明确提供 `campaign_id/campaign_name/campaign_type` 和 `budget`、`spend/ad_spend`、`gmv/revenue`、`orders/order_count`、`visitors/uv`、`clicks/click_count`、`impressions/exposure`、`coupon_cost` 或 `refund_amount/refunds` 等指标时，返回 `status=success`、`adapter_status=manual_input`、`data_readiness=explicit_campaign_metrics`，并包含 `query`、`totals`、`derived_metrics`、`week_over_week`、`risks` 和 `next_actions`。当没有显式活动指标且未接 `campaign_metrics` adapter 时，返回 `status=deferred`、`adapter_status=campaign_metrics_required`、`data_readiness=missing_user_input`；前端应展示为缺活动指标 / 待接入状态，不得伪造淘宝、抖音、千川或其它平台实时活动数据。
- 当用户自然语言明确表达“查询/加载/读取商品指标、商品数据、单品数据、SKU指标、商品表现”时，worker autosuggest 可自动产生 `skill_requested.skill_id=query_product_metrics`；如果本轮存在用户手动选择的商品，input 会携带 `product_ref=product:{uuid}` 和商品名，未选商品时不会留下空 `product_ref`。当用户明确表达“查询/加载/读取活动指标、活动数据、活动复盘、活动表现、投放活动数据”时，worker autosuggest 可自动产生 `skill_requested.skill_id=query_campaign_metrics`。后端会解析 GMV、订单、UV、点击、广告费、退款、毛利、库存、缺货次数、预算、活动花费、曝光、优惠券成本等显式指标；没有显式指标时仍只返回缺数据状态，不伪造平台实时读取。
- `data_channel_roi` 和 `data_refund_decomposition` 现在按 Agent runtime native skill 处理；当本轮 skill input 明确提供 `channels[]` 或 `items[]` 时返回 `status=success`、`execution_mode=native`、`risk_level=low`，成功输出仍可分别沉淀为 `data_channel_roi_report` 和 `data_refund_report`。二者都支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显；未选商品时 worker 不会写空引用。缺少渠道或退款项数据时仍返回 `status=deferred`，不会创建假报告，也不会把未接入的淘宝、抖音、千川等平台数据当作来源。
- 当用户自然语言明确表达渠道 ROI / 投放投产复盘，并在同一轮文本里写出 `抖音GMV12万，广告费3万，订单600，UV2万`、`淘宝GMV8万，广告费2.6万` 等渠道指标时，worker autosuggest 可把这些值解析成 `data_channel_roi.input.channels[]`。当用户表达退款帕累托 / 退款拆解，并写出 `质量问题退款2.4万`、`物流破损退款9000`、`色差退款4000` 等退款项时，worker autosuggest 可解析成 `data_refund_decomposition.input.items[]`。这些解析只来自本轮显式用户输入，不读取默认商品、历史商品或平台实时数据。
- `data_attribution_analysis` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发 `渠道归因` / `多触点归因` / `归因模型` 等意图时，会携带 `product_ref`；未选商品时不会写空引用，也不会把历史商品当作默认商品。`渠道ROI` / `渠道投产` 仍触发 `data_channel_roi`，避免把 ROI 复盘和多模型归因混在一起。缺少多渠道输入、少于两个有效渠道或总 GMV 为 0 时仍返回 `deferred`，不会创建假归因报告。
- `data_dashboard` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发 `dashboard` / `看板` / `经营概览` / `经营指标` / `店铺大盘` 等意图时，会携带 `product_ref`，并解析本轮显式 `GMV`、订单、UV、点击、广告费、退款、毛利、目标 GMV 等经营指标；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。未提供经营指标时仍以 `adapter_status=store_metrics_required` 和 sample-only 状态明示，不会伪造淘宝、抖音、千川、CRM、ERP、WMS 或实时店铺数据。
- `data_comprehensive_diagnosis` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发 `综合诊断` / `全面诊断` / `业务诊断` / `经营体检` / `全链路诊断` 等意图时，会携带 `product_ref`，并解析本轮显式经营指标、渠道指标、退款项和数字序列，编排经营看板、漏斗、异常、客户、退款、预测、归因和 LTV 分项；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。未提供足够业务输入时仍以 `adapter_status=user_input_required`、`data_readiness=missing_user_input` 或分项 `deferred` 明示，不会伪造淘宝、抖音、千川、CRM、ERP、WMS、DMP、会员系统或实时数仓数据。
- `data_ltv_calculator` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发 `LTV` / `客户价值` / `生命周期价值` / `获客成本回收` 等意图时，会携带 `product_ref`，并解析本轮显式 `CAC`、客单价、月购买频次、毛利率、月留存率等输入；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。未提供 LTV 参数时仍按 deterministic 默认估算并以 `data_readiness=calculated_from_defaults` 明示。
- `data_cohort_analysis` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发 `队列留存` / `同期群` / `留存分析` / `复购留存` / `新客留存` / `流失节点` 等意图时，会携带 `product_ref`，并解析本轮显式 `首期新客` 等输入；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。未提供 `new_customers` 时仍按 deterministic 默认估算并以 `data_readiness=calculated_from_defaults` 明示，本阶段不读取会员系统、订单系统或平台实时留存数据。
- `data_seasonal_decompose` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发 `季节性` / `周期性` / `周期分解` / `季节分解` / `周内波动` / `周期波动` 等意图时，会携带 `product_ref`，并把本轮显式数字序列解析成 `values`；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。少于 `period * 2` 个历史数据点时仍返回 `deferred`，不会创建假季节性报告，也不会伪造平台历史销售或实时趋势。
- `data_price_elasticity` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发 `价格弹性` / `价格敏感` / `需求弹性` / `调价测试` / `涨价影响` / `降价影响` 等意图时，会携带 `product_ref`，并解析本轮显式 `价格-销量` 点、成本和当前售价；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。未提供 `price_points` 时仍按 deterministic 默认估算并以 `data_readiness=calculated_from_defaults` 明示，本阶段不执行真实调价。
- `data_demand_forecast` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发 `需求预测` / `销量预测` / `订单预测` / `备货预测` / `补货预测` / `库存预测` 等意图时，会携带 `product_ref`，并把本轮显式数字序列解析成 `values`；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。少于 3 个历史数据点时仍返回 `deferred`，不会创建假需求预测报告，也不会生成真实补货或平台写操作。
- `data_multi_period_trend` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发 `多周期趋势` / `多周期环比` / `多周期对比` / `环比趋势` / `同比环比` / `移动平均` / `趋势动能` 等意图时，会携带 `product_ref`，并把本轮显式数字序列解析成 `values`；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。少于 4 个历史数据点时仍返回 `deferred`，不会创建假多周期趋势报告，也不会伪造平台历史销售、行业周期或活动影响数据。
- `data_trend_forecast` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发普通 `趋势` / `预测` 问法并路由到 `data_trend_forecast` 时，会携带 `product_ref`，并把本轮显式数字序列解析成 `values`；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。少于 2 个历史数据点时仍返回 `deferred`，不会创建假趋势预测报告，也不会伪造平台历史销售、行业周期或活动影响数据。
- `data_anomaly_diagnosis` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发普通 `异常` / `波动` / `下滑` / `暴涨` / `暴跌` 问法并路由到 `data_anomaly_diagnosis` 时，会携带 `product_ref`，并把本轮显式数字序列解析成 `values`；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。少于 3 个历史数据点时仍返回 `deferred`，不会创建假异常诊断报告，也不会伪造平台历史销售、实时告警或活动影响数据。
- `data_customer_segmentation` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发普通 `客户` / `人群` / `RFM` / `分层` 问法并路由到 `data_customer_segmentation` 时，会携带 `product_ref`，并解析本轮显式 `客户总数`、`客单价`、`人均订单数`、`平均购买间隔` 等 RFM 聚合指标；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。未提供 RFM 聚合输入时仍按 deterministic 默认估算并以 `data_readiness=calculated_from_defaults` 明示，本阶段不读取会员系统、订单系统、CRM、DMP 或平台实时人群数据。
- `data_ab_test_analyzer` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发普通 `A/B` / `AB测试` / `对照组` / `实验组` / `转化对比` / `推全` 问法并路由到 `data_ab_test_analyzer` 时，会携带 `product_ref`，并解析本轮显式对照组/实验组访客数和转化数；未选商品时不会写空引用，也不会把历史商品或预设商品当作默认商品。缺少对照组/实验组样本和转化输入时仍按真实参数缺失处理，不会创建假 A/B 测试报告，也不会伪造实验平台、淘宝、抖音、千川、CRM 或 DMP 数据。
- `data_funnel_analysis` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在成功输出 `商品引用` 回显。data worker 在用户手动选择商品后触发转化漏斗 / 曝光点击加购下单付款等意图时，会携带 `product_ref`；未选商品时不会写空引用，也不会把历史商品当作默认商品。缺少曝光/点击等漏斗数据时仍返回 `deferred`，不会创建假漏斗报告。
- `data_competitor_analysis` 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，并在输出 `商品引用` 回显。data worker 在用户手动选择商品后触发竞品/对标/价格带对比意图时，会把选中商品名作为 `our_product`，并携带 `product_ref`；未选商品时不会写空引用，也不会把历史商品当作默认商品。该 native skill 不联网搜索竞品公开资料，真实外部竞品来源仍由 `search_competitor` 或后续 adapter 负责。
- 搜索类 `search_*` skill 会读取后端 `[external_search]` 配置：配置完整且 provider 返回可信来源时，`skill_result` 可返回 `status=success`、`execution_mode=adapter`，`output` 内包含 `status=live`、`sources`、`data.items`、`provenance`、`confidence`、`freshness`、`evidence` 和 `query.bypass_cache`；未配置、provider 失败或无来源时仍返回 `status=deferred`，`output` 内带 `adapter_status=adapter_required/degraded`、空 `evidence` 和 `degraded_reason`。
- 当用户自然语言明确表达“搜索/联网/实时/最新/公开来源/公开资料/真实评价/平台规则/行业基准”等意图时，worker autosuggest 可自动产生 `skill_requested.skill_id=search_trends/search_competitor/search_market_info/search_product_reviews/search_platform_policy/search_industry_benchmarks`。后端会解析 `query`、`platform`、`focus`、`topic`、`policy_type`、`metric`、`days` 和 `_needs_fresh_data` 等输入；普通“竞品分析、趋势预测、关键词研究、创意趋势捕捉”等本地 native 意图仍优先进入对应专门 skill，不会被搜索 skill 抢走。
- 搜索类 skill 会兼容旧产品 `_needs_fresh_data` 语义，也接受 `bypass_cache` / `bypassCache`，并把结果写入 `query.bypass_cache` 后转发给 provider；这只表示请求实时性偏好，不代表 provider 一定绕过缓存。
- 当搜索类 `skill_result` 同时满足 `status=success`、`execution_mode=adapter`、`output.status=live` 且 `output.sources` 非空时，web-server 会把完整搜索输出沉淀为 `artifactType=search_evidence`、`format=json`、`storageKind=inline` 的工作区交付物，并额外发送 `workspace_artifact_created`。未配置、降级或无来源时不会创建搜索证据 artifact。
- `service_query_product` 通过既有 `skill_result` 读取本轮显式注入的商品上下文。若用户已在本轮选择/引用商品且 `product_ref=product:{uuid}`、旧 `product_id` 或商品名筛选命中，返回 `status=success`、`execution_mode=native`、`output.adapter_status=context_metadata`、`output.data_readiness=explicit_product_context`；若未选择商品或商品引用 / ID / 名称筛选不命中，返回 `status=deferred`，不会默认注入或猜测商品。客服岗位自然语言表达“查询商品 / 商品信息 / 商品资料 / 引用商品”时，worker 可自动产生该 skill；已显式选品时 input 会携带 `product_ref`，未选品时不会留下空引用字段。
- `service_ticket_handler`、`service_faq_playbook`、`service_escalation_flow`、`service_return_handler` 支持 `product_ref=product:{uuid}` 作为本轮显式选中商品的关联字段。worker 只会在 `context.selected_summary` 存在时给这些客服支持类 skill 自动补偿 `product_ref`；未选商品时不会写入空引用，也不会把历史商品或默认商品塞入客服处理流程。
- `creative_video_script`、`creative_seeding_copy`、`creative_product_article`、`creative_live_script`、`ops_listing_copy`、`ops_promo_planning`、`ops_pricing_strategy`、`ops_channel_strategy`、`ops_inventory_planning`、`ops_assortment_planning`、`ops_smart_pricing`、`ops_inventory_optimizer`、`ops_ad_fatigue_detector`、`web_keyword_research`、`web_title_generator`、`web_product_description`、`design_main_image`、`design_detail_page` 等商品型自动补偿会优先消费本轮 `context.selected_summary`。如果用户手动选择了商品，skill input 中的 `product_name` / `product` / `product_type` / `category` / `product_ref` / `product_refs` / `platform` / `features` / `selling_points` 会来自选中商品和知识；如果没有手动选择商品，这些字段不会从历史摘要或默认商品推断，也不会留下空的 `product_ref/product_refs`。
- 当 creative / design / data / accounting / service / ops / web / engineering 内容类 `skill_result` 返回 `status=success` 且命中支持名单时，web-server 会把输出沉淀为 markdown 工作区交付物，并额外发送 `workspace_artifact_created`。当前 creative 支持：`creative_video_script`、`creative_seeding_copy`、`creative_ip_branding`、`creative_content_calendar`、`creative_live_script`、`creative_product_article`、`creative_trend_catch`、`creative_title_ctr_scorer`。当前 design 支持：`design_main_image`、`design_detail_page`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster`。当前 data 支持：`data_dashboard`、`data_funnel_analysis`、`data_channel_roi`、`data_refund_decomposition`、`data_anomaly_diagnosis`、`data_trend_forecast`、`data_customer_segmentation`、`data_competitor_analysis`、`data_ab_test_analyzer`、`data_attribution_analysis`、`data_multi_period_trend`、`data_ltv_calculator`、`data_cohort_analysis`、`data_seasonal_decompose`、`data_price_elasticity`、`data_demand_forecast`、`data_comprehensive_diagnosis`、`query_store_metrics`、`query_product_metrics`、`query_campaign_metrics`。`query_store_metrics` / `query_product_metrics` / `query_campaign_metrics` 只有在本轮显式指标输入并返回 `status=success` 时才分别创建店铺指标报告、商品指标报告和活动指标报告；缺指标的 `deferred` 状态不会创建 artifact。当前 accounting 支持：`accounting_cost_calc`、`accounting_profit_analysis`、`accounting_roi_calc`、`accounting_budget_plan`、`accounting_break_even_calc`、`accounting_compliance_check`、`accounting_pl_statement`、`accounting_budget_vs_actual`、`accounting_gmv_waterfall`、`accounting_scenario_analysis`、`accounting_cash_flow_forecast`、`accounting_financial_narrative`。当前 service 支持：`service_ticket_handler`、`service_faq_playbook`、`service_escalation_flow`、`service_dsr_improvement`、`service_return_handler`、`service_nps_analyzer`、`service_sentiment_analyzer`、`service_nps_driver_analysis`。当前 ops 支持：`ops_promo_planning`、`ops_pricing_strategy`、`ops_channel_strategy`、`ops_inventory_planning`、`ops_listing_copy`、`ops_assortment_planning`、`ops_execution_plan`、`ops_smart_pricing`、`ops_inventory_optimizer`、`ops_ad_fatigue_detector`、`ops_abc_xyz_classifier`。当前 web 支持：`web_seo_optimize`、`web_title_generator`、`web_page_conversion`、`web_store_design`、`web_keyword_research`、`web_product_description`、`web_title_seo_scorer`。当前 engineering 支持：`engineering_arch_review`、`engineering_bug_analysis`、`engineering_perf_optimize`、`engineering_tech_selection`、`engineering_sla_monitor`。
- 其它需要内容引擎、平台数据或未接 adapter 的能力会返回 `deferred`，不会伪造外部资料或平台执行结果。

`skill_result.status` 当前使用：`success`、`deferred`、`pending_approval`、`not_found`、`validation_error`、`failed`。其中 `pending_approval` 和 `approval_required=true` 表示后端没有直接执行平台写操作，需要进入审批或人工确认链路；`deferred` 表示 skill 已被识别但依赖搜索、外部数据、数据库或平台适配器，当前不会伪造结果。搜索类 skill 在 provider 返回可信来源时会以 `success` 承载 `output.status=live`，provider 不可用时仍以 `deferred` 承载明确降级原因。

补充：`query_store_metrics` / `query_product_metrics` / `query_campaign_metrics` 成功输出会通过既有内容 artifact 物化链路分别创建 `artifactType=data_store_metrics_report`、`artifactType=data_product_metrics_report` 和 `artifactType=data_campaign_metrics_report` 的 markdown 工作区交付物；缺店铺、商品或活动指标的 `deferred` 状态不会触发 `workspace_artifact_created`，也不会伪造平台实时指标报告。

`rag_context_loaded` 当前使用本地 RAG：来源只来自当前用户自己的 active `products` 和 active `product_knowledge`。只有本轮显式选择商品时，服务端才会把检索摘要追加到 agent runtime `context.summary`，让模型可在回答中使用这些事实。如果没有选择商品，事件可返回空检索状态，但不会向模型注入商品知识或“默认商品”提示。如果已选择商品但没有命中知识，事件会返回 `degraded=true` 和 `degradedReason`。当后端 `[external_search]` 未完整配置时，`externalSearchAvailable=false`，降级原因为 `No matching local product knowledge was found; external search adapter is not configured.`；当 `[external_search]` 已完整配置时，`externalSearchAvailable=true`，降级原因会说明本地知识未命中，但搜索类 skill 可以在任务需要实时市场资料时请求 live sources。当前不会伪造外部搜索结果；只有同一轮 `search_*` skill 成功返回 `output.status=live` 和真实来源时，前端才应展示 live 外部搜索证据。

`workspace_context_loaded` 当前只聚合来源元数据，不读取文件正文，不做 OCR，不伪造素材总结。普通工作区查询接口可以按请求参数查看历史 workspace artifact；但 SSE Agent 自动注入时只使用本次 run 范围，避免历史报告影响未指定商品的普通问答。服务端会把 `agentContext.refs` 追加到 runtime `context.combined_refs`，把 `summary` 追加到 `context.combined_summary`，并写入 `workspaceContext.readiness`、`workspaceContext.counts_json`、`workspaceContext.refs`、`workspaceContext.sources_json`、`workspaceContext.missing_json`、`workspaceContext.warnings_json`。`readiness=empty` 表示没有可用附件、素材或交付物，前端应展示明确空状态；这不是外部搜索或附件解析结果。

`workspace_context_pack_loaded` 会继续把可读正文追加到 runtime `context.combined_summary`，并写入 `workspaceContextPack.readiness`、`workspaceContextPack.counts_json`、`workspaceContextPack.items_json`、`workspaceContextPack.refs`、`workspaceContextPack.omitted_json`、`workspaceContextPack.warnings_json`。可读来源包括本轮明确 product_id 下的 `product_material.content`、`product_asset.description`、`workspace_artifact.content/preview`，以及 `conversation_file.metadata.extractionStatus=extracted` 时的 `extractedText/textContent/plainText/markdown/text/content/preview/summary` 等字段。未成功抽取的附件会返回 `contentKind=metadata_only`、`extraction_pending`、`extraction_failed` 或 `extraction_unsupported`，不会被当成已读取正文。

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

`workspace_run_linked` 的 `run.source` 固定为 `agent_session`。`workspace_run_tasks_linked` 与 `workspace_run_task_updated` 返回的数据结构等同于 `API_DOCS/app.md` 中的 `WorkspaceRunTaskResponse`，字段使用 camelCase。`workspace_run_completed` / `workspace_run_failed` 返回的数据结构等同于 `WorkspaceRunResponse`。`workspace_artifact_created` 返回的数据结构等同于 `WorkspaceArtifactResponse`。`agent_learning_record_created` 返回的数据结构等同于 `API_DOCS/app.md` 中的 `AgentLearningRecordResponse`，字段使用 camelCase。`campaign_draft_created` 返回的数据结构等同于 `API_DOCS/app.md` 中 `POST /api/app/campaigns/draft` 的成功响应，字段使用 camelCase。搜索证据 artifact 的 `content` 是完整搜索输出 JSON 字符串，`metadata.kind=agent_search_evidence`，并包含 `skillId`、`requestId`、`role`、`query`、`sourceCount`、`domains`、`confidence`、`provenance`、`freshness` 和 `evidence`。creative / design / data / accounting / service / ops / web 内容 artifact 的 `metadata.kind=agent_generated_content`，并包含 `contentCategory=creative/design/data/accounting/service/ops/web`、`artifactType`、`skillId`、`requestId`、`role`、`summary`。`creative_video_script`、`creative_seeding_copy`、`creative_ip_branding`、`creative_content_calendar`、`creative_live_script`、`creative_product_article`、`creative_trend_catch`、`creative_title_ctr_scorer` 成功输出会分别创建脚本、种草文案、IP定位、内容排期、直播脚本、产品软文、趋势报告和标题CTR评分交付物；使用默认商品、排期或直播参数时会以 `data_readiness=calculated_from_defaults` 明示，缺少必填参数时返回 validation error，不会创建假创意交付物。`design_main_image`、`design_detail_page`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster` 成功输出会分别创建主图 brief、详情页 brief、配色方案、素材规范和活动海报 brief 交付物；主图、详情页或素材规范使用默认商品/平台规则时会以 `data_readiness=calculated_from_defaults` 明示，本阶段不生成真实图片文件，缺少必填参数时返回 validation error。`data_dashboard` 交付物会保留 skill 输出中的 `has_data` 和 `adapter_status`，前端应据此区分人工入参看板与缺少真实平台数据的降级看板。`data_funnel_analysis`、`data_channel_roi`、`data_refund_decomposition` 缺少用户提供数据时会返回 `deferred`，不会创建假报告 artifact。`data_anomaly_diagnosis`、`data_trend_forecast`、`data_customer_segmentation`、`data_competitor_analysis`、`data_ab_test_analyzer` 成功输出会分别创建异常诊断、趋势预测、客户分层、竞品分析和 A/B 测试报告；异常/趋势缺少必要时序数据时返回 `deferred`，客户分层或竞品分析使用默认估算时会以 `data_readiness=calculated_from_defaults` 明示。`data_attribution_analysis` 成功输出会创建渠道归因报告；缺少多渠道输入、少于两个有效渠道或总 GMV 为 0 时返回 `deferred`，不会创建假平台渠道报告。`data_multi_period_trend`、`data_ltv_calculator`、`data_cohort_analysis`、`data_seasonal_decompose`、`data_price_elasticity`、`data_demand_forecast` 成功输出会分别创建多周期趋势、LTV、队列留存、季节性分解、价格弹性和需求预测报告；多周期趋势、季节性分解或需求预测缺少必要时序数据时返回 `deferred`，LTV、队列留存或价格弹性使用默认估算时会以 `data_readiness=calculated_from_defaults` 明示。`data_comprehensive_diagnosis` 成功输出会创建综合业务诊断报告；它会组合经营看板、漏斗、异常、客户、退款、预测、归因和 LTV 分项，缺失分项以 `deferred` 和 `缺失数据` 明示，整体没有有效用户数据时不会创建 artifact。`accounting_profit_analysis`、`accounting_roi_calc` 缺少必要财务输入时会返回 `deferred`，不会创建假财务报告；`accounting_cost_calc`、`accounting_budget_plan` 如使用默认参数会在输出中以 `data_readiness=calculated_from_defaults` 明示。`accounting_break_even_calc`、`accounting_compliance_check`、`accounting_pl_statement`、`accounting_budget_vs_actual`、`accounting_gmv_waterfall`、`accounting_scenario_analysis`、`accounting_cash_flow_forecast`、`accounting_financial_narrative` 成功输出会分别创建盈亏平衡、合规检查、P&L、预算执行、GMV瀑布、情景分析、现金流预测和财务健康诊断交付物；缺少必要核心输入时返回 `deferred`，不会创建假财务报告，使用默认假设时会以 `data_readiness=calculated_from_defaults` 明示。`service_ticket_handler`、`service_faq_playbook`、`service_escalation_flow` 成功输出会分别创建客服工单、FAQ、升级流程 playbook；`service_dsr_improvement` 如使用默认样例评分会在输出中以 `data_readiness=calculated_from_defaults` 明示。`service_return_handler`、`service_nps_analyzer`、`service_sentiment_analyzer`、`service_nps_driver_analysis` 成功输出会分别创建退换货处理、NPS/CSAT、情绪分析、满意度驱动因素报告；NPS/驱动因素如使用默认样例数据会以 `data_readiness=calculated_from_defaults` 明示。`ops_promo_planning`、`ops_pricing_strategy`、`ops_channel_strategy`、`ops_inventory_planning`、`ops_listing_copy`、`ops_assortment_planning`、`ops_execution_plan` 成功输出会分别创建运营计划类交付物；`ops_smart_pricing`、`ops_inventory_optimizer`、`ops_ad_fatigue_detector`、`ops_abc_xyz_classifier` 成功输出会分别创建智能定价、库存优化、广告疲劳、ABC-XYZ库存分类报告；缺少必要用户输入时返回 `deferred`，不会创建假运营交付物。促销策划、定价策略、渠道策略、库存规划、选品规划、智能定价、库存优化、广告疲劳检测和 ABC-XYZ 库存分类都支持 `product_ref=product:{uuid}` 记录本轮显式选中商品，成功输出中的 `商品引用` 会回显该引用；未选商品时 worker 不注入空 `product_ref`，直接调用 skill 时该输出字段为 `null`。促销策划可由活动、促销、大促、618、双11、campaign、promotion 等意图触发，并优先解析本轮文本中的 `预算` / `活动预算` / `总预算` / `budget` 标签；定价策略可由定价、价格、售价、成本、price、pricing、cost 等意图触发，并优先解析本轮文本中的 `单位成本` / `单品成本` / `成本` / `cost` 和 `竞品价` / `竞品价格` / `对手价格` 标签；渠道策略可由渠道、投放渠道、流量渠道、渠道分配、渠道策略、channel 等意图触发，并优先解析本轮文本中的 `渠道预算` / `月预算` / `预算` / `budget` 标签；库存规划可由库存、补货、备货、inventory、stock 等意图触发，并优先解析本轮文本中的 `日均销量` / `日均需求` / `日销量` / `avg_daily_demand` 和 `供货周期` / `补货周期` / `交货周期` / `lead_time_days` 标签；选品规划可由货盘、SKU组合、选品组合、组合规划、assortment 等意图触发，并优先解析本轮文本中的 `选品预算` / `货盘预算` / `采购预算` / `预算` / `budget` 标签；`618`、`双11` 等活动名数字不会被当成预算，也不会读取或伪造淘宝、抖音、千川、平台活动、平台渠道、供应链、ERP、WMS、供应商报价、实时竞品价格或实时库存数据。广告疲劳检测可由广告疲劳、素材疲劳、CTR衰减、点击率衰减、曝光频次或 ROAS健康意图触发，并只解析本轮显式渠道 CTR、频次、预算和 ROAS；缺渠道明细时会以 `missing_user_input` / `deferred` 明示，不读取或伪造淘宝、抖音、千川、广告平台或实时数仓数据。ABC-XYZ 库存分类可由 ABC-XYZ、ABC分类、XYZ分类、九宫格库存分类、SKU分层等意图触发，并只解析本轮显式 SKU GMV 和历史销量；缺 SKU 明细时会以 `calculated_from_defaults` 明示使用确定性演示 SKU，不读取或伪造 WMS/ERP/平台实时库存或历史销量。智能定价缺少竞品价时会以 `data_readiness=calculated_from_defaults` 明示；库存优化缺少 SKU 明细时会以 `missing_user_input` / `deferred` 明示，不读取或伪造 WMS/ERP/平台实时库存。`web_seo_optimize`、`web_title_generator`、`web_page_conversion`、`web_store_design`、`web_keyword_research`、`web_product_description`、`web_title_seo_scorer` 成功输出会分别创建 SEO 方案、标题候选、页面转化、店铺装修、关键词矩阵、详情页文案和标题 SEO 评分交付物；使用平台规则、通用主推商品或页面基准默认值时会以 `data_readiness=calculated_from_defaults` 明示，缺少必填参数时返回 validation error，不会创建假 Web 增长交付物。`quality_checked` 返回的数据结构等同于 `API_DOCS/app.md` 中 `POST /api/app/workspaces/{workspaceId}/runs/{runId}/quality-checks` 的成功响应，字段使用 camelCase。

补充：engineering 内容 artifact 也使用 `metadata.kind=agent_generated_content`，`metadata.contentCategory=engineering`。`engineering_arch_review`、`engineering_bug_analysis`、`engineering_perf_optimize`、`engineering_tech_selection`、`engineering_sla_monitor` 成功输出会分别创建架构评估、Bug排查、性能优化、技术选型和SLA监控报告；当前基于用户输入和内置工程规则生成，不读取真实代码仓库、日志、APM、监控系统或实时外部 benchmark，缺少关键上下文时会以 `data_readiness=calculated_from_defaults` 明示。`engineering_system_check` 可由“平台 API 连通性 / 接口健康 / 授权 / 同步状态”等自然语言触发；仅查看配置时返回 `deferred`，要求真实测试连接时返回 `pending_approval`、`execution_mode=adapter`、`risk_level=high`、`approval_required=true`，仍需平台 adapter 和审批，不会创建假平台健康检查交付物。

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
