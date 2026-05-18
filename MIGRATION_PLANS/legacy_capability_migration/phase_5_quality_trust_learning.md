# Phase 5：质量、信任、学习闭环

状态：done

## 本次迁移目标

把旧产品中“回答质量检查、可信度评分、经验沉淀/学习”的能力迁移到新产品框架内。新实现以后端 workspace run 和 artifact 为核心，不在前端伪造评分，不额外调用 LLM 做二次判断。

## 产品内核底层变化

- Agent SSE 正常完成后，会基于本轮 `workspace_run`、最终输出和任务验收快照自动生成质量检查。
- 质量分会进入信任分闭环：后端按 EMA 方式把本次质量分写回 `agent_trust_scores`。
- 质量结果会沉淀成学习记录：高质量输出写入 `success`，低质量或有问题输出写入 `blindspot` / `improvement`。
- 下一次 Agent SSE 启动前，后端会读取最近学习记录和信任摘要，并注入 agent runtime metadata，作为后续回答的学习上下文。

## 后端变化

- 新增 migration：`agent_quality_checks`、`agent_trust_scores`、`agent_learning_records` 三张表。
- 新增 repo table service：质量检查、信任分、学习记录的 create/list/count/update 能力。
- `WorkspaceApi` 新增：
  - `evaluate_agent_run`
  - `list_agent_quality_checks`
  - `submit_agent_run_feedback`
  - `list_agent_learning_records`
  - `list_agent_trust_scores`
  - `build_agent_learning_context`
- `web-server` 新增 HTTP 接口：
  - `GET /api/app/workspaces/{workspaceId}/runs/{runId}/quality-checks`
  - `POST /api/app/workspaces/{workspaceId}/runs/{runId}/quality-checks`
  - `POST /api/app/workspaces/{workspaceId}/runs/{runId}/feedback`
  - `GET /api/app/workspaces/{workspaceId}/learnings`
  - `GET /api/app/trust-scores`
- `web-server` SSE 在完成 `workspace_run_completed` 和 `workspace_artifact_created` 后自动触发质量检查，并发出 `quality_checked`。
- 新增错误码：`AGENT_QUALITY_CHECK_NOT_FOUND`、`AGENT_QUALITY_FORBIDDEN`、`AGENT_QUALITY_REQUEST_INVALID`、`AGENT_LEARNING_NOT_FOUND`、`AGENT_TRUST_NOT_FOUND`。

## 前端变化

- `AgentRuntimeEvent` 新增 `quality_checked` 类型。
- SSE 事件归一化支持 `quality_check` / `trust_update` / `learning_records` 到 camelCase 字段。
- 问答右侧时间线展示“质量检查完成”及通过/需改进状态。
- 虚拟员工工作区卡片展示质量分、信任分和学习记录数量。
- 中英文语言包已补齐质量检查相关文案。

## 用户体验变化

- 用户发送问题并收到回答后，右侧过程区会看到后端真实质量检查结果。
- 如果回答质量通过，用户看到质量通过、质量分、信任分和学习记录数量。
- 如果回答需要改进，右侧卡片会显示“需要改进”，让用户知道后端已经识别问题并沉淀改进经验。
- 用户反馈接口已经具备后端能力，后续前端可接入评分按钮或反馈入口。

## 用户可感知变化

用户在问答页发送问题后，回答结束时右侧“虚拟员工工作区”会新增质量检查状态卡。卡片会显示：

- 质量分，例如 `质量分：82`
- 信任分，例如 `信任分：60 / moderate`
- 学习记录数量，例如 `学习记录：1 条`

这些数据来自后端 SSE `quality_checked` 事件，不是前端静态展示。

## API 文档

- 已更新 `API_DOCS/app.md`：质量检查、反馈、学习记录、信任分 HTTP 接口。
- 已更新 `API_DOCS/sse.md`：`quality_checked` 事件和 `evaluate_agent_run` 持久化失败说明。

## 当前验证

- [x] `cargo check -p migration`
- [x] `cargo xtask migrate up`
- [x] `sea-orm-cli generate entity -o crates/repo/src/entity --with-serde both --date-time-crate time`
- [x] `cargo check -p repo`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `cargo test -p service`
- [x] `cargo test -p service quality_checker`
- [x] `cargo test -p service trust_ema`
- [x] `cargo test -p web-server sse`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] 前端 dev server HTTP smoke：`http://localhost:5173` 返回 200，Vite HTML 和 `src/main.tsx` 可加载

说明：Browser 插件在当前 Windows sandbox 下启动失败，错误为 `CreateProcessWithLogonW failed: 1326`，因此本次未完成浏览器自动点击截图验证；已用 HTTP smoke、前端 build 和 TypeScript 门禁验证页面包可加载。
