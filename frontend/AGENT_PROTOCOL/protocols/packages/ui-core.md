# @workspace/ui-core 协议

本协议不是普通组件库说明书，而是给 AI 的底层边界文档。

`ui-core` 的唯一目标是提供 primitive 能力，让上层可以在不破坏行为契约的前提下继续封装。

## 1. 宪章

- `ui-core` 是 primitive 层，不是产品层。
- `ui-core` 负责行为契约、可访问性接线、结构基础与 token 挂接点。
- `ui-core` 不负责页面语义、业务语义、装配语义。
- AI 不得把 `ui-core` 当成“方便直接给 app 用的组件库”。

## 2. 目录真相

- `src/primitives/*` 是唯一有效实现目录。
- `src/components/*` 只保留 shadcn 原始生成结果或迁移参考，不是最终 API。
- `tests/rules/*` 是 `ui-core` 协议的自动检查层，不是普通单元测试。
- 推荐公共语义路径是 `@workspace/ui-core/primitives/*`。
- 兼容导出可以存在，但 AI 判断组件归属时必须以 `primitives` 语义理解本包。

## 3. 本包允许做什么

- 定义 primitive 组件的 DOM 结构与交互行为。
- 定义无障碍属性、键盘交互、状态挂接与基础样式钩子。
- 提供 `styled` 与 `primitive` 两种模式。
- 提供供上层二次封装所需的最小、清晰、稳定控制面。

## 4. 本包禁止做什么

- 禁止写页面装配逻辑。
- 禁止写业务语义组件。
- 禁止为了某个 app 的临时需求直接改 primitive 行为。
- 禁止把产品层视觉决策下沉到只能服务某个 app 的实现里。
- 禁止依赖 `ui-components`、`app` 或 `apps/*`。

## 5. AI 默认判断规则

当 AI 被要求新增或修改组件时，先判断：

1. 这是行为原语，还是页面/业务语义？
2. 外部是否需要保留较强控制权？
3. 这个能力是否应该被多个上层封装复用？

只有当答案明确指向“primitive 能力”时，才允许进入 `ui-core`。

以下情况通常属于 `ui-core`：

- button / input / dialog / popover / table 这类基础交互与结构原语
- 可访问性、焦点、状态、触发器、内容容器等行为基础件

以下情况不属于 `ui-core`：

- data table
- admin sidebar
- top bar
- 页面级 filter bar
- 带明确产品文案和装配结构的组件

## 6. 实现标准

- 所有组件必须遵循通用 `mode` 规范：`"styled" | "primitive"`。
- 默认 `mode` 必须是 `styled`。
- `primitive` 模式下不得注入设计系统样式与设计系统状态属性。
- `styled` 模式下可使用组件内部样式系统，但不得混入 app 私有决策。
- `mode` 分支必须在组件主体中显式处理，不允许把 styled/primitive 混在模糊工具函数里。
- `resolve*ClassName` 只服务 styled 分支。
- 公共 `index.ts` 只能显式导出，禁止 `export *`。

## 7. 新组件工作流

AI 在 `ui-core` 新增组件时，必须按以下顺序：

1. 优先通过 shadcn CLI 生成原始实现。
2. 将原始实现保留在 `src/components/*` 作为参考。
3. 在 `src/primitives/<component>/` 中重构为正式实现。
4. 拆分为 `*.tsx`、`*.styles.ts`、`*.types.ts`、`index.ts`。
5. 参考 `src/primitives/button` 组织 API。
6. 检查该能力是否被错误地做成了产品层语义。
7. 运行 `pnpm -C packages/ui-core rules`，确保结构与依赖边界合法。
8. 运行 `pnpm -C packages/ui-core typecheck`。
9. 通过相关门禁后，才允许对外导出。

## 8. 规则系统

- `ui-core` 的规则测试放在 `tests/rules/*`。
- 当前规则优先覆盖结构边界与依赖边界，再逐步扩展到更强的语义边界。
- 已有 AST 规则至少应保证：
  - primitive 目录结构完整
  - primitive `index.ts` 使用显式导出
  - primitive `index.ts` 只导出本地文件
  - `ui-core` 不依赖上层包
- 当协议与实现冲突时，以规则和本协议共同定义的边界为准。

## 9. 与上层的关系

- `ui-components` 可以消费 `ui-core`，但必须收紧 API。
- `@workspace/app-kit` 可以消费 `ui-core`，但不得把 primitive 控制面原样泄露给 app。
- `apps/*` 默认不应直接散落使用 `ui-core`。
- 当 app 确有例外需求时，应先在本地或 `@workspace/app-kit` 做一层协议化封装。

## 10. 样式原则

- 样式必须消费 token，不得硬编码产品色值。
- `ui-core` 可以有默认样式，但这些样式仍然只能表达 primitive 默认态，不能表达具体业务品牌策略。
- 颜色、状态、语义 token 必须来自共享主题体系。

## 11. 输出检查清单

- 组件确实属于 primitive 层。
- 实现位于 `src/primitives/*`。
- 没有引入页面语义或业务语义。
- `mode` 规范完整。
- 公共导出为显式导出。
- 没有依赖上层包。
- `pnpm -C packages/ui-core rules` 通过。
- `pnpm -C packages/ui-core typecheck` 通过。
