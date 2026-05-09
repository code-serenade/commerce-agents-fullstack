# Post 8-2：问答运行反馈前端接入

状态：done

计划序号：基础 8 项已完成后的第 2 个前端补强项。

## 本次目标

- 把 Phase 5 已有的用户反馈后端接口接入问答工作台。
- 用户在一次后端 workspace run 形成后，可以对本轮回答评分并提交文字反馈。
- 前端只对真实 `workspace_run_linked` / `workspace_run_completed` 事件开放反馈入口，不伪造运行记录。

## 产品内核底层变化

- 质量闭环从“后端自动质量检查”补强为“用户反馈 -> 后端学习记录”。
- 反馈结果进入已有 `agent_learning_records`，和质量、信任、学习上下文处于同一闭环。
- 本次未新增模型调用、未新增数据库表、未改变 agent runtime，只把已有后端能力暴露给用户操作。

## 后端变化

- 未新增或修改后端接口。
- 复用已有接口：`POST /api/app/workspaces/{workspaceId}/runs/{runId}/feedback`。
- 前端 API client 兼容响应中的 `learningRecord` 与 `learning_record`，避免 DTO 序列化形态差异导致页面读不到结果。

## 前端变化

- `frontend/apps/web/src/api/commerce.ts` 新增运行反馈 API client 与类型。
- `agent-chat-data.ts` 把反馈接口纳入问答页数据入口。
- 新增 `agent-run-feedback-view-model.ts`，从 SSE runtime event 推导当前可反馈的 workspace/run。
- 新增 `use-agent-run-feedback.ts`，管理评分、评论、提交中、已提交和错误状态。
- 新增 `AgentRunFeedbackPanel`，挂到右侧虚拟员工工作区，复用 `@workspace/ui-components` 的 `Button`、`Textarea`、`Badge`、`toast`。
- 补充中英文文案。

## 用户体验变化

- 用户发送问题并形成后端运行记录后，右侧虚拟员工工作区出现“本轮反馈”。
- 用户可以选择 1 到 5 分，并填写可选反馈说明。
- 回答仍在生成时，反馈按钮会明确禁用。
- 没有后端运行记录、未登录或本轮已提交时，页面会说明原因。

## 用户可感知变化

- 在问答页右侧工作台下方，用户能看到当前反馈目标和 run 短 ID。
- 点击星级后，再点“提交反馈”，成功后显示“已提交”，并弹出“反馈已写入学习记录”。
- 这不是前端本地标记，提交成功来自真实后端接口。

## 验证

- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`
- [x] 真实 HTTP smoke，使用测试账号创建临时 workspace/run，调用反馈接口返回 `feedback_positive` learning record，临时 workspace 已归档。

## 测试注意

- 冒烟测试第一次使用 `completed` 作为运行状态，后端按契约拒绝并返回 `invalid status: completed`。
- 复核后确认 workspace run 合法状态为 `planned/running/succeeded/failed/cancelled/archived`，改用 `succeeded` 后通过。
- 后端反馈响应的嵌套学习记录字段保留 snake_case，例如 `feedback_rating`；前端 API client 已归一为 `feedbackRating`，并继续兼容 `learningRecord` 与 `learning_record`。
