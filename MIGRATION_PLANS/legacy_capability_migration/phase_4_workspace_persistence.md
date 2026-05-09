# 4/8：工作区沉淀与交付物

状态：done

## 本次目标

把一次 Agent 问答从“只在 SSE 里临时展示”推进到“后端能沉淀为真实工作区运行记录和交付物”。本阶段完成 workspace/run/task 的后端链路、最终报告 artifact 表、HTTP 查询/维护接口、SSE 交付物事件和前端运行态展示对齐。

## 产品内核底层变化

- Agent session 启动前会先绑定一个 `workspace_run`。
- 请求未传 `workspaceId` 时，后端创建或复用当前用户默认 `agent_chat` 工作区。
- Agent 任务图节点会被映射为工作区任务和运行任务快照。
- 最终输出会写入 `workspace_runs.report.output`，刷新后仍可通过工作区 API 查询。
- 最终输出会额外沉淀为 `workspace_artifacts` 中的 `final_report`，后续可承接旧产品的报告、素材、文件类交付物。

## 后端变化

- 新增 `workspace_artifacts` 表，关联 `workspace_id`、`run_id`、`workspace_task_id`、`run_task_id`、`user_id`。
- 新增 `WorkspaceArtifactService`，提供创建、详情、列表、计数、更新能力。
- `WorkspaceApi` 新增 agent run 绑定、任务图沉淀、节点开始、worker report、evaluation、完成和失败记录方法。
- `WorkspaceApi` 新增 artifact 创建、查询、更新、归档方法，并在 `complete_agent_run` 中使用 service 层事务同时完成 run 成功状态和 final report artifact 创建。
- `POST /api/sse/agent/session/stream` 在启动 agent stream 前调用 `WorkspaceApi::link_agent_run`。
- SSE 收到 `task_graph_planned` 时写入 `workspace_tasks` / `workspace_run_tasks`。
- SSE 收到 `task_node_started`、`task_node_reported`、`task_node_evaluated` 时更新运行任务状态和摘要。
- SSE 收到 `finished` 时标记 run 为 `succeeded` 并创建 `workspace_artifact_created` 交付物事件；收到 `failed` 时标记 run 为 `failed`。
- 运行中单次沉淀失败会发 `workspace_run_persistence_error`，不截断原始 LLM 回答。

## API 与事件变化

- 已更新 `API_DOCS/sse.md`。
- 已更新 `API_DOCS/app.md` 中 workspace run / run task / artifact 的说明。
- 新增 HTTP 接口：
  - `GET /api/app/workspaces/{workspaceId}/artifacts`
  - `POST /api/app/workspaces/{workspaceId}/artifacts`
  - `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}`
  - `PATCH /api/app/workspaces/{workspaceId}/artifacts/{artifactId}`
  - `DELETE /api/app/workspaces/{workspaceId}/artifacts/{artifactId}`
- 新增 SSE 事件：
  - `workspace_run_linked`
  - `workspace_run_tasks_linked`
  - `workspace_run_task_updated`
  - `workspace_run_completed`
  - `workspace_artifact_created`
  - `workspace_run_failed`
  - `workspace_run_persistence_error`

## 前端对齐

- `AgentRuntimeEvent` 已增加 workspace run、run task、artifact 和 persistence error 事件类型。
- 问答请求不再发送假的 `workspace-001`，未传 `workspaceId` 时交由后端创建/复用默认工作区。
- 右侧运行态面板已展示真实 workspace/run/task/artifact 沉淀结果。
- 后续完整工作区页面仍需要继续用 `/api/app/workspaces/{workspaceId}/runs`、`/tasks`、`/artifacts` 做刷新后列表与详情展示。

## 当前验证

- [x] `cargo fmt`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo test -p web-server sse`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
