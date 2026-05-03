# @workspace/ui-theme 协议

## 宪章

`@workspace/ui-theme` 是全仓库唯一的主题语义变量来源（单一事实源）。

所有应用共享的颜色、表面、边框、状态、图表、半径等语义 token，必须由本包定义并对外导出。

## 强制规则

- `ui-theme` 必须提供完整的 light/dark/system 可解析语义变量集合。
- `ui-theme` 必须通过单一样式入口对外导出（`./styles.css`）。
- `ui-core` 允许消费这些语义变量，并且必须维护一套最小可用的兜底主题变量。
- `ui-core` 的兜底变量用于容错与可用性保障，不作为业务主题定制入口。
- 当新增语义变量超出 `ui-core` 已约定语义集合时，必须在 `ui-components` 中重写/扩展组件实现来适配，禁止通过修改 `ui-core` 处理。
- 应用包必须同时引入 `@workspace/ui-theme/styles.css` 与 `@workspace/ui-core/styles.css`。

## 依赖关系（强制）

- `ui-theme`：定义主题变量与主题模式映射。
- `ui-core`：消费变量实现组件基础样式与状态规则，并提供最小兜底 token。
- `ui-components`：基于 `ui-core` 做产品级封装。
- `apps/*`：统一消费 `ui-theme + ui-core (+ ui-components)`，不得绕过。

## 完成标准

- 新变量在 `ui-theme` light/dark 主题中都有定义。
- `ui-core` 对应新增最小兜底 token，且命名与语义保持一致。
- 应用入口已引入 `@workspace/ui-theme/styles.css`。
- 主题切换下组件表现一致，无变量缺失导致的回退样式。
