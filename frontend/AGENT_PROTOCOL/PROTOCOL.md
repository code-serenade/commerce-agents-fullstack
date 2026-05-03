# Codex 协议入口

本文件是仓库给 AI 的第一份工作契约。它故意保持短小、可执行。

项目目标不是让 AI 自由写 React，而是让不懂代码但聪明的使用者，通过一组框架协议约束 AI，得到可以商用的代码。

## 不可跳过的工作流

任何回答只要涉及架构建议、文件落点、实现策略或代码修改，AI 都必须按顺序执行：

1. 判断目标层级。
2. 阅读 `AGENT_PROTOCOL/protocols/**` 下相关协议。
3. 先检查现有导出和参考实现，再决定是否写新代码。
4. 选择最小合法的现有组件或 app-kit 协议。
5. 只有 app 私有业务装配才写在 app 本地。
6. 按触碰到的协议运行最近的 package scripts。

第 3 步是最常见失败点。不要从 JSX 起手；必须从框架库存起手。

## 分层判断闸门

每次都按这个顺序判断：

1. `packages/ui-core`
   primitive 行为、无障碍接线、底层结构。App 几乎不应直接修改或消费这一层。

2. `packages/ui-components`
   稳定产品组件，API 受限制。这是 app 页面使用基础 UI 的默认来源。

3. `packages/app-kit`
   共享应用装配能力：DataTable、MetricCards、shell、navigation surface、route feedback、profile page、date/time picker、file upload、copyable text 等。

4. `packages/services`、`packages/locales`、`packages/ui-theme`
   共享非页面能力。

5. `apps/*`
   最终装配层：路由、页面组合、app API 聚合、app 状态、app 私有业务流程。

如果某段代码可能服务第二个 app 或第二类页面，先评估 package 归属，不要直接藏进 `apps/*`。

## 必须检查共享库存

创建组件、布局、表格、表单控件、toast、shell、页面脚手架、picker、upload UI 或反馈状态前，必须先检查：

- `packages/ui-components/package.json#exports`
- `packages/ui-components/src/index.ts`
- `packages/ui-components/src/components/stable/*`
- `packages/app-kit/package.json#exports`
- `packages/app-kit/src/index.ts`
- `packages/app-kit/src/components/*`
- `packages/app-kit/src/pages/*`
- `showcases/guide` 中的合法用法示例

如果现有能力能覆盖 70% 以上需求，优先使用现有能力。若只差少量可复用能力，应改进共享包，而不是在 app 内 fork 一套模式。

## App 代码规则

App 代码应该像装配代码：

- 路由接线
- 页面目录模块
- API adapter 调用
- query/data hook
- 业务表格 columns 与 actions
- 业务 dialog 内容
- app shell 接线

App 代码不应变成组件库：

- 不写第二套 design system
- 不写通用 UI primitive
- 不复制 `ui-core`、`ui-components` 或 `app-kit` 内部实现
- 不在本地替代 DataTable、MetricCards、TopBar、SidebarShell、ToastProvider、route loading/error state 或稳定表单控件

## 遇到历史违规代码时

历史违规不是模板。

如果触碰区域存在违规，并且小范围内可修正，就顺手往协议方向收敛。若修复会扩大任务，必须说明不匹配点，并避免继续新增同类违规。

## 协议阅读地图

始终阅读最接近任务的协议：

- App 工作：`AGENT_PROTOCOL/protocols/apps/index.md`，再读 `AGENT_PROTOCOL/protocols/apps/<app>.md`
- 页面工作：继续读匹配页面类型的 `AGENT_PROTOCOL/protocols/page-types/*.md`
- DataTable 使用：`AGENT_PROTOCOL/protocols/components/data-table.md`
- Stable 组件工作：`AGENT_PROTOCOL/protocols/packages/ui-components.md`
- App-kit 工作：`AGENT_PROTOCOL/protocols/packages/app-kit.md`
- Primitive 工作：`AGENT_PROTOCOL/protocols/packages/ui-core.md`
- Service/theme/i18n 工作：读 `AGENT_PROTOCOL/protocols/packages/` 下对应文件

## 重要改动前必须说明

非平凡改动在编辑文件前，必须简短说明：

- 目标层级
- 已阅读协议
- 已检查共享库存
- 为什么实现应该落在这里

这段说明可以很短，但必须有。它用于阻止闭门造车。
