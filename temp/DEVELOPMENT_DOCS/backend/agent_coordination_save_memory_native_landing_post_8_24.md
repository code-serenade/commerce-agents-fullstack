# Post 8-24：Coordination 保存工作记忆真实落地

## 1. 需求摘要

继续按旧产品能力迁移计划推进，在新产品后端里把 `coordination_save_memory` 从骨架能力升级为真实可用能力。

目标不是新增一套独立的 memory 系统，而是用新产品现有架构，把“将事实 / 决策 / 洞察 / 风险保存为后续可复用工作记忆”的能力落到真实后端链路中。

约束：

- 继续以后端优先。
- `agent` 不直接写数据库。
- 优先复用现有 `agent_skill_runs`、`agent_learning_records`、SSE 和 workspace 归属校验。
- 不新增假前端状态。

## 2. 业务理解

旧产品里的“保存记忆”本质上不是聊天摘要，而是：

1. 用户或 Agent 明确识别出一个值得复用的事实 / 决策 / 洞察 / 风险。
2. 该内容需要绑定到当前工作区语境。
3. 后续问答应能再次加载这些记忆，避免重复说明。
4. 同一个 key 的记忆应支持覆盖更新，而不是无限追加重复记录。

在新产品里，这类能力更适合落到：

- `agent`：产出结构化 memory draft
- `service`：做 workspace / run / skill run 归属校验与 upsert 语义
- `agent_learning_records`：承接当前阶段的工作记忆持久化
- `web-server SSE`：在 skill result 后触发真实沉淀，并回传事件

## 3. 数据落点

本轮不新增表，复用现有：

- `agent_skill_runs`
  - 继续作为 skill 请求 / 结果 / 沉淀关联的审计主链
  - 在 metadata 中记录 `learningRecordId`、`workspaceMemoryKey`、`workspaceMemorySaved`

- `agent_learning_records`
  - 作为当前阶段的 workspace memory 承载表
  - 只使用 `status=active`
  - `role` 统一落为共享 `agent`
  - `category` 复用 `fact / decision / insight / warning`
  - `source_action` 使用固定来源，例如 `workspace_memory_manual`
  - `metadata` 保存：
    - `kind=workspace_memory`
    - `memoryKey`
    - `memoryType`
    - `source=agent_skill_result`
    - `sourceRole`
    - `requestId`
    - `skillId`
    - `runId`
    - `workspaceId`
    - `upsertMode=create|update`

## 4. HTTP / SSE / 鉴权

### 4.1 HTTP API

本轮不新增外部 HTTP API。

### 4.2 SSE

沿用现有 `POST /api/sse/agent/session/stream`。

新增一个工作区沉淀事件：

- `agent_learning_record_created`

用途：

- 表示 `coordination_save_memory` 的结果已真实沉淀为一条 learning / workspace memory 记录。
- 返回结构复用 `AgentLearningRecordResponse`，字段走 camelCase。

### 4.3 鉴权

继续沿用现有 app user + workspace/run 归属校验：

1. 先校验当前 JWT 用户。
2. 校验 workspace 属于当前用户。
3. 校验 run 属于当前 workspace。
4. skill run / learning record 的读写都绑定到当前用户自己的 workspace。

## 5. 后端流程

### 5.1 agent native skill

`coordination_save_memory` 改为 deterministic native skill：

输入：

- `key`
- `content`
- `memory_type`

输出：

- `status=success`
- `memory_save.should_persist=true`
- 规范化后的 `key / content / memory_type / workspace_id / role / source`

不直接写库，只返回 draft。

### 5.2 SSE 持久化

web-server 在收到 `skill_result` 后：

1. 先按既有链路写 `record_agent_skill_result`
2. 再调用新的 `WorkspaceApi::materialize_agent_workspace_memory`
3. 由 service：
   - 解析 skill output
   - 校验 skill id 必须为 `coordination_save_memory`
   - 按 `workspace + key + source_action=workspace_memory_manual` 查找已有记录
   - 同 key 时 update
   - 不存在时 create
   - 回写当前 `agent_skill_run.metadata`
4. 成功后 SSE 发出 `agent_learning_record_created`

### 5.3 后续问答加载

当前 SSE 启动时构建 learning context：

- 增加 `workspace_id` 维度，优先读取当前工作区下的 learning records
- 若当前工作区没有记录，再回退到全局 `agent` 级学习记录
- 将 learning summary 继续注入 runtime metadata，并补充到 `context.summary`

这样本轮新保存的 workspace memory，后续同工作区问答可直接进入上下文。

## 6. 兼容与边界

本轮明确不做：

- 不新增独立 `workspace_memory` 表
- 不做长期冲突治理、策略修剪、跨会话 profile memory
- 不做前端记忆面板重构
- 不做 OCR / 外部知识抽取联动

本轮解决的是：

- “记住这个事实 / 决策 / 风险”能在新产品后端真实生效
- 同 key 支持覆盖更新
- 记忆能在后续 workspace 问答中再次被加载

## 7. 开发顺序

1. `temp/DEVELOPMENT_DOCS/backend/` 写本设计文档
2. `agent`：`coordination_save_memory` native 化 + autosuggest 补强
3. `repo/service`：补 learning record update 与 memory materialize
4. `web-server`：SSE 调用 materialize 并发事件
5. `API_DOCS/sse.md` 同步
6. 迁移记录与矩阵同步
7. `cargo fmt`
8. 相关测试、`cargo check`、`cargo clippy`
