# Agent Runner 与 Task Graph 配置化协议

本文档约束 `agent` crate 里的 commander、task graph 和 worker runner。agent 的业务编排和 worker 业务能力必须优先通过配置完成，Rust 代码只允许保留通用编排引擎和通用执行引擎。

## 核心原则

1. worker runner 是一次任务执行过程中的短生命周期组件，由外部 agent manager 和 commander 监管；任务完成后释放。
2. commander 负责任务入口、task graph 规划、节点调度、重试、收集 worker report 和完成 graph；worker 只执行当前分配给自己的单个节点。
3. task graph 是跨角色业务编排的唯一配置入口；worker 不决定多角色协作顺序，也不决定最终汇总节点。
4. runner 不负责维护长期业务状态，不重新设计 memory、cognition、skill 或 manager。
5. 角色、任务图、autosuggest、inference、输出策略和 prompt 都必须优先放在 `backend/config/` 下。
6. 普通业务调整只允许修改配置文件，不允许修改 commander/task_graph/worker runner Rust 代码。
7. Rust 侧只保留“读取配置、匹配配置、构建图、调度节点、执行 skill、组装上下文、校验输出、降级处理”等通用能力。
8. task graph 节点和动态角色配置可以配置 `llm_usage`，取值只使用 `fast`、`thinking` 或 `default`，用于路由到 `services.toml` 的 `[agent_llm_usage]`。轻量短答、上游草案或显式岗位协作的动态角色可以用 `fast`；最终综合、财务测算或复杂证据综合的节点保留默认 `thinking`。不要在 Rust 里按行业或业务关键词硬编码模型选择。
9. task graph 节点和动态角色配置可以配置 `skill_autosuggest`。显式多岗位轻量协作、最终润色或不需要外部适配器的节点可以设为 `false`，避免自动技能请求拖慢和污染输出；需要后台、搜索、投放、财务等真实工具能力的深度图保持默认开启。动态角色还可以配置 `skill_autosuggest_by_role`，用于在同一条规则里覆盖某个 runtime role 的自动技能开关，例如轻量显式团队默认关闭 autosuggest，但对 `search` 证据岗单独打开。
10. task graph 的 `match.min_explicit_role_aliases` 是通用匹配条件，用于要求用户文本至少命中若干个 `role-aliases.json` 中的岗位别名。它只表达“是否真的点名了岗位”，不在 Rust 里写业务岗位词。`match.min_chars` / `match.max_chars` 是通用长度门控，用于区分短寒暄、短答、视觉直答和需要进入业务编排的任务文本；它只能表达文本长度边界，不得替代业务关键词、岗位别名或运行时超时配置。
11. worker inference 的搜索配置可以在 `search.max_results.response_mode_defaults` 中按 `strategy.response_mode` 配置搜索默认强度。该默认值只在 autosuggest 已经触发搜索且用户未显式指定条数、深度/标准/轻量强度时补齐，不得反向触发搜索。
12. worker cognition 超时只能通过 `backend/config/services.toml` 的 `[agent_runtime].worker_cognition_timeout_sec` 统一配置。task graph 节点、动态角色、最终节点、role defaults 和 task graph defaults 都不允许配置 `timeout_sec`；业务编排 JSON 只描述角色、依赖、prompt 目标、llm_usage、skill_autosuggest、验收条件等业务调度信息，不承载运行时保护时间。
13. dynamic roles 的 `from_explicit_request.depends_on_roles` 和 `from_route.depends_on_roles` 是通用依赖配置，用于声明某类动态角色应依赖哪些上游 runtime role，例如业务岗位依赖 `search` 证据岗。Rust 只按配置把已存在的上游 role 映射为 node `input_refs`，不写具体行业流程；如果上游 role 未被本轮规划，则该依赖自动忽略。
14. worker cognition 遇到模型访问权限错误时必须形成结构化失败诊断。`llm_access_denied` 代表服务端模型配置或 provider 权限不可用，worker report 应保留兼容 marker `failure_kind:llm_access_denied`，同时写入 `WorkerReport.diagnostics[]` 的 `kind=failure`、`fields.failure_kind=llm_access_denied`；task graph 最终合成应优先向用户说明模型配置不可用；不能把它伪装成岗位业务失败、资料不足或“请补充更明确问题”。
15. 搜索证据的 `WorkerReport.diagnostics[]`、`search_evidence_packet:{...}` 和 `search_evidence_trace: ...` 是过程核对信息，只能留在 worker report、task graph process、smoke 报告、session memory 和后台日志里。最终用户回答必须通过 user-facing sanitizer 过滤内部 packet/trace，只保留自然语言结论和可读的“实时搜索来源”。新增搜索 evidence/diagnostics 字段时也要遵守同一边界，避免把内部 JSON、trace 或调试语句直接展示给学生。
16. `task_graph_finished.process.check_items[]` 可以承载由 runtime 从内部 diagnostics/evidence 提炼的可读核对项，例如搜索核对。该字段只做展示摘要，不替代 `WorkerReport.diagnostics[]` 或 `WorkerReport.evidence` 里的结构化事实；搜索核对应优先读取 `diagnostics.kind=search_evidence`，仅在缺少 typed diagnostic 时兼容解析旧 `search_evidence_trace`。新增 check item 时不得要求前端解析 Markdown 或 provider 原始 JSON，也不得把 API key、provider request id、完整内部 trace 或未脱敏调试信息写入用户可见过程面板。
17. commander 不允许因为某个 worker 当前未注册或发送失败而在 `TaskGraphPlanned` 前静默删除该岗位节点。任务图应保留用户显式选择或配置规划出的完整节点；如果 ready node 无法派发，commander 必须生成 `status=failed` 的结构化 `WorkerReport`，在 evidence 中保留兼容 marker `failure_kind:worker_unavailable`，同时写入 `WorkerReport.diagnostics[]` 的 `kind=failure`、`fields.failure_kind=worker_unavailable`，并让 task graph runtime 统一产生 `task_node_reported`、`task_node_failed`、`task_graph_finished.process.incomplete[]` 和可读“岗位核对”项。最终回答不能把这类执行缺口伪装成“请补充材料”。
18. smoke、浏览器回归或接口回归需要核对多岗位执行链路时，应优先复用 `agent::api::audit_session_event_roles*` 和 `render_session_event_role_audit`，对 `route_decided`、`task_graph_planned`、`task_node_reported`、`task_graph_finished.process.roles[]` 和 `process.incomplete[]` 做统一差异报告。不要在各测试里重新写一套 planned/reported/finished 对比逻辑；审计逻辑只读取事件，不改变 SSE 协议或业务编排。
19. skill 执行失败、待执行、待审批、缺 adapter 或入参校验失败时，worker report 必须优先写入 `WorkerReport.diagnostics[]` 的 `kind=failure`。`fields.failure_kind` 使用通用值，例如 `skill_adapter_timeout`、`skill_adapter_unavailable`、`skill_pending_approval`、`skill_not_found`、`skill_validation_error`、`skill_execution_failed`、`skill_deferred`；`fields.skill`、`fields.status`、`fields.request_id` 和脱敏短 `fields.reason` 用于后台核对。`task_graph_finished.process.check_items[]` 可以把这些 typed diagnostics 提炼为“Skill核对”可读项；普通最终回答不得暴露 diagnostic JSON、adapter 原始异常或 provider 调试细节。
20. `workspaceContextPack.*` 资料上下文包若不是完整 `ready`，或存在 omitted、metadata only、解析中、解析失败、解析不支持等状态，worker report 应写入 `WorkerReport.diagnostics[]` 的 `kind=failure`。`fields.failure_kind` 使用 `context_pack_extraction_failed`、`context_pack_extraction_unsupported`、`context_pack_extraction_pending`、`context_pack_omitted`、`context_pack_empty`、`context_pack_partial` 等通用值；`fields.readiness`、`item_count`、`omitted_count`、`extraction_*_count` 和脱敏短 `warning_preview` 用于后台核对。`task_graph_finished.process.check_items[]` 可以提炼为“资料核对”可读项；最终回答不得把未解析、未进入上下文或仅元数据的资料说成已读取全文，也不得暴露 object key、storage path、内部 parser 异常或原始调试 JSON。
21. smoke、浏览器回归、模式矩阵或接口回归需要统计搜索、Skill、资料包、worker 不可用、超时或模型权限等后台异常时，应优先复用 `WorkerReport.diagnostics[]` 和 `agent::api::audit_session_event_diagnostics` / `render_session_event_diagnostic_audit`。旧 evidence marker（如 `failure_kind:*`、`search_evidence_trace:*`）只能作为兼容 fallback；当同一 report 已有 typed failure diagnostic 时，不得重复统计旧 marker，避免报告里同一问题被算两次。回归报告还应检查 diagnostic 与最终回答的一致性，例如资料包失败时最终回答不得声称已读取全文，Skill 失败时不得声称已完成外部执行动作，缺少搜索证据时不得声称已完成实时搜索。

## 配置入口

业务配置入口统一在：

- `backend/config/json/agent/roles.json`
- `backend/config/json/agent/task-graph/index.json`
- `backend/config/json/agent/worker-autosuggest/`
- `backend/config/json/agent/worker-inference/`
- `backend/config/json/agent/worker-output-policy.json`
- `backend/config/json/agent/worker-output-normalization-policy.json`
- `backend/config/prompts/`
- `backend/config/services.toml` 的 `[agent_runtime]`

旧的单文件策略：

- `backend/config/json/agent/worker-autosuggest-policy.json`
- `backend/config/json/agent/worker-inference-policy.json`

只作为兼容 fallback，不作为新增业务规则的首选位置。

## 允许修改 Rust 的情况

只有下面情况可以修改 commander、task_graph 或 runner Rust 代码：

1. 新增一种通用配置字段，并同步更新配置说明。
2. 新增一种通用模板占位符求值器，例如把配置里的 `{{field}}` 解析为结构化 skill input。
3. 修复通用执行 bug、错误处理、日志或性能问题。
4. 接入新的通用 skill executor 边界。
5. 删除已经被配置完全覆盖的旧代码。

即使需要修改 Rust，也必须保持代码与业务解耦：Rust 里不能写具体行业方案、角色话术、业务关键词列表、平台规则文案、跨角色业务流程或 skill 路由规则。

## 禁止事项

1. 不允许在 commander/task_graph/runner Rust 文件里新增大段业务 if/else、match 或关键词数组。
2. 不允许把 role prompt、回答模板、教学深度、角色边界、输出格式写死进 Rust。
3. 不允许把 autosuggest skill 路由写死进 Rust。
4. 不允许把 inference 的别名、默认值、分类标签、搜索关键词写死进 Rust。
5. 不允许在 runner 里重新实现 cognition；cognition 已有独立目录，prompt 从 `backend/config/prompts/` 读取。
6. 不允许在 runner 里重新实现 memory；长期记忆和 session 记忆应走既有 agent session/memory 边界。
7. 不允许把 `old_runner` 里的旧业务代码复制回新 runner。
8. 不允许把 task graph 的业务编排写进 `commander/task_graph/*.rs` 或 `commander/runner/*.rs`。
9. 不允许让 worker 根据业务意图自行选择其它 worker、插入新 graph 节点或决定最终汇总策略。

## Commander 与 Task Graph 边界

commander 是 graph 编排运行时，只负责通用调度：

- 根据配置选择或构建 task graph。
- 把 ready node 转换成 `WorkerAssignment` 发给对应 worker。
- 接收 `WorkerReport`，更新 graph runtime。
- 根据 graph 配置决定重试、rework、完成和最终输出。
- 发出 session event，记录 graph 进度。

task graph Rust 只允许做通用配置加载、校验、模板渲染、节点构建和依赖计算。新增业务编排时，必须修改：

- `backend/config/json/agent/task-graph/index.json`
- `backend/config/json/agent/task-graph/defaults.json`
- `backend/config/json/agent/task-graph/role-aliases.json`
- `backend/config/json/agent/task-graph/role-defaults.json`
- `backend/config/json/agent/task-graph/graphs/*.json`

新增一套业务流程时，优先在 `graphs/` 下新增单独 JSON 文件，并在 `index.json` 里登记。不要恢复旧的 `task-graph-rules.json`，不要在 Rust 里维护第二份 graph。

## Worker Runner 边界

worker runner 是单节点执行器，只负责当前 `WorkerAssignment`：

- 接收 assignment 或单 worker task。
- 构建 cognition input。
- 执行显式或 autosuggest 生成的 skill request。
- 解析 worker 输出并生成 `WorkerReport`。
- 把 `graph_id`、`node_id`、上游 inputs 和 assignment metadata 透传给 cognition/skill context。

worker 允许读取 assignment 的 graph 信息，但只能作为上下文使用。worker 不允许：

- 选择下一节点。
- 调度其它 worker。
- 改写 graph 依赖关系。
- 判断整张 graph 是否完成。
- 生成跨 worker 的最终汇总结构。

如果发现 worker 里出现这些职责，必须迁移到 task graph 配置或 commander 的通用 graph runtime。

## Autosuggest 规则

autosuggest 的业务路由在 `backend/config/json/agent/worker-autosuggest/`。

Rust 只负责：

- 读取 `index.json`。
- 合并分片配置。
- 根据 role、关键词和依赖 skill 匹配规则。
- 处理 `fallback` 规则。
- 渲染 `input` 里的模板占位符。

新增业务触发、修改关键词、调整 skill input、增加链式 search，都应修改 JSON 分片。

## Inference 规则

inference 的业务解析在 `backend/config/json/agent/worker-inference/`。

Rust 只负责：

- 通用数字解析。
- 通用 metric 解析。
- 根据配置读取 alias、defaults、labels、rules。
- 为 autosuggest 模板提供结构化值。
- 在搜索已触发时，根据 `search.max_results` 的显式条数、强度关键词和 `response_mode_defaults` 依次推断 `max_results`。

新增别名、分类标签、默认预算、默认天数、搜索参数、平台/行业识别，优先修改 JSON 配置。

## Output 规则

输出补全、边界说明、默认 findings/recommendations/risks、normalization 清洗规则，分别放在：

- `backend/config/json/agent/worker-output-policy.json`
- `backend/config/json/agent/worker-output-normalization-policy.json`

回答风格、角色边界、教学深度和输出格式优先放在 prompt 或 output policy，不写进 runner。

`worker-output-policy.json` 中的 `finding_rules`、`recommendation_rules` 和 `risk_rules` 是通用输出补全规则：

- `absent_any`：当前输出条目中缺少任一关键词时，追加 `message`。
- `task_any`：可选；非空时，仅当任务文本命中其中任一关键词才启用该规则。
- `task_none`：可选；非空时，任务文本命中其中任一关键词则跳过该规则。

新增角色级业务补全时，应优先用 `task_any/task_none` 做任务门控，避免某个细分业务规则污染同角色的普通问题。

## Prompt 规则

prompt 放在：

- `backend/config/prompts/zh/cognition/`
- `backend/config/prompts/zh/roles/`
- `backend/config/prompts/en/cognition/`

不同 worker 需要不同 prompt 时，应增加或修改对应 prompt 文件，并通过配置选择；不要向 cognition Rust 代码里塞业务 prompt。

## 修改后的验证

修改 runner 配置后至少运行：

```bash
jq empty config/json/agent/*.json config/json/agent/worker-autosuggest/*.json config/json/agent/worker-inference/*.json
cargo fmt
cargo check
cargo clippy -- -D warnings
```

如果只修改 Markdown 文档，可以不运行 cargo；如果修改 JSON 配置，必须至少运行 `jq empty`。
