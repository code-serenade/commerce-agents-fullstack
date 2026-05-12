# Post 8-22：Coordination 状态查询真实化

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 22 个补强项：`Post 8-22`。

本次目标：

- 把 `coordination_status_query` 从 deferred 骨架升级为真实 Rust native skill。
- 让新产品多智能体问答链路具备“可查询当前协同状态”的后端原生能力。
- 不新增假前端入口，不引入数据库或平台写操作。

涉及范围：

- 产品内核：协同层从“有任务图和 SSE 事件”继续推进到“有真实状态查询 skill”。
- 后端 agent：`coordination_status_query` 改为 native deterministic 输出；`SkillContext` 透传更多 session/task/workspace/runtime metadata。
- 主问答链路：补充 chat / engineering 对状态类问题的 autosuggest 命中。
- 角色能力边界：允许 `chat` 角色使用 `coordination_status_query`，避免用户问“当前状态/进度”时只能落回普通闲聊。

本次实现：

- `backend/crates/agent/src/skills/coordination.rs`
  - 将 `coordination_status_query` 从 deferred 宏实例改为独立 native skill。
  - 输出当前可见的：
    - session 作用域
    - 当前 worker/role
    - task / graph / node / attempt
    - workspace / run / conversation 关联
    - product/workspace context readiness
    - blockers 与 observability limits
  - 明确能力边界：只报告当前 session 可见状态，不伪造全局集群和外部系统健康。
- `backend/crates/agent/src/agent/worker/runner.rs`
  - `build_skill_context(...)` 透传：
    - `message.user_id`
    - `message.workspace_id`
    - `message.agent_id`
    - `message.session_id`
    - `message.task_id`
    - `request.*` metadata
    - assignment input roles / count / rework instruction
  - 新增 `chat` 对状态类问题的 autosuggest。
  - `engineering` 新增状态/健康/进度意图命中 `coordination_status_query`，优先于通用 SLA fallback。
- `backend/crates/agent/src/roles/skill_catalog.rs`
  - 允许 `chat` 使用 `coordination_status_query`。

用户可感知变化：

- 用户问“当前任务状态”“现在运行到哪了”“workspace 状态怎么样”“系统健康如何”时，更容易看到后端命中真实 `coordination_status_query`。
- 过程面板后续看到的 skill 事件将来自真实后端状态查询，而不是泛化回答。
- 输出会明确说明：
  - 当前 session / worker / task / workspace 是否已关联
  - 上下文是否已加载
  - 哪些信息当前看不到

当前验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent autosuggest`
- [x] `cargo test -p agent executor_runs_representative_native_categories`
- [x] `cargo check -p agent`
- [x] `cargo check`
- [x] `cargo clippy`

后续缺口：

- 该 skill 目前仍是“当前 session 可见状态查询”，不是全局运行态监控。
- 若要查询数据库中的历史 run/task/approval 详情，还需要后续 service/API 聚合能力接入。
- 前端还未专门做 Kimi Agent 风格的“协同状态卡片”，后续可以直接消费这类真实 skill 输出。
