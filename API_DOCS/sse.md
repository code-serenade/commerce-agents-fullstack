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
      "contextRefs": [
        "product:product-uuid",
        "conversation_file:file-uuid",
        "product_material:material-uuid",
        "workspace_artifact:artifact-uuid"
      ]
    },
    "strategy": {
      "id": "execution",
      "responseMode": "execution",
      "routingMode": "commander",
      "collaborationMode": "manual",
      "learningLevel": "higher_vocational",
      "primaryRole": "data",
      "selectedRoles": ["data", "ops", "creative"]
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
| `metadata.context.contextRefs` | `string[]` | 否 | 上下文引用，当前支持 `product:{productId}`、`conversation_file:{fileId}` / `file:{fileId}`、`product_asset:{assetId}` / `asset:{assetId}`、`product_material:{materialId}` / `material:{materialId}`、`workspace_artifact:{artifactId}` / `artifact:{artifactId}`，最多 20 个 |
| `metadata.attachments.images` | `object[]` | 否 | 本轮传给 vision LLM 的图片输入，最多 4 张 |
| `metadata.attachments.images[].url` | `string` | 是 | 图片 URL，允许 HTTPS URL 或 `data:image/png/jpeg/webp;base64,...`；本地 Ollama 等不能拉外部 URL 的 provider 建议使用 data URL |
| `metadata.attachments.images[].detail` | `string` | 否 | vision detail，允许 `auto`、`low`、`high` |
| `metadata.attachments.images[].sourceRef` | `string` | 否 | 图片来源引用，例如 `conversation_file:{fileId}`，仅作来源标记，不绕过上下文权限校验 |
| `metadata.strategy.id` | `string` | 否 | 回答策略 ID，当前前端使用 `learning` / `quick` / `execution`；历史 `collab` 只作为兼容值处理 |
| `metadata.strategy.responseMode` | `string` | 否 | 回答模式，当前前端使用 `learning` / `quick` / `execution` |
| `metadata.strategy.routingMode` | `string` | 否 | 路由模式，允许 `commander` / `fast`；默认按 `commander` 处理，先由 Commander 分析；`fast` 才允许后端前置配置规则/快速任务图匹配 |
| `metadata.strategy.collaborationMode` | `string` | 否 | 岗位分配模式，当前前端使用 `auto` / `single` / `manual`；`auto` 由后端判断单岗或多岗，`manual` 会要求后端优先进入多角色任务图 |
| `metadata.strategy.learningLevel` | `string` | 否 | 讲解深度，当前前端使用 `beginner`（低） / `higher_vocational`（中） / `professional`（高） |
| `metadata.strategy.primaryRole` | `string` | 否 | 用户指定的主岗位 runtime role，例如 `data`、`ops`、`creative` |
| `metadata.strategy.selectedRoles` | `string[]` | 否 | 用户指定参与本轮协作的岗位 runtime role，最多 6 个，服务端去重 |

`metadata.context.*` 只作为引用入口，不允许前端直接塞商品事实或附件正文。后端会通过 service 层读取当前用户自己的 active 商品、active 商品知识、同一会话历史消息、会话附件、商品资产、商品素材和工作区交付物，压缩成 agent runtime metadata：`context.product_ids`、`context.refs`、`context.selected_summary`、`context.combined_summary`、`context.sources_json`、`conversation.*`、`workspaceContext.*` 等。商品、会话、工作区、附件、商品资产、商品素材或交付物不存在、已归档、不属于当前用户或 ID 非法时返回对应业务错误码。商品上下文是显式选择能力：只有传入 `metadata.context.productIds` 或 `metadata.context.contextRefs` 中的 `product:{id}` 时，后端才会注入对应商品资料；如果本轮没有选择商品，即使用户问题包含“咖啡”等关键词，也不会自动把某个已有商品或商品知识注入模型上下文。历史工作区交付物、历史附件和历史商品资料也是显式选择能力：只有传入 `workspace_artifact:{artifactId}`、`artifact:{artifactId}`、`conversation_file:{fileId}`、`file:{fileId}`、`product_asset:{assetId}`、`asset:{assetId}`、`product_material:{materialId}` 或 `material:{materialId}` 时，SSE 才会读取对应 active 来源；仅传入 `workspaceId` 不会自动读取历史 run 的全部交付物。

`context.selected_summary` 是后端唯一认可的“本轮手动选中商品”来源。worker 自动补偿内容、上架、详情、设计、关键词、促销、渠道等商品型 skill input 时，只会从 `context.selected_summary` 解析商品名称、平台、类目、卖点和商品引用；`conversation.history_summary`、`context.summary`、`context.combined_summary`、`rag.summary`、`workspaceContext.summary` 只能作为普通上下文，不会被当作本轮选品。未选择商品时，相关 skill 会使用本轮用户问题中的普通文本或返回缺数据 / 默认假设状态，不会把历史商品、预设商品或测试商品作为默认指定商品。

商品型 skill input 的新产品商品引用使用 `product_ref` 或 `product_refs`，值为 `product:{uuid}`。旧字段 `product_id` 仍保留为旧整数参数兼容字段，后端不会把 UUID 写入 `product_id`。

当 `metadata.context.conversationId` 存在时，服务端会使用当前登录用户身份读取该会话最近的 user/assistant 消息，并把同一会话历史压缩为 `conversation.history_summary` 后追加到 `context.combined_summary`。当前这一次 `input` 会从历史摘要中排除，避免重复；历史摘要只用于“同一对话连续追问和补充信息”场景，例如用户先说明咖啡商品、后续补充价格、规格、促销活动时，Agent 能在下一轮继续使用这些补充信息。长会话会保留最近窗口，同时从更早历史中按本轮问题挑选少量相关原文片段，并在 `conversation.history_relevant_older_count` 记录数量。服务端还会读取 `conversations.metadata.memorySummary`，合并旧 facts、constraints、decisions 与本轮最近消息，生成 `conversation.memory_summary_json` 并追加 `Same conversation structured memory` 到 `context.combined_summary`。本轮正常完成后，服务端会把更新后的结构化 memory 写回当前会话 metadata，并发送 `conversation_memory_updated` 事件；该读取和写回都受会话归属校验限制，不会跨用户、跨账号或跨会话注入历史。

写回 `memorySummary` 前会做轻量压缩和预算治理：`rollingSummary` 最多 1400 字符，facts 最多 10 条，decisions 最多 6 条，constraints 最多 8 条，openQuestions 最多 5 条，单条记忆文本最多 260 字符，`lastUserInput` 最多 500 字符，`lastAssistantOutput` 最多 700 字符，实际注入到模型的 memory 文本最多 2400 字符。旧 memory 中的超长字段、重复字段、空字段和无关嵌套对象会被清洗；文本完全相同或一条文本明显包含另一条文本时，会保留信息更完整的一条。预算值会写入 `conversation.memory_summary_json.sourceMap.memoryBudget`，方便前端或调试工具解释为什么没有全量带入历史。

Agent SSE 不接受前端选择本轮 LLM；前端不得发送 `metadata.model`，不得展示、缓存或让用户选择模型。服务端始终使用 `backend/config/services.toml` 中的 `default_llm` 和 `[agent_llm_usage]` 路由配置。即使旧请求体里带了模型名，后端也会忽略该字段，不会因为旧前端缓存的模型名不存在而拒绝本轮请求；普通前端接口不暴露模型名称、模型映射、`base_url` 或 `api_key`。

如果上游模型服务返回 401/403、`Forbidden` 或 `AccessDenied`，Agent runtime 会把本轮 worker 标记为结构化失败：`task_node_reported.report.status=failed`，`report.evidence` 包含兼容 marker `failure_kind:llm_access_denied`，`report.diagnostics[]` 包含 `kind=failure`、`fields.failure_kind=llm_access_denied`，`report.openQuestions` 和最终输出会给出“模型服务配置不可用”的用户可理解诊断。该诊断只提示检查服务端 `services.toml` 的 `[agent_llm_usage]`、模型名和 API Key 权限，不会通过 SSE 暴露 `api_key`、provider request id 或完整 provider 原始错误。

如果 task graph 已规划出某个岗位节点，但 commander 找不到该岗位对应的可用 worker 或派发失败，后端不会在 `task_graph_planned.graph.nodes[]` 中静默删除该节点；该节点会产生 `task_node_reported.report.status=failed`，`report.evidence` 包含兼容 marker `failure_kind:worker_unavailable`，`report.diagnostics[]` 包含 `kind=failure`、`fields.failure_kind=worker_unavailable`，随后产生 `task_node_failed`。依赖该节点的下游节点会进入 `skipped`，并写入 `task_graph_finished.process.incomplete[]`。当本轮没有任何已通过节点可用于最终回答时，最终输出会说明“部分岗位没有成功执行”，而不是返回“请补充更明确的问题或关键材料”。

`metadata.attachments.images` 存在时，普通 Agent 会话不会直接使用原始图片输入；原始图片输入仅保留给系统图片预处理流程。外部 HTTPS 图片 URL 不会由 web-server 下载或代理。若图片来自系统附件、商品资产、商品素材或工作区 artifact，前端仍应同步传入 `metadata.context.contextRefs` 或 `metadata.context.conversationId`，让后端完成当前用户、active 状态和归属校验。当前若前端传入 `metadata.context.conversationId`，服务端会自动从该会话 active 附件中筛选 `image/png`、`image/jpeg`、`image/jpg`、`image/webp`，最多 4 张。若附件对应对象已有成功的 `image_preprocess` 分析，服务端会把摘要写入 runtime `attachments.image_summaries_json`，并追加到 `context.combined_summary`，让本轮请求 LLM 时带上“图片内容是什么”的文本上下文。若没有摘要，服务端不会回退为实时 vision 解析，也不会把原始图片 data URL 发送给普通 Agent LLM；前端应等待预处理完成后再发起基于图片内容的提问。预识别摘要仅用于 runtime 模型输入，不会进入 `workspace_context_pack_loaded` 的可读正文。

`metadata.strategy.*` 仅允许小写字母、数字、`_`、`-`，单字段最大 64 字符。后端会把这些字段转换为 agent runtime metadata：`strategy.id`、`strategy.response_mode`、`strategy.routing_mode`、`strategy.collaboration_mode`、`strategy.learning_level`、`strategy.primary_role`、`strategy.selected_roles`。前端不得把策略说明拼进 `input` 来伪造后端策略行为。

`metadata.strategy.id/responseMode` 的三种新产品模式必须真实影响 Agent runtime：`learning` 注入教学表达和术语解释要求，`quick` 注入短链路结论要求并在自动分配时倾向单岗，`execution` 注入完整执行方案、表格清单、岗位分工、风险和验收要求。`metadata.strategy.learningLevel` 会进入 Commander / Worker cognition context，控制讲解深度，不是纯 UI 文案。

`metadata.strategy.routingMode` 只决定路由链路，不决定回答长短：`commander` 会跳过前置 fast route/chat bypass，先进入 Commander cognition 进行决策；`fast` 会先尝试配置任务图和快速路由，未命中时再降级。显式选择 `chat` role 仍可直接进入 chat worker，因为这是用户指定岗位，不属于兜底。

当 `metadata.strategy.collaborationMode` 为 `manual` 或 `single`，且 `primaryRole` 或 `selectedRoles` 中包含非 `chat` 岗位时，后端会优先尊重这次显式岗位选择并进入 commander/任务图路由；即使用户输入文本本身像“只输出/answer only”这类短答请求，也不会被 `chat_bypass` 吞掉。没有显式非 `chat` 岗位选择的短答请求仍可走 `route_decided.route_mode=chat_bypass`，用于普通轻问答。

每次 SSE Agent 调用都会先在 service 层绑定工作区运行记录：如果 `workspaceId` 合法且属于当前用户，则写入该工作区；如果不传 `workspaceId`，后端会创建或复用当前用户的默认 `agent_chat` 工作区。绑定成功后，agent runtime metadata 会追加 `workspace.workspace_id`、`workspace.run_id` 和 `workspace.source=agent_session`。随后后端会调用 `WorkspaceApi::context_summary` 和 `WorkspaceApi::context_pack`，但 Agent SSE 内部默认只按本次新建的 `workspace_run.id` 聚合工作区 artifact，避免未选择商品时把同一工作区历史交付物自动注入模型。若本轮 `metadata.context.contextRefs` 显式传入附件、商品资产、商品素材或 artifact 引用，后端会额外校验当前用户、active 状态和所属资源，并和 `conversationId` / `productId` / 当前 run 范围来源合并去重后进入 `workspace_context_loaded` / `workspace_context_pack_loaded`。SSE 会把本轮 `input` 作为 context pack 的 `query`，长文本来源会优先抽取与本轮问题相关的片段，命中时在 `items[].metadata.relevanceSelection` 中记录片段选择策略；没有命中或文本较短时仍保持顺序截断。CSV/TSV 大表格会生成 `table_profile`，JSON/JSONL 大结构化文本会生成 `json_profile`，都不会默认把完整原文 inline 到模型上下文。显式商品资产 / 商品素材不会反向把所属商品当成本轮“已选商品”；如果需要商品知识和商品型 skill 输入，仍必须通过 `productIds` 或 `product:{uuid}` 显式选择商品。后端会发送 `workspace_context_loaded` 与 `workspace_context_pack_loaded` 事件，并把这两个事件中的本轮上下文 metadata 注入 Agent runtime。Agent SSE 场景下，工作区绑定过商品也不会被当成本轮默认商品；必须由本轮 `metadata.context.productIds/contextRefs` 显式选择后，商品资产和商品素材才会进入本轮上下文。如果绑定或上下文归属校验失败，SSE 不会启动，接口直接返回业务错误，避免前端误以为本次结果已沉淀或已读取资料。

SSE 事件分为三类：

1. 原始 agent 过程事件，事件名来自 `agent::api::SessionEvent::event_name()`。
2. 工作区沉淀事件，由 web-server 调用 service 层写入真实 `workspaces`、`workspace_runs`、`workspace_tasks`、`workspace_run_tasks`、`workspace_artifacts` 后产生。
3. 兼容旧前端的文本事件：`delta` / `done` / `error`。

原始过程事件示例：

```text
event: started
data: {"type":"started","session_id":"session-001"}

event: route_decided
data: {"type":"route_decided","session_id":"session-001","strategy_id":"execution","response_mode":"execution","collaboration_mode":"manual","learning_level":"higher_vocational","route_mode":"task_graph","primary_runtime_role":"data","support_runtime_roles":["ops","creative"]}

event: dispatch_planned
data: {"type":"dispatch_planned","session_id":"session-001","difficulty":"complex","route_mode":"task_graph","primary_runtime_role":"data","support_runtime_roles":["ops","creative"],"selected_skills":["query_store_metrics","data_dashboard","data_funnel_analysis","coordination_skill_chain_planner"],"planned_steps":[{"id":"context","title":"读取上下文","detail":"读取本轮问题、最近 12/18 条同会话历史、早期压缩摘要和 7 个资料信号。"},{"id":"route","title":"判断任务难度和路线","detail":"调度中心选择 task_graph，计划 3 个执行步骤。"},{"id":"answer","title":"汇总回答","detail":"把执行结果整理为学生可直接使用的回答和下一步建议。"}],"context_budget":{"memory_record_count":1,"history_summary_available":true,"recent_history_message_count":12,"history_total_message_count":18,"history_truncated":true,"history_older_summary_available":true,"explicit_product_count":1,"explicit_context_ref_count":2,"workspace_context_ref_count":1,"workspace_pack_item_count":2,"approximate_input_chars":3600},"reasoning_summary":"问题需要拆成 3 个岗位步骤协作，按任务图顺序执行。","student_visible_explanation":"这个问题会拆给 3 个岗位/步骤协作，过程更长但结果会更完整。 本轮历史较长，已读取最近 12/18 条消息并纳入早期压缩摘要，同时读取 7 个资料信号。"}

event: context_loaded
data: {"type":"context_loaded","session_id":"session-001","product_ids":["product-uuid"],"refs":["product:product-uuid"],"product_count":1,"knowledge_count":3,"source_count":4,"summary":"Product source product:...","sources_json":"[...]"}

event: rag_context_loaded
data: {"type":"rag_context_loaded","mode":"local_product_knowledge","queryTerms":["咖啡","转化"],"productIds":["product-uuid"],"knowledgeCount":3,"sourceCount":4,"summary":"Local commerce RAG context...","sources":[{"sourceType":"product_knowledge","id":"knowledge-uuid","productId":"product-uuid","title":"核心卖点"}],"degraded":false,"degradedReason":null,"externalSearchAvailable":true}

event: conversation_context_loaded
data: {"type":"conversation_context_loaded","conversationId":"conversation-uuid","readiness":"ready","messageCount":12,"totalMessageCount":18,"truncated":true,"relevantOlderCount":4,"olderSummaryAvailable":true,"memoryAvailable":true,"previousMemoryAvailable":true,"historyChars":3600,"memoryChars":900,"summary":"Same conversation context loaded: 12/18 recent message(s), truncated=true, relevant older message(s)=4, previous memory=true.","memory":{"kind":"conversation_memory_summary","rollingSummary":"用户正在围绕咖啡商品转化连续追问。","factCount":6,"decisionCount":2,"constraintCount":1,"openQuestionCount":1}}

event: conversation_memory_updated
data: {"type":"conversation_memory_updated","conversationId":"conversation-uuid","memorySummary":{"kind":"conversation_memory_summary","rollingSummary":"用户正在围绕咖啡商品转化连续追问。","facts":[{"source":"conversation_message:12","text":"预算不能超过 3000","confidence":"explicit"}],"decisions":[{"source":"agent_session:session-001","text":"建议优先保留利润率和退款率判断。","confidence":"generated"}],"constraints":[],"openQuestions":[],"lastUserInput":"继续分析利润","lastAssistantOutput":"建议优先保留利润率和退款率判断。","sourceMap":{"messageCount":12,"recentWindow":12,"previousMemoryAvailable":true,"lastSessionId":"session-001","persisted":true,"memoryBudget":{"rollingSummaryMaxChars":1400,"itemMaxChars":260,"factsMaxItems":10,"decisionsMaxItems":6,"constraintsMaxItems":8,"openQuestionsMaxItems":5,"lastUserInputMaxChars":500,"lastAssistantOutputMaxChars":700,"memoryTextMaxChars":2400}}}}

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

`task_node_reported.report` 对应后端 `WorkerReport`。除 `graph_id`、`node_id`、`task_id`、`worker_id`、`role`、`content`、`role_output`、`evidence[]`、`risks[]`、`open_questions[]` 和 `status` 外，搜索等内部核对能力可返回可选 `diagnostics[]`。`diagnostics[]` 中每项包含 `kind`、`source` 和 `fields`；当 `kind=search_evidence` 时，`fields` 可包含 `skill`、`status`、`adapter_status`、`source_state`、`intent`、`strength`、`sources`、`usable_claims`、`gaps`、`confidence`、`query` 和 `gap_preview`；当 `kind=failure` 时，`fields` 可包含 `failure_kind`、`status`、`role`、`node_id`、`graph_id`、`worker_id` 和脱敏后的 `reason`，常见 `failure_kind` 包括 `timeout`、`llm_access_denied`、`worker_unavailable`、`skill_*` 和 `context_pack_*`。该字段用于过程面板、smoke 报告、session memory 和下游岗位上下文判断，不是最终回答正文；前端展示给普通用户时应优先消费 `task_graph_finished.process.check_items[]` 的可读摘要，不要直接展示内部 JSON。

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
| `conversation_context_loaded` | 后端已读取同会话历史、滚动记忆和早期相关片段，并返回消息数量、是否截断、是否使用旧 memory、字符量和结构化记忆计数；该事件用于可观测性，不额外发送完整历史原文 |
| `rag_context_loaded` | 后端已根据用户问题检索当前用户自己的本地商品知识，返回检索模式、关键词、来源、摘要、是否降级和外部搜索可用状态 |
| `workspace_context_loaded` | 后端已聚合当前工作区的会话附件、商品资产、商品素材和工作区交付物，返回 `ready/partial/empty`、来源计数、引用 key、缺失项和警告，并已把摘要注入 agent metadata |
| `workspace_context_pack_loaded` | 后端已把可读上下文内容包注入 Agent metadata；内容只来自数据库已有文本字段或 `extractionStatus=extracted` 的附件抽取文本，不读取 object storage、不做 OCR、不伪造二进制附件正文 |
| `commander_thinking` | commander 正在规划或调度 |
| `route_decided` | 后端已识别策略和路由模式，包含策略、学习层级、实际模型、主角色和支持角色 |
| `dispatch_planned` | 后端已生成本轮学生可见调度计划，包含难度、路线、主/支持 runtime role、计划 Skill、步骤、上下文预算、调度理由摘要和学生可读说明；简单问题也会发送该事件，只是 `planned_steps` 和 `selected_skills` 更少。`context_budget` 会暴露同会话历史是否可用、最近历史消息数、历史总消息数、是否截断、是否包含早期压缩摘要、资料信号数和近似输入字符数 |
| `worker_started` / `worker_finished` | 单个 worker 开始或完成 |
| `skill_requested` | worker 请求调用某个 skill，包含请求 ID、skill ID、角色和原因；请求来源可能是模型显式 `skill_requests`，也可能是后端按岗位/任务自动补偿的代表 skill |
| `skill_result` | skill 已返回执行结果，包含状态、摘要、错误、可选结构化 `output`、执行模式、风险等级和是否需要审批 |
| `collaboration_started` / `collaboration_worker_started` / `collaboration_worker_finished` / `collaboration_finished` | 多 worker 协作过程 |
| `task_graph_planned` / `task_graph_finished` | 任务图规划或完成；`task_graph_finished` 会额外携带 `process` 结构化过程摘要，供前端按本次回答绑定岗位过程 |
| `task_node_ready` / `task_node_started` / `task_node_reported` / `task_node_evaluated` / `task_node_rework_requested` / `task_node_passed` / `task_node_failed` | 任务节点生命周期 |
| `output` | agent 产生最终输出 |
| `failed` | agent session 失败 |
| `finished` | agent session 完成 |

如果单个 worker LLM cognition 超过后端配置的 `[agent_runtime].worker_cognition_timeout_sec` 或 task graph 节点 `timeout_sec`，后端会把该节点的 `task_node_reported.report.status` 标记为 `timed_out`，在 `report.evidence[]` 中保留兼容 marker `failure_kind:timeout`，并在 `report.diagnostics[]` 写入 `kind=failure`、`fields.failure_kind=timeout`；随后发出 `task_node_failed`，并在 `task_graph_finished.process.incomplete[]` 中记录该节点缺口；下游依赖节点会被标记为 skipped。前端应把 `timed_out` 当作失败节点展示，但不要把整轮 SSE 误判为网络断开。

Skill typed diagnostics：

- 当 worker 调用 Skill 后出现失败、待执行、待审批、adapter 未接入、入参校验失败或执行超时，后端会在对应 `task_node_reported.report.diagnostics[]` 写入 `kind=failure`。
- Skill 失败诊断的 `fields.failure_kind` 可能为 `skill_adapter_timeout`、`skill_adapter_unavailable`、`skill_pending_approval`、`skill_not_found`、`skill_validation_error`、`skill_execution_failed` 或 `skill_deferred`。
- 常见字段包括 `fields.skill`、`fields.status`、`fields.request_id`、`fields.reason`、`fields.execution_mode`、`fields.risk_level`、`fields.adapter_status`、`fields.data_readiness`、`fields.approval_required`。
- `task_graph_finished.process.check_items[]` 会把 typed Skill failure diagnostic 提炼成可读“Skill核对”项，例如 `Skill核对：ops / platform_update_price 未完成，状态 deferred，原因 Skill 适配器未接入或当前不可用`。
- 前端普通过程面板优先展示 `process.check_items[]`；只有调试视图才直接展示 `report.diagnostics[]`，且不得把 provider 原始错误、adapter 调试 JSON 或 API key 等敏感信息展示给学生。

Context pack typed diagnostics：

- 当本轮 `workspaceContextPack.readiness` 不是完整 `ready`，或内容包存在 omitted、metadata only、解析中、解析失败、解析不支持等状态，worker 会在 `task_node_reported.report.diagnostics[]` 写入 `kind=failure`。
- 资料包诊断的 `fields.failure_kind` 可能为 `context_pack_extraction_failed`、`context_pack_extraction_unsupported`、`context_pack_extraction_pending`、`context_pack_omitted`、`context_pack_empty` 或 `context_pack_partial`。
- 常见字段包括 `fields.readiness`、`fields.item_count`、`fields.omitted_count`、`fields.warning_count`、`fields.metadata_only_count`、`fields.extraction_pending_count`、`fields.extraction_failed_count`、`fields.extraction_unsupported_count` 和脱敏短 `fields.warning_preview`。
- `task_graph_finished.process.check_items[]` 会把 typed context pack diagnostic 提炼成可读“资料核对”项，例如 `资料核对：ops 的资料包状态 partial，进入上下文 2 项，省略 1 项，解析中 0 项，解析失败 1 项，不支持 0 项，原因 部分资料抽取失败，不能当作已读取正文`。
- 最终回答不能把未解析、未进入上下文或仅元数据的资料说成已读取全文；前端调试视图也不得展示 object key、storage path、内部 parser 原始异常或完整调试 JSON。

`skill_requested` 的触发来源：

- 模型显式输出 `skill_requests` 时，后端优先执行模型请求，不再追加自动补偿。
- 模型没有输出 `skill_requests` 时，后端可根据当前 worker 岗位、任务文本、上下文摘要和上游结果，自动补偿一个代表 skill 请求。
- 自动补偿仍走角色白名单、`SkillExecutorSet`、真实 `skill_result` 和结果合并链路，不绕过权限，也不伪造前端事件。
- 自动补偿不会直接执行高风险平台写操作；平台写操作仍返回 `pending_approval` 并进入审批链路。
- `coordination_create_campaign` 在返回 `status=success` 且 `campaign_creation.should_persist=true` 时，web-server 会把结构化草稿沉淀成真实 `campaign draft`，并额外发送 `campaign_draft_created` 与对应 `workspace_artifact_created`。该 skill 支持 `product_ref=product:{uuid}` 作为新产品商品引用；未显式传入时可从本轮 `context.refs/context.product_ids` 推断，并继续回填旧 `product_id` 字符串兼容现有草稿持久化链路。
- `coordination_status_query` 通过既有 `skill_result` 返回当前会话可见的团队/任务/工作区状态。输出包含 `status=success`、`query_type`、`scope`、`session`、`worker`、`task`、`workspace`、`context`、`system_health`、`blockers`、`observability`，并额外提供前端可直接展示的 `progress_view`、`role_statuses`、`next_actions` 和 `user_visible_summary`。该 skill 只读取当前 Agent `SkillContext` 和 runtime metadata，不直连数据库、不探测平台 adapter、不查看其它用户或其它会话；如果缺少 `session_id`、`task_id`、`run_id` 或工作区上下文，会在 `blockers` / `observability` 中明确说明可见性边界，而不是伪造成全局集群状态。
- `coordination_create_task` 通过既有 `skill_result` 返回 `task_creation` 草案，并在 web-server/service 消费后落成真实 `workspace_task`。`priority` 输入兼容数字、数字字符串和常见优先级文本，例如 `low` / `normal` / `medium` / `important` / `high` / `urgent` / `普通` / `重要` / `紧急`；后端会归一化为工作区任务使用的 `0..10` 数字，并在 `task_creation.priority` 与 `normalized_inputs.priority` 中回显。未知或空优先级按普通任务处理，不会再让多岗位链路因为 `invalid parameter 'priority'` 变成红色失败。
- `task_graph_finished` 保持原事件名不变，并在完成多岗位任务图时返回 `process`。`process` 包含 `graph_id`、`root_task`、`primary`、`roles[]`、`handoffs[]`、`next_actions[]`、`check_items[]`、`deliverables[]`、`risks[]`、`open_questions[]`、`incomplete[]` 和 `panel`。其中 `roles[]` 来自真实任务节点和 worker report，`handoffs[]` 来自 `TaskNode.input_refs`，`check_items[]` 可包含后端优先从 `WorkerReport.diagnostics[]`、兼容从搜索 trace 提炼出的可读“搜索核对”项，用于展示搜索 skill、adapter 状态、意图、搜索强度、来源数、可用结论数、缺口数和置信度；也可包含 typed Skill failure 生成的“Skill核对”、typed context pack failure 生成的“资料核对”，以及 runtime 从 failed/skipped 节点提炼出的可读“岗位核对”项，例如岗位未注册、模型不可用、执行超时或依赖未通过。前端不需要解析 `search_evidence_trace` 原始文本、`search_evidence_packet` JSON、`workspaceContextPack.items_json` 或 `failure_kind:*` 内部 marker 即可展示过程核对。`panel.default_collapsed=true` 且 `panel.anchor=answer_scoped_process`，表示前端应把过程面板绑定到本次回答并默认折叠，用户需要回看时再展开。该字段只汇总当前 Agent runtime 可见过程，不查询其它用户/会话，不伪造外部搜索、平台执行或工作区 artifact。完成后的同一份 `process` 也会进入 `TaskGraphMemorySummary.process`，随 session memory 一起持久化；后续历史对话或工作台回放应优先消费这个结构化记忆，而不是解析 Markdown 正文或复用全局悬浮过程面板。
- `coordination_skill_chain_planner` 通过既有 `skill_result` 返回结构化技能链规划；`output` 包含 `status=success`、`角色`、`动作`、`推荐执行链`、`前置依赖说明`、`前置依赖缺口`、`不可用技能`、`审批提示`、`adapter提示` 和 `可用技能过滤`。每个步骤会标明 `技能`、`阶段`、`负责角色`、`执行模式`、`风险等级`、`前置依赖` 和 `可用`，供前端过程面板展示真实计划状态；它只做规划，不执行平台写操作，不调用外部 Agent，也不把 adapter-required 步骤伪装成已完成。
- `learning_teaching_explain` 通过既有 `skill_result` 返回教学解释结果；当用户自然语言明确表达“教学 / 讲解 / 解释 / 什么是 / 区别 / 为什么 / 怎么做 / 常见误区 / 自检”等意图时，worker autosuggest 可自动产生 `skill_requested.skill_id=learning_teaching_explain`。输出包含 `30秒结论`、`术语白话解释`、`为什么`、`怎么做`、`场景示例`、`常见误区`、`自检问题` 和 `下一步练习`。该 skill 是 native deterministic，不调用平台、不读取实时店铺数据、不执行高风险动作；未显式选品时不会注入历史商品、预设商品或演示商品。如用户要求最新规则、公开来源或实时数据，应另行触发 `search_*` adapter。
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
- `douyin_ads_plan_structure`、`douyin_ads_audience_package`、`douyin_ads_budget_pacing` 和 `douyin_ads_live_funnel_diagnosis` 现在按 Agent runtime native skill 处理，用于千川计划结构、人群包、预算节奏和直播间投放漏斗诊断。`douyin_ads` worker 可从“千川冷启动 / 计划结构 / 人群包 / 定向 / 预算节奏 / ROI / ROAS / 直播间漏斗 / 进房 / 商品点击 / 付款”等自然语言自动产生对应 `skill_requested`。这些 skill 支持 `product_ref=product:{uuid}` 作为本轮显式选品引用，未选商品时不会写空引用，也不会从历史商品或预设商品注入默认商品。计划结构和人群包可基于确定性默认策略生成并用 `data_readiness=calculated_from_defaults` 明示；预算节奏缺少预算或 ROAS / GMV / 消耗时返回 `deferred`；直播漏斗缺少曝光和点击时返回 `deferred`。本阶段不读取或伪造巨量千川、抖音罗盘、直播间或广告后台实时数据。
- 千川投放岗位如果同时包含“最新 / 实时 / 联网 / 公开来源 / 平台规则 / 千川规则 / 行业ROI基准 / 直播转化基准 / 人群趋势”等诉求，worker 会生成可见双 skill 链路：先运行对应 `douyin_ads_*` native skill，再追加 `search_platform_policy(policy_type=algorithm)`、`search_industry_benchmarks(metric=roi)`、`search_trends` 或 `search_market_info`。搜索步骤独立出现在 `skill_requested` / `skill_result` 中，provider 未配置、失败、无可信 URL 来源或来源过期时仍按搜索降级，不把外部基准或平台规则混入 native 输出伪造成已读取。
- 搜索类 `search_*` skill 会读取后端 `[external_search]` 配置：配置完整且 provider 返回可信来源时，`skill_result` 可返回 `status=success`、`execution_mode=adapter`，`output` 内包含 `status=live`、`sources`、`data.items`、`provenance`、`confidence`、`freshness`、`evidence` 和 `query.bypass_cache`；未配置、provider 失败或无来源时仍返回 `status=deferred`，`output` 内带 `adapter_status=adapter_required/degraded`、空 `evidence` 和 `degraded_reason`。`provenance.stale_filtered_count` 与 `freshness.stale_filtered_count` 表示后端按 `query.days` 过滤掉的过期来源数量。
- 搜索类 `skill_result.output.adapter.readiness` 会返回外部搜索 adapter 诊断：`status=disabled` 表示 `[external_search].enabled=false`，`status=missing_config` 表示已启用但缺 `base_url` 或 `search_endpoint`，`status=configured` 表示后端会尝试请求真实 provider。该结构还包含 `missing_config`、`has_api_key`、`timeout_sec`、`default_topic`、`default_days`、`default_max_results`、`max_results_limit` 和中文 `message`；`api_key` 只暴露是否存在，不暴露原文。`skill_result.metadata.adapter_readiness` 会保存同一份诊断，供前端过程面板、工作区搜索证据和排查使用。即使 `readiness.status=configured`，provider 请求失败、无可追溯 URL 来源或来源超出 freshness 窗口时仍会降级，不得展示成 live 搜索。
- 当用户自然语言明确表达“搜索/联网/实时/最新/公开来源/公开资料/真实评价/平台规则/行业基准”等意图时，worker autosuggest 可自动产生 `skill_requested.skill_id=search_trends/search_competitor/search_market_info/search_product_reviews/search_platform_policy/search_industry_benchmarks`。后端会解析 `query`、`platform`、`focus`、`topic`、`policy_type`、`metric`、`days` 和 `_needs_fresh_data` 等输入；普通“竞品分析、趋势预测、关键词研究、创意趋势捕捉”等本地 native 意图仍优先进入对应专门 skill，不会被搜索 skill 抢走。
- 客服岗位的 DSR / 店铺评分 / 服务评分 / NPS / 满意度驱动因素 / 投诉分类意图如果同时包含“行业基准 / 行业均值 / 最新 / 实时 / 联网”等诉求，worker 会生成可见双 skill 链路：先请求 `service_dsr_improvement`、`service_nps_analyzer` 或 `service_nps_driver_analysis`，再请求 `search_industry_benchmarks`。`search_industry_benchmarks.input.metric` 会按语义使用 `dsr` 或 `nps`；该搜索步骤会独立出现在 `skill_requested` / `skill_result` 中，前端应把它展示为外部基准检索，而不是把搜索来源混入客服 native 分析。搜索失败、未配置、无可信来源或来源过期时仍按搜索 skill 降级，不得伪造成实时行业基准。
- 客服岗位的情绪分析 / 工单处理 / 退换货处理 / 满意度驱动因素分析如果同时包含“真实评价 / 商品评价 / 买家评价 / 口碑 / 差评 / 吐槽 / 真实反馈 / 联网 / 搜索 / 最新”等诉求，worker 会生成可见双 skill 链路：先请求 `service_sentiment_analyzer`、`service_ticket_handler`、`service_return_handler` 或 `service_nps_driver_analysis`，再请求 `search_product_reviews`。差评、负面、投诉、吐槽、不满意等语义会写入 `search_product_reviews.input.focus=negative`；好评、正面、满意等语义会写入 `focus=positive`；否则为 `overall`。类似“最近30天、最多5条来源”的表述会分别写入 `days=30` 和 `max_results=5`。普通客服处理没有真实评价诉求时仍只进入 native skill；未显式选择商品时不会从历史摘要或默认演示商品注入 `product_ref/product_name/product_category`。
- 客服岗位的工单处理 / FAQ话术 / 升级处理 / 退换货处理如果同时包含“平台规则 / 平台政策 / 售后规则 / 退换货规则 / 赔付标准 / 运费规则 / 投诉规则 / 平台介入规则 / 七天无理由 / 最新 / 联网 / 公开资料”等诉求，worker 会生成可见双 skill 链路：先请求 `service_ticket_handler`、`service_faq_playbook`、`service_escalation_flow` 或 `service_return_handler`，再请求 `search_platform_policy(policy_type=general)`。类似“最多4条来源”的表述会写入 `max_results=4`；普通客服处理没有平台规则诉求时仍只进入 native skill。搜索 provider 未配置、失败或无可信来源时仍按搜索 skill 降级，不伪造平台规则、赔付标准或售后政策；未显式选择商品时不会从历史摘要或默认演示商品注入 `product_ref/product_name/product_category`。
- 运营岗位的定价策略 / 智能定价 / 促销策划 / 渠道策略 / 货盘选品规划 / 上架文案如果同时包含“最新 / 实时 / 联网 / 搜索 / 公开资料 / 市场定价 / 竞品价格区间 / 平台活动规则 / 大促规则 / 渠道趋势 / 行业 ROI 基准 / 热卖趋势 / 标题规则 / 上架规则 / 搜索量 / 买家痛点 / 真实评价”等诉求，worker 会生成可见双 skill 链路：先请求 `ops_pricing_strategy`、`ops_smart_pricing`、`ops_promo_planning`、`ops_channel_strategy`、`ops_assortment_planning` 或 `ops_listing_copy`，再按场景请求 `search_competitor`、`search_platform_policy`、`search_market_info`、`search_industry_benchmarks`、`search_trends` 或 `search_product_reviews`。定价链路搜索 `focus=price`；促销平台规则搜索 `policy_type=promotion`；上架标题 / 发布 / 内容规范搜索使用 `search_platform_policy(policy_type=content)`；上架买家痛点 / 评价搜索使用 `search_product_reviews(focus=negative/positive/overall)`；标题趋势 / 搜索量 / 卖点趋势搜索使用 `search_market_info(topic=general)`。普通运营任务没有实时资料诉求时仍只进入 native skill；未显式选择商品时不会从历史摘要或默认演示商品注入 `product_ref/product_name/product_category`。
- 财务岗位的财务健康诊断 / P&L / GMV利润瀑布 / 预算执行分析如果同时包含“行业财务基准 / 财务对标 / 最新毛利率 / 净利率 / 广告ROI基准 / 财务benchmark / 联网 / 搜索 / 最新”等诉求，worker 会生成可见双 skill 链路：先请求 `accounting_financial_narrative`、`accounting_pl_statement`、`accounting_gmv_waterfall` 或 `accounting_budget_vs_actual`，再请求 `search_market_info(topic=finance)`。类似“最多4条来源”的表述会写入 `max_results=4`，默认财务基准搜索窗口为 `days=60`；纯“查最新电商行业财务基准”只触发搜索，不会误生成预算编制或财务诊断。普通财务计算没有实时财务基准诉求时仍只进入 native skill；未显式选择商品时不会从历史摘要或默认演示商品注入 `product_ref/product_name/product_category`。
- 设计岗位的主图设计 / 活动海报设计 / 详情页设计如果同时包含“视觉趋势 / 设计趋势 / 爆款主图 / 高点击率 / 爆款海报 / 竞品视觉 / 最新 / 实时 / 联网 / 搜索 / 公开资料”等诉求，worker 会生成可见双 skill 链路：先请求 `design_main_image`、`design_campaign_poster` 或 `design_detail_page`，再请求 `search_trends`。主图和详情页视觉趋势默认 `days=60`，活动海报默认 `days=30`；类似“最多4条来源”的表述会写入 `max_results=4`。纯“查最新爆款主图视觉趋势”只触发 `search_trends`，不会误生成设计 brief。品牌配色如果同时包含“流行色 / 配色趋势 / 竞品配色 / 最新 / 联网 / 公开资料”等诉求，会先请求 `design_color_scheme`，再请求 `search_trends`，默认 `days=90`。素材规范如果同时包含“平台素材规范 / 广告位规格 / 尺寸规则 / 上传格式 / 文件大小 / 安全区 / 审核要求 / 最新 / 联网 / 公开资料”等诉求，会先请求 `design_material_spec`，再请求 `search_platform_policy(policy_type=material)`；纯“查最新素材尺寸规范”只触发平台规则搜索，不会误生成素材规范交付物。普通设计任务没有实时视觉趋势或平台素材规则诉求时仍只进入 native skill；未显式选择商品时不会从历史摘要或默认演示商品注入 `product_ref/product_name/product_category`。
- 数据岗位的竞品分析 / 价格弹性分析如果同时包含“最新 / 实时 / 联网 / 搜索 / 公开资料 / 市场定价 / 竞品价格区间”等诉求，worker 会生成可见双 skill 链路：先请求 `data_competitor_analysis` 或 `data_price_elasticity`，再请求 `search_competitor`。价格弹性链路中的搜索 `focus=price`；该搜索步骤独立出现在 `skill_requested` / `skill_result` 中，前端应展示为外部竞品 / 市场定价检索。普通“竞品分析、价格弹性分析”没有实时资料诉求时仍只进入 native skill，不会自动伪造实时市场情报。
- 数据岗位的渠道 ROI / 渠道归因 / 退款分解 / 趋势预测 / 需求预测 / 多周期趋势 / 季节性分解 / 异常诊断 / 综合诊断 / 数据看板如果同时包含“最新 / 实时 / 联网 / 搜索 / 公开资料 / 行业基准 / 行业均值 / 市场趋势 / 消费趋势 / 热卖趋势 / 市场报告 / 行业动态”等诉求，worker 会生成可见双 skill 链路：先请求 `data_channel_roi`、`data_attribution_analysis`、`data_refund_decomposition`、`data_trend_forecast`、`data_demand_forecast`、`data_multi_period_trend`、`data_seasonal_decompose`、`data_anomaly_diagnosis`、`data_comprehensive_diagnosis` 或 `data_dashboard`，再按场景请求 `search_industry_benchmarks`、`search_trends` 或 `search_market_info`。渠道 ROI / 归因基准搜索写入 `metric=roi`；退款基准搜索写入 `metric=refund`；趋势、需求、多周期和季节性场景走 `search_trends`；异常诊断走 `search_market_info(topic=news)`；综合诊断和看板走 `search_market_info(topic=general)`。普通数据分析没有实时资料诉求时仍只进入 native skill；未显式选择商品时不会从历史摘要、预设商品或测试商品注入 `product_ref/product_name/product_category`。
- 创意岗位的短视频脚本、种草文案、内容排期、趋势捕捉、直播话术和产品软文如果同时包含“最新 / 实时 / 联网 / 搜索 / 爆款 / 热门风格 / 热门话题 / 营销节点 / 直播间玩法 / 公众号爆款文章”等诉求，worker 会生成可见双 skill 链路：先请求 `creative_video_script`、`creative_seeding_copy`、`creative_content_calendar`、`creative_trend_catch`、`creative_live_script` 或 `creative_product_article`，再按场景请求 `search_trends` 或 `search_market_info`。直播话术和产品软文的外部趋势搜索默认使用 `search_trends`，并按“最多4条来源”等自然语言写入 `max_results`；类似来源数量不会被误当成直播时长。该搜索步骤独立出现在 `skill_requested` / `skill_result` 中，前端应展示为外部内容趋势 / 营销节点检索，而不是把外部来源隐藏进创意 native 输出。普通“写脚本 / 写种草 / 做内容排期 / 分析内容趋势 / 写直播话术 / 写产品软文”没有实时资料诉求时仍只进入 native skill；未显式选择商品时不会从历史摘要或默认演示商品注入 `product_ref/product_name/product_category`。
- Web 岗位的 SEO 优化、关键词研究、页面转化分析、标题生成、详情文案如果同时包含“最新 / 实时 / 联网 / 搜索 / 公开来源 / 搜索算法 / 流量规则 / 关键词竞争 / 搜索量 / CPC / 行业转化率基准 / 标题规则 / 标题规范 / 热搜词 / 高点击 / 买家痛点 / 真实评价 / 详情页趋势 / 高转化卖点”等诉求，worker 会生成可见双 skill 链路：先请求 `web_seo_optimize`、`web_keyword_research`、`web_page_conversion`、`web_title_generator` 或 `web_product_description`，再按场景请求 `search_platform_policy`、`search_market_info`、`search_product_reviews` 或 `search_industry_benchmarks`。SEO 算法 / 流量规则走 `search_platform_policy.input.policy_type=algorithm`；标题 / 内容发布规则走 `search_platform_policy.input.policy_type=content`；关键词竞争 / 搜索量 / CPC / 标题趋势 / 详情页趋势走 `search_market_info`；详情买家痛点 / 真实评价走 `search_product_reviews`；页面转化率基准走 `search_industry_benchmarks.input.metric=conversion`。普通 SEO / 关键词 / 页面转化 / 标题 / 详情文案任务没有实时资料诉求时仍只进入 native skill；未显式选择商品时不会从历史摘要或默认演示商品注入 `product_ref/product_name/product_category`。
- 工程岗位的 Bug 排查 / 技术选型 / 性能优化 / 架构评估如果同时包含“联网 / 搜索 / 公开资料 / 已知解决方案 / 最佳实践 / 最新对比 / 性能 benchmark”等诉求，worker 会生成可见双 skill 链路：先请求 `engineering_bug_analysis`、`engineering_tech_selection`、`engineering_perf_optimize` 或 `engineering_arch_review`，再请求 `search_market_info(topic=general)`。Bug 排查搜索 query 会包含错误类型、技术栈、解决方案、排查和最佳实践；技术选型搜索 query 会包含选型领域、技术选型、对比、性能和 benchmark；默认搜索窗口为 `days=90`。普通工程任务没有外部资料诉求时仍只进入 native skill；搜索 provider 未配置、失败或无可信来源时仍按搜索 skill 降级，不得伪造 benchmark、社区案例或技术对比。
- 搜索类 skill 会兼容旧产品 `_needs_fresh_data` 语义，也接受 `bypass_cache` / `bypassCache`，并把结果写入 `query.bypass_cache` 后转发给 provider；这只表示请求实时性偏好，不代表 provider 一定绕过缓存。搜索类 skill 声明并接受可选 `max_results`，同时兼容 `maxResults`；显式输入优先于 `[external_search].default_max_results`，并会 clamp 到 1-12。worker autosuggest 会从“最多3条来源 / 返回5条结果 / top 4 sources”等搜索意图中提取 `skill_requested.input.max_results`，但不会把“最近30天”这类时间窗口误当作结果数量。`output.query.max_results` 是本次搜索计划使用的来源数量上限，后端会在 provider 返回后按该值截断 `sources`、`data.items`、`provenance` 和 `evidence`，避免 provider 返回过多来源污染本轮回答和搜索证据 artifact。
- 搜索 provider 返回结构兼容 `data.items`、`data.results`、`data.sources`、`data.organic`、`data.organic_results`、`data.webPages.value`、`items`、`sources`、`results`、`organic`、`organic_results`、`webPages.value` 和 `value`；来源字段兼容 `title/name/headline`、`url/link/href/source_url/sourceUrl`、`snippet/body/summary/content/description/text/excerpt`、`published_at/publishedAt/pubDate/published/datePublished/date`。只有带可追溯 URL 且可从 URL 本身形成 domain 的来源会进入 live `sources` 和 `evidence`，无 URL 或相对路径的弱结果会被过滤；有可解析发布时间且超出本次 `query.days` 的来源也会被过滤；过滤后没有可信来源时仍降级，不创建搜索证据 artifact。
- 搜索类 worker report 会在 `task_node_reported.report.evidence[]` 中写入 `search_evidence_packet:{...}`，供下游业务岗位作为 structured fact 使用；即使搜索 provider 未配置、失败、超时或无可信来源，只要 skill output 有 `evidence_packet`，也会保留该证据包和缺口，不让下游误以为没有搜索过。后端还会在 `task_node_reported.report.diagnostics[]` 中写入 `kind=search_evidence` 的 typed diagnostic，字段包含 skill、status、adapter_status、source_state、intent、strength、sources、usable_claims、gaps、confidence 和 query 等，用于过程面板、smoke 报告、session memory 和下游岗位快速判断搜索链路。旧的紧凑 `search_evidence_trace: skill=... status=... adapter_status=... intent=... strength=... sources=... usable_claims=... gaps=... confidence=... query="..."` 仍会保留在 evidence 中作为兼容 fallback；后端最终回答清洗层会过滤 `search_evidence_packet` 和 `search_evidence_trace`，用户最终回答只应展示自然语言结论和可读的“实时搜索来源”，不得直接暴露内部 trace、diagnostic JSON 或 packet JSON。
- 搜索类 skill 也支持可选 `product_ref`、`product_name`、`product_category`。当本轮已通过 `metadata.context.productIds` 或 `metadata.context.contextRefs` 显式选择商品，且用户说“这个商品 / 本品 / 当前商品 / 所选商品 / selected product”等指代时，worker 会把选中商品名称和类目用于 `query` 或 `industry`，并把选中商品平台用于未显式指定平台的搜索；搜索输出的 `output.query.product_ref/product_name/product_category` 会原样回显。未选择商品时，这些字段不会出现，也不会从历史对话、工作区摘要、预设商品或测试商品推断默认商品。
- 当搜索类 `skill_result` 同时满足 `status=success`、`execution_mode=adapter`、`output.status=live` 且 `output.sources` 非空时，web-server 会把完整搜索输出沉淀为 `artifactType=search_evidence`、`format=json`、`storageKind=inline` 的工作区交付物，并额外发送 `workspace_artifact_created`。未配置、降级或无来源时不会创建搜索证据 artifact。
- `service_query_product` 通过既有 `skill_result` 读取本轮显式注入的商品上下文。若用户已在本轮选择/引用商品且 `product_ref=product:{uuid}`、旧 `product_id` 或商品名筛选命中，返回 `status=success`、`execution_mode=native`、`output.adapter_status=context_metadata`、`output.data_readiness=explicit_product_context`；若未选择商品或商品引用 / ID / 名称筛选不命中，返回 `status=deferred`，不会默认注入或猜测商品。客服岗位自然语言表达“查询商品 / 商品信息 / 商品资料 / 引用商品”时，worker 可自动产生该 skill；已显式选品时 input 会携带 `product_ref`，未选品时不会留下空引用字段。
- `service_ticket_handler`、`service_faq_playbook`、`service_escalation_flow`、`service_return_handler` 支持 `product_ref=product:{uuid}` 作为本轮显式选中商品的关联字段。worker 只会在 `context.selected_summary` 存在时给这些客服支持类 skill 自动补偿 `product_ref`；未选商品时不会写入空引用，也不会把历史商品或默认商品塞入客服处理流程。
- `creative_video_script`、`creative_seeding_copy`、`creative_product_article`、`creative_live_script`、`ops_listing_copy`、`ops_promo_planning`、`ops_pricing_strategy`、`ops_channel_strategy`、`ops_inventory_planning`、`ops_assortment_planning`、`ops_smart_pricing`、`ops_inventory_optimizer`、`ops_ad_fatigue_detector`、`web_keyword_research`、`web_title_generator`、`web_product_description`、`design_main_image`、`design_detail_page` 等商品型自动补偿会优先消费本轮 `context.selected_summary`。如果用户手动选择了商品，skill input 中的 `product_name` / `product` / `product_type` / `category` / `product_ref` / `product_refs` / `platform` / `features` / `selling_points` 会来自选中商品和知识；如果没有手动选择商品，这些字段不会从历史摘要或默认商品推断，也不会留下空的 `product_ref/product_refs`。
- 当 creative / design / data / accounting / service / ops / douyin_ads / web / engineering 内容类 `skill_result` 返回 `status=success` 且命中支持名单时，web-server 会把输出沉淀为 markdown 工作区交付物，并额外发送 `workspace_artifact_created`。当前 creative 支持：`creative_video_script`、`creative_seeding_copy`、`creative_ip_branding`、`creative_content_calendar`、`creative_live_script`、`creative_product_article`、`creative_trend_catch`、`creative_title_ctr_scorer`。当前 design 支持：`design_main_image`、`design_detail_page`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster`。当前 data 支持：`data_dashboard`、`data_funnel_analysis`、`data_channel_roi`、`data_refund_decomposition`、`data_anomaly_diagnosis`、`data_trend_forecast`、`data_customer_segmentation`、`data_competitor_analysis`、`data_ab_test_analyzer`、`data_attribution_analysis`、`data_multi_period_trend`、`data_ltv_calculator`、`data_cohort_analysis`、`data_seasonal_decompose`、`data_price_elasticity`、`data_demand_forecast`、`data_comprehensive_diagnosis`、`query_store_metrics`、`query_product_metrics`、`query_campaign_metrics`。`query_store_metrics` / `query_product_metrics` / `query_campaign_metrics` 只有在本轮显式指标输入并返回 `status=success` 时才分别创建店铺指标报告、商品指标报告和活动指标报告；缺指标的 `deferred` 状态不会创建 artifact。当前 accounting 支持：`accounting_cost_calc`、`accounting_profit_analysis`、`accounting_roi_calc`、`accounting_budget_plan`、`accounting_break_even_calc`、`accounting_compliance_check`、`accounting_pl_statement`、`accounting_budget_vs_actual`、`accounting_gmv_waterfall`、`accounting_scenario_analysis`、`accounting_cash_flow_forecast`、`accounting_financial_narrative`。当前 service 支持：`service_ticket_handler`、`service_faq_playbook`、`service_escalation_flow`、`service_dsr_improvement`、`service_return_handler`、`service_nps_analyzer`、`service_sentiment_analyzer`、`service_nps_driver_analysis`。当前 ops 支持：`ops_promo_planning`、`ops_pricing_strategy`、`ops_channel_strategy`、`ops_inventory_planning`、`ops_listing_copy`、`ops_assortment_planning`、`ops_execution_plan`、`ops_smart_pricing`、`ops_inventory_optimizer`、`ops_ad_fatigue_detector`、`ops_abc_xyz_classifier`。当前 douyin_ads 支持：`douyin_ads_plan_structure`、`douyin_ads_audience_package`、`douyin_ads_budget_pacing`、`douyin_ads_live_funnel_diagnosis`。当前 web 支持：`web_seo_optimize`、`web_title_generator`、`web_page_conversion`、`web_store_design`、`web_keyword_research`、`web_product_description`、`web_title_seo_scorer`。当前 engineering 支持：`engineering_arch_review`、`engineering_bug_analysis`、`engineering_perf_optimize`、`engineering_tech_selection`、`engineering_sla_monitor`。
- 其它需要内容引擎、平台数据或未接 adapter 的能力会返回 `deferred`，不会伪造外部资料或平台执行结果。

`skill_result.status` 当前使用：`success`、`deferred`、`pending_approval`、`not_found`、`validation_error`、`failed`。其中 `pending_approval` 和 `approval_required=true` 表示后端没有直接执行平台写操作，需要进入审批或人工确认链路；`deferred` 表示 skill 已被识别但依赖搜索、外部数据、数据库或平台适配器，当前不会伪造结果。搜索类 skill 在 provider 返回可信来源时会以 `success` 承载 `output.status=live`，provider 不可用时仍以 `deferred` 承载明确降级原因。

补充：`query_store_metrics` / `query_product_metrics` / `query_campaign_metrics` 成功输出会通过既有内容 artifact 物化链路分别创建 `artifactType=data_store_metrics_report`、`artifactType=data_product_metrics_report` 和 `artifactType=data_campaign_metrics_report` 的 markdown 工作区交付物；缺店铺、商品或活动指标的 `deferred` 状态不会触发 `workspace_artifact_created`，也不会伪造平台实时指标报告。

`rag_context_loaded` 当前使用本地 RAG：来源只来自当前用户自己的 active `products` 和 active `product_knowledge`。只有本轮显式选择商品时，服务端才会把检索摘要追加到 agent runtime `context.summary`，让模型可在回答中使用这些事实。如果没有选择商品，事件可返回空检索状态，但不会向模型注入商品知识或“默认商品”提示。如果已选择商品但没有命中知识，事件会返回 `degraded=true` 和 `degradedReason`。当后端 `[external_search]` 未完整配置时，`externalSearchAvailable=false`，降级原因为 `No matching local product knowledge was found; external search adapter is not configured.`；当 `[external_search]` 已完整配置时，`externalSearchAvailable=true`，降级原因会说明本地知识未命中，但搜索类 skill 可以在任务需要实时市场资料时请求 live sources。当前不会伪造外部搜索结果；只有同一轮 `search_*` skill 成功返回 `output.status=live` 和真实来源时，前端才应展示 live 外部搜索证据。

`workspace_context_loaded` 当前只聚合来源元数据，不读取文件正文，不做 OCR，不伪造素材总结。普通工作区查询接口可以按请求参数查看历史 workspace artifact；但 SSE Agent 自动注入时默认只使用本次 run 范围，避免历史报告影响未指定商品的普通问答。用户显式传入的 `conversation_file:{fileId}` / `file:{fileId}`、`product_asset:{assetId}` / `asset:{assetId}`、`product_material:{materialId}` / `material:{materialId}`、`workspace_artifact:{artifactId}` / `artifact:{artifactId}` 例外，会在权限校验后加入本轮上下文来源。若同一会话存在 active 图片附件，Agent SSE 会额外注入最多 4 张支持格式图片的 runtime 输入：使用已沉淀的 `storage_object_analyses.image_preprocess` 摘要，并同步追加到 runtime `context.combined_summary`；这属于 runtime 模型输入，不等同于把 object storage 二进制内容写入 context pack。服务端会把 `agentContext.refs` 追加到 runtime `context.combined_refs`，把 `summary` 和图片预处理摘要追加到 `context.combined_summary`，并写入 `workspaceContext.readiness`、`workspaceContext.counts_json`、`workspaceContext.refs`、`workspaceContext.sources_json`、`workspaceContext.missing_json`、`workspaceContext.warnings_json`。`readiness=empty` 表示没有可用附件、素材或交付物，前端应展示明确空状态；这不是外部搜索或附件解析结果。

`workspace_context_pack_loaded` 会继续把可读正文追加到 runtime `context.combined_summary`，并写入 `workspaceContextPack.readiness`、`workspaceContextPack.counts_json`、`workspaceContextPack.items_json`、`workspaceContextPack.refs`、`workspaceContextPack.omitted_json`、`workspaceContextPack.warnings_json`。可读来源包括本轮明确 product_id 下的 `product_material.content`、`product_asset.description`、`workspace_artifact.content/preview`，以及 `conversation_file.metadata.extractionStatus=extracted` 时的 `extractedText/textContent/plainText/markdown/text/content/preview/summary` 等字段。长文本来源如果按本轮问题抽取了相关片段，会在 item metadata 写入 `relevanceSelection.strategy=query_keyword_chunks`、`chunkCount`、`totalChunkCount`、`queryTerms` 和 `chunks[]`；每个 chunk 记录 `chunkIndex/displayIndex/startChar/endChar/score/preview`，用于解释哪些原文区间进入了本轮模型上下文。CSV/TSV 或疑似表格文本进入内容包时使用 `contentKind=table_profile`，不会整表 inline；表格 profile 会保留 schema、样例行、query 相关列、query 相关样例行和可解析数值列的 `count/sum/avg/min/max`，并对“利润/成本/退款/投产/转化/库存”等常见中文电商指标词做轻量表头别名匹配；样例行选择策略记录在 `metadata.tableProfile.sampleRowSelection/sampleRowIndexes`。当表头能识别常见电商字段时，`metadata.tableProfile.derivedMetrics` 会额外给出 `net_revenue/gross_profit/gross_margin/refund_rate/ad_roi/conversion_rate/click_through_rate/inventory_to_orders` 等确定性派生指标，包含公式、格式化值和输入列；这些指标只来自已抽取表格文本，不代表平台实时数据。较大的 JSON/JSONL 文本进入内容包时使用 `contentKind=json_profile`，不会整份 inline；JSON profile 会保留结构路径、样例值、对象/数组/标量计数、最大深度和 query 相关路径，策略与命中结果记录在 `metadata.jsonProfile.strategy/queryMatches/queryMatchCount`。`artifactType=search_evidence` 且 `metadata.kind=agent_search_evidence` 的交付物进入内容包时使用 `contentKind=search_evidence_summary`，后端从已沉淀的搜索 JSON 中整理 query、显式商品上下文、证据摘要、来源数量、置信度、域名和 Top sources；这不会触发新搜索，也不会从 query 或历史推断商品。未成功抽取的附件会返回 `contentKind=metadata_only`、`extraction_pending`、`extraction_failed` 或 `extraction_unsupported`，不会被当成已读取正文。

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

补充：`ops_execution_plan` 支持 `product_ref=product:{uuid}` 记录本轮显式选中商品，成功输出中的 `商品引用` 会回显该引用；未选商品时 worker 不注入空 `product_ref`，直接调用 skill 时该输出字段为 `null`。运营执行计划是运营岗位的兜底计划能力，适用于执行计划、运营计划、新品冷启动计划等问法；当前只基于用户目标、阶段和确定性执行框架生成，不读取或伪造淘宝、抖音、千川、ERP、WMS、供应链或实时平台数据。

补充：`search_trends`、`search_competitor`、`search_market_info`、`search_product_reviews`、`search_platform_policy`、`search_industry_benchmarks` 支持在 input 和 `output.query` 中携带 `product_ref=product:{uuid}`、`product_name`、`product_category`，用于标记本轮显式选中商品触发的外部搜索。该标记只来自 `context.selected_summary`，不会由搜索 skill 自行查询商品库，也不会把未选商品的普通搜索强行绑定到历史商品。搜索 provider 未配置、失败或无来源时仍返回 `deferred/degraded`，不会创建假 live 来源或假搜索证据 artifact。

补充：当上述 `search_*` skill 返回真实 live 来源并创建 `search_evidence` artifact 时，后端会把 `output.query.product_ref/product_name/product_category` 沉淀到 artifact 的 `metadata.productContext`，并冗余到 `metadata.productRef/productName/productCategory`，preview 也会出现 `Product:` 行。没有显式商品上下文时不会写空字段，也不会从 query 文本、历史对话或工作区摘要推断商品。

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

- 新前端应优先消费 `dispatch_planned` 作为过程面板的第一来源；如果旧后端没有该事件，再回退到 `route_decided`、`task_graph_planned`、`skill_requested` 和 `skill_result` 推导展示。
- 前端不得根据 UI 关键词伪造岗位、Skill 或多智能体过程；`dispatch_planned.selected_skills` 是计划提示，不等于已经执行，实际执行仍以 `skill_requested` / `skill_result` 为准。
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
