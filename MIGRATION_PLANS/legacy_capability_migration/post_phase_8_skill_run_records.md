# Post 8-20：Skill run 记录查询 API

## 状态

done

## 本轮定位

旧产品能力迁移主体计划已完成 `12/12`。本轮是主体计划后的第 20 个补强项：把旧产品 skill run audit 能力迁成新产品后端可持久化、可查询的 Agent skill run 记录。

## 产品内核变化

- Skill 调用不再只停留在 SSE 流式过程里。
- 每次 `skill_requested` 和 `skill_result` 都可以沉淀为后端记录。
- 后续前端可以在工作区运行详情里查看历史工具调用、状态、风险、审批和输出摘要。

## 后端变化

- 新增 `agent_skill_runs` 表，记录 skill 请求和结果。
- SSE 处理链路接入 skill run 记录写入。
- 新增 workspace run 下的 skill run 查询 API。
- 支持按状态、skillId、role 和分页查询。
- `skill_requested` 先写 `requested` 记录；`skill_result` 按 `requestId` 更新结果，缺失请求记录时补写结果记录。

## 前端变化

- `apps/web/src/api/commerce.ts` 同步 skill run 查询响应类型。
- 新增 workspace run skill runs 查询 API adapter。
- 本轮不新增页面入口，后续工作区详情页可直接接入。

## 用户可感知变化

- 后续前端接入后，用户可以在某次 Agent 工作区运行里看到调用过哪些 skill。
- 用户能区分 skill 是 native、adapter、deferred 还是 pending approval。
- 当某个 skill 失败或需要审批时，用户可以追踪具体 requestId、角色、风险等级和摘要。

## 当前验证

- [x] `cargo fmt`
- [x] `cargo fmt --check`
- [x] `cargo test -p service skill_run`
- [x] `cargo check -p repo`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

## 后续缺口

- 还需要在工作区运行详情页接入 skill run 历史列表。
- 还需要把审批、搜索证据 artifact 与 skill run 记录做前端关联展示。
