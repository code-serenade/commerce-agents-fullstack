# API 迁移检查清单

所有新增或修改 API 必须同步 `API_DOCS/`。

API 是后端能力和前端适配之间的唯一契约。任何前端适配开始前，必须先完成对应后端 DTO、事件结构、权限、错误码和 `API_DOCS/` 更新。前端不得绕过 API 文档直接按临时字段对接。

## 后端 API 门禁

- [ ] 能力归属已确认：agent / service / repo / web-server。
- [ ] 涉及数据库时已完成后端设计和 migration 方案。
- [ ] 涉及 LLM 或 agent runtime 时已按后端 LLM/agent 边界实现。
- [ ] DTO、错误码、鉴权、权限边界明确。
- [ ] SSE/HTTP 真实请求验证通过。
- [ ] `API_DOCS/` 更新完成后，前端才开始适配。

## Agent SSE

- [x] 扩展 request DTO：strategy/context/product/workspace metadata。
- [x] 更新 `API_DOCS/sse.md` 请求字段。
- [x] 增加或确认事件：`route_decided`。
- [x] 增加或确认事件：`context_loaded`。
- [x] 增加或确认事件：`skill_requested`。
- [x] 增加或确认事件：`skill_result`。
- [x] 增加或确认事件：`workspace_run_linked`。
- [x] 增加或确认事件：`workspace_run_tasks_linked`。
- [x] 增加或确认事件：`workspace_run_task_updated`。
- [x] 增加或确认事件：`workspace_run_completed` / `workspace_run_failed`。
- [x] 增加或确认事件：`workspace_artifact_created`。
- [x] 前端 `AgentRuntimeEvent` 同步 workspace 事件类型。
- [x] 增加或确认事件：`quality_checked`。
- [x] 增加或确认事件：`approval_required`。
- [x] 增加或确认事件：`rag_context_loaded`。
- [x] 增加或确认事件：`workspace_context_loaded`。
- [x] 增加或确认事件：`workspace_context_pack_loaded`。
- [x] 更新 `skill_requested` 触发语义：模型显式请求优先，缺失时后端可按岗位/任务自动补偿代表 skill。

## 业务 API

- [ ] 产品生命周期状态 API。
- [x] 产品知识查询/绑定到对话 API。
- [x] 附件摘要状态 API。
- [x] 附件/素材/交付物可读上下文内容包 API。
- [x] 工作区交付物导出 API。
- [x] 会话附件抽取文本登记 API。
- [x] 竞品分析读取 API。
- [x] Campaign draft 创建 API。
- [x] Commerce agent playbook 场景入口 API。
- [x] Workspace run 与 agent session 对齐 API。
- [x] Workspace artifact 创建、查询、更新、归档 API。
- [ ] Skill run 记录查询 API。
- [x] Quality check 查询 API。
- [x] Trust/learning feedback API。
- [x] Approval 承接高风险 skill API/SSE。
- [x] 旧产品内置能力包同步 API。

## 文档

- [x] `API_DOCS/sse.md`
- [x] `API_DOCS/app.md`
- [ ] `API_DOCS/common.md`
- [ ] 如新增：`API_DOCS/agent-capabilities.md`
