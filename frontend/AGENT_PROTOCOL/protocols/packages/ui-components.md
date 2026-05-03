# @workspace/ui-components 协议

本包是整个框架最关键的约束层。

它的职责不是“提供很多组件”，而是“给 AI 提供一组稳定、固定、不可随意漂移的前端协议”。

## 1. 宪章

- `ui-components` 是 stable 协议层。
- 外部 app 默认应优先消费本包，而不是直接消费 `ui-core`。
- 本包的首要目标是减少 AI 乱写，不是提高灵活性。
- 任何会削弱边界的便利性写法，都应默认视为不合法。

## 2. 本包在整个框架中的位置

- `ui-core`：primitive 能力层
- `ui-components`：稳定、固定、可复用的产品级协议层
- `app`：复合装配层
- `apps/*`：最终应用层

AI 在判断归属时，若组件是“简单、稳定、通用、格式清晰”的共享能力，应优先进入本包。

## 3. 目录规则

- `src/components/stable/*`：正式稳定协议组件
- `src/components/labs/*`：实验组件

禁止：

- 把组件直接写到 `src/components/*` 根目录
- 恢复 `custom` 这一层
- 把复合装配组件伪装成 stable 组件塞进本包

## 4. Stable 的真正含义

这里的 stable 不是“功能很多”，而是：

- API 边界稳定
- 使用方式稳定
- DOM 结构和交互入口稳定
- 样式控制权不向外扩散
- AI 在修改时不需要重新发明协议

如果一个组件离开了固定结构、固定触发方式、固定样式边界就无法成立，那它不属于 stable。

## 5. Stable 组件强制规则

- 必须使用显式、受控的 props。
- 禁止暴露 `className`、`style`、自由形态 `...props`。
- 禁止暴露 `mode`、`asChild`、`classResolver`、`classNameMode` 或同类 primitive 控制面。
- 禁止暴露 render prop、slot 注入、结构改写入口。
- 禁止直接透传 `ui-core` props。
- 禁止直接返回 `<CoreXxx {...props} />` 这类透明包装。
- 禁止通过“先包一层再原样导出”的方式伪装稳定组件。

## 6. 什么应该进入本包

通常适合进入 `ui-components` 的能力：

- button
- input
- checkbox
- stable table
- stable dialog
- alert
- badge

这些组件有共同特征：

- 页面到页面之间格式基本一致
- 不需要大量页面语义定制
- 可以用固定协议描述清楚

## 7. 什么不应该进入本包

以下能力通常应进入 `@workspace/app-kit`：

- data table
- admin sidebar
- top bar
- 页面 filter/header 组合
- 明显依赖场景语义的 tooltip 包装
- 任意需要大量结构或样式逃生口的组件

一句话判断：

- 若组件本质是“规范”
  - 进入 `ui-components`
- 若组件本质是“场景装配”
  - 进入 `@workspace/app-kit`

## 8. AI 默认工作流

AI 在修改或新增本包组件时，必须按以下顺序：

1. 先判断该能力是否真的属于 stable。
2. 阅读本包协议与同类组件样例。
3. 优先参考现有合法 stable 组件写法。
4. 只暴露最小白名单 API。
5. 写完后运行：
   - `pnpm -C packages/ui-components rules`
   - `pnpm -C packages/ui-components typecheck`

如果 AI 发现自己需要加：

- `className`
- `style`
- `asChild`
- render props
- Core props alias
- 透明 passthrough

应立即停止，并重新判断该能力是否应该移到 `app` 或 `ui-core`。

## 9. 导出规则

- 根导出 `@workspace/ui-components` 只导出 stable 组件。
- `labs` 只能通过显式子路径导出。
- 包导出列表必须保持极小且明确。
- 不允许恢复模糊目录出口。

## 10. 规则与样例

- `tests/rules/*` 是本包协议的自动执行层。
- `tests/rules/ast-helpers.ts` 是当前规则基础设施的一部分；新增规则应优先继续使用 AST，而不是退回文本匹配。
- 现有 stable 组件的价值首先是“给 AI 提供样例”，其次才是“提供功能”。
- 新组件数量不是目标。
- 高质量、边界清晰、可作为 AI 参考的样例才是目标。

## 11. 完成定义

只有同时满足以下条件，改动才算完成：

- 组件归属判断正确。
- API 没有泄露 primitive 控制面。
- 没有新增灵活逃生口。
- `pnpm -C packages/ui-components rules` 通过。
- `pnpm -C packages/ui-components typecheck` 通过。
