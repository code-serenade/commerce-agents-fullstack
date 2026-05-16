# Agent Team Answer Quality And Process Binding 2026-05-16

## 需求摘要

本轮继续处理截图暴露的多智能体问答问题：

- 多岗位任务中，数据诊断、服务方案等支持岗位仍可能把“缺少真实指标、需要补数、需要复核”的可执行边界说明误判为失败。
- 最终回答不像电商虚拟员工团队交付：主回答岗位不清晰、协作岗位差异不明显、岗位之间的承接关系不够像真实团队。
- 未手动选择预设商品时，不能把历史商品、测试商品或默认商品注入为本轮商品上下文；用户只说“咖啡类商品”时，只能按类目/用户文本回答。
- 前端右侧任务过程必须绑定到每条回答，打开时不能让左侧正文滚到底，也不能压缩正文布局。

## 后端设计

- 继续使用新产品 agent runtime / task graph / SSE / workspace artifact 体系。
- 不新增数据库表。
- 不新增或修改 HTTP API。
- 不在 web-server 中直接调用 LLM。
- worker 规范化以任务图 assignment 的真实岗位为准，而不是 worker 实例默认角色，确保 `data`、`service`、`engineering` 等岗位都补齐自己的 findings、recommendations、risks 和 evidence。
- task graph 最终合成时优先选择“终端主岗”作为主回答岗位，例如运营整合节点，而不是默认使用第一个完成的支持岗位。
- 最终合成扫描岗位 evidence，明确区分：
  - 有 `context.selected_summary`：用户本轮手动选择了预设商品资料。
  - 无 `context.selected_summary`：未选择预设商品，只按用户文本、会话历史和工作区上下文理解问题。

## 前端对齐

- 每条 assistant 回答自己的过程卡片提供显式入口：查看这条回答的任务过程。
- 右侧过程面板只展示当前回答对应的 runtime events。
- 打开或关闭右侧过程面板时保存并恢复聊天滚动位置，避免用户查看中间某条回答时被拉到底部。
- 右侧过程面板作为覆盖式关联资料显示，不改变中间正文的宽度和位置。

## 验收

- 截图中的 `data role should provide at least two findings` 和 `service role output lacks service-specific content` 不应在正常可执行缺口说明中导致节点失败。
- 多岗位回答包含主岗结论、岗位分工、协同关系、执行清单和可沉淀交付物。
- 未选商品时最终回答必须明确“不注入默认预设商品资料”。
- 打开任意一条回答的任务过程，左侧正文滚动位置保持稳定。
- 后端运行 cargo fmt/check/clippy 和相关 agent 测试。
- 前端运行 `pnpm -C frontend/apps/web rules/test/lint/build`。
