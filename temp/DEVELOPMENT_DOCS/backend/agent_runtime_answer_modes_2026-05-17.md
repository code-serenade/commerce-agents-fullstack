# Agent Runtime 回答模式与模型默认策略后端设计

## 需求摘要

用户要求把问答主链路的回答模式收敛为三种：

- `教学模式`：默认模式，面向学生和电商新手，保留干货但把概念、判断逻辑和下一步讲清楚。
- `快速简答`：更短、更直接，优先结论和最少必要动作。
- `丰富执行`：更完整的执行方案，适合复盘、汇报、落地计划和表格化清单。

岗位选择默认 `自动分配`，但仍保留 `指定单岗` 与 `指定多岗`。讲解深度改为 `低 / 中 / 高`，默认 `中`，并且必须真实影响 Agent 输入，而不是只改前端 UI。模型默认优先使用配置中的豆包；没有真实配置时不能伪造豆包选项。

## 数据库设计

本次不新增数据库表，不修改 migration。

原因：回答模式、讲解深度、岗位分配和模型选择都属于 Agent runtime request metadata 与 runtime options contract；当前已有会话消息 `contentJson.strategy` 可以保存这些字段。

## 接口设计

修改既有接口：

- `GET /api/app/agent/runtime-options`
- 需要登录态：是，沿用当前 app 用户启用校验。

响应字段保持兼容：

- `strategies`：改为 `learning` / `quick` / `execution` 三项，默认 `learning`。
- `responseModes`：同样返回三项，供前端或后续 app 消费。
- `learningLevels`：保留既有 ID `beginner` / `higher_vocational` / `professional`，显示文案改为 `低` / `中` / `高`，默认 `higher_vocational`。
- `collaborationModes`：`auto` 默认，保留 `single` 与 `manual`。
- `models` / `defaultModelName`：来自 `config/services.toml` 的真实 `[[llm]]`。默认模型选择规则为：
  - 如果 `default_llm` 配置有效，使用它。
  - 如果没有配置 `default_llm`，但存在 `name` 或 `model` 包含 `doubao` 的模型，默认选该模型。
  - 否则使用第一个 `[[llm]]`。

## Agent 行为设计

前端发送 SSE 时继续写入：

- `strategy.id`
- `strategy.response_mode`
- `strategy.collaboration_mode`
- `strategy.learning_level`
- `strategy.primary_role`
- `strategy.selected_roles`

后端 Agent 在 Commander 和 Worker cognition context 中加入 `runtime.answer_style` 事实：

- `learning`：要求学生友好、术语解释、步骤化、仍保留可执行动作。
- `quick`：要求短答案，少铺垫，限制为结论、关键理由、下一步。
- `execution`：要求完整方案、表格、负责人、验收指标、风险边界。

讲解深度加入 `runtime.explanation_depth` 事实：

- `beginner`：低，减少术语，补白话解释。
- `higher_vocational`：中，按电商岗位应用口径解释。
- `professional`：高，保留指标、推导、风险和约束。

自动岗位分配时，快速简答会倾向单岗短链路；教学模式和丰富执行仍允许后端根据问题分配多岗。手动单岗/多岗选择不被覆盖。

## 鉴权

沿用当前 `AuthUser` 与 `CommercePlaybookApi::ensure_current_user_enabled`，不新增权限。

## 错误与降级

- 未配置任何 `[[llm]]`：保持当前启动错误。
- 请求不存在模型：保持当前 `llm model '{name}' is not configured`。
- 未配置豆包：不返回假豆包选项，前端显示真实可用模型。

## 验证

- 后端：`cargo fmt`、`cargo test -p agent --lib`、`cargo check`、`cargo clippy`。
- 前端：`pnpm -C frontend/apps/web lint`、`test`、`rules`、`build`，并按协议运行根 `pnpm rules`。
- API 文档：同步更新 `API_DOCS/app.md`。
