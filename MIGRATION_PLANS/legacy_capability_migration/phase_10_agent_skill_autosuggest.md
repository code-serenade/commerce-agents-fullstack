# 第 10 项：Agent 代表 Skill 自动补偿触发

状态：done

## 本次目标

旧产品的价值不只是有 skill 名录，而是用户提问后对应岗位能真实调用能力。新产品第 9 项已经把旧产品内置能力包同步到治理目录，但真实问答运行时仍可能因为 LLM 没有主动输出 `skill_requests`，导致前端看不到工具调用。

本次第 10 项解决这个断点：当 worker 输出没有显式 `skill_requests` 时，Agent 后端根据岗位、任务文本、上下文和上游 worker 结果，自动补偿一个代表 skill 请求，并继续走原有 skill executor、角色白名单、SSE 事件和结果合并链路。

## 产品内核变化

- Agent skill 触发从“完全依赖模型主动写 JSON”升级为“模型显式请求 + 后端岗位补偿”。
- 自动补偿只触发低风险 native 或明确 deferred 的代表 skill。
- 高风险平台写操作仍不自动执行，继续通过 `pending_approval` 和审批链路控制。
- skill 结果仍会合并进 worker report 的 evidence/risk/open questions，使最终回答可以引用真实工具状态。

## 后端变化

- 修改 `backend/crates/agent/src/agent/worker/runner.rs`。
- 修改 `backend/crates/agent/src/agent/commander/task_graph.rs` 和 `backend/crates/agent/src/agent_session/session_runtime.rs`，收紧快速聊天绕过范围，让明显业务问题进入岗位路由。
- 新增 `resolve_skill_requests`：先解析 LLM 显式 `skill_requests`，若为空再进入自动补偿。
- 新增岗位级补偿规则：
  - `ops`：定价、促销、库存、执行计划。
  - `data`：漏斗、异常、趋势、客户分层。
  - `accounting`：ROI、成本、利润、预算。
  - `service`：退换货、DSR。
  - `creative`：直播脚本、标题 CTR。
  - `web`：标题 SEO、页面转化。
  - `design`：素材规范、主图设计。
  - `engineering`：SLA 监控。
- 新增 agent 测试：
  - 显式 skill 请求不会被自动补偿覆盖。
  - 运营任务能自动补 `ops_execution_plan`。
  - 数据漏斗任务能自动补 `data_funnel_analysis`。
  - stream 级别能发出补偿后的 `skill_requested` / `skill_result`。
  - 默认策略下直接问漏斗问题，也会路由到 `data` 并触发 `data_funnel_analysis`。

## 前端变化

本次不改前端代码。

前端已有 `AgentRuntimeEvent`、时间线和工作区运行态对 `skill_requested` / `skill_result` 的消费逻辑，因此本次后端增强后，用户在问答页会更稳定看到真实 skill 调用过程。

## 用户可感知变化

用户在问答页提出运营、数据、财务、客服、创意、Web/SEO、设计、工程等岗位问题时，即使模型没有主动写出工具调用 JSON，后端也会按岗位自动补偿一个代表 skill。

例如：

- 问“请分析转化漏斗：曝光10000 点击500 加购120 下单60 付款54”，过程面板会出现 `data_funnel_analysis` 的真实调用和结果。
- 问“新品冷启动怎么做运营计划”，过程面板会出现 `ops_execution_plan`。
- 问“ROI 怎么算/投产是否健康”，过程面板会出现 `accounting_roi_calc`。
- 问“hi/你好”这种短问候仍保持快速聊天，不启动多岗位流程。

如果能力还依赖内容引擎、搜索、平台规格或真实平台数据，返回状态会是 `deferred`，不会假装已经查外部资料或执行平台动作。

## 测试记录

- [x] `cargo fmt`
- [x] `cargo test -p agent autosuggest`
- [x] `cargo test -p agent stream_emits_autosuggested_skill_events_when_worker_omits_tool_json`
- [x] `cargo test -p agent stream_routes_default_funnel_question_to_data_skill_without_manual_collab`
- [x] 真实云端 Ollama smoke：默认漏斗问题已输出 `task_graph_planned`、`skill_requested`、`skill_result`、`output`

## 未完成与后续

- 自动补偿是第一层稳定触发机制，不等于全部旧 skill 都已完整 native 化。
- 外部搜索、内容生成、平台执行、教学/导出/素材产物仍需继续做 adapter 和业务落地。
- 下一项建议继续做真实外部搜索/RAG adapter 或内容/导出类 skill runtime adapter。
