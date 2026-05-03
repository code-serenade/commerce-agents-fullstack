# @workspace/app-kit 协议

`@workspace/app-kit` 不是杂物层，也不是 `ui-components` 的失败收容站。

它的职责是承载“共享应用能力”：可复用页面、应用壳层、页面级复合组件和场景装配协议。

## 1. 宪章

- `@workspace/app-kit` 是应用级共享层，不只是组件层。
- 本包允许比 `ui-components` 更灵活，但不能没有边界。
- 本包可以使用 `ui-core`，但不能把 `ui-core` 的原始控制面直接泄露给最终 app。
- 本包的目标不是追求自由，而是把高价值场景沉淀成可复用框架件。

## 2. 本包适合承载什么

适合：

- data table
- admin sidebar
- top bar
- date-time picker
- profile 等跨 app 可复用页面
- login/register 认证视图与表单协议
- 其他明确服务于页面结构、列表页、后台场景、业务装配的共享能力

这些能力的共同点：

- 比基础组件复杂
- 带场景语义
- 适合多个 app 共享
- 需要内部装配多个下层组件

## 2.1 目录语义

- `src/components/*`：共享应用组件、壳层、复合控件。
- `src/pages/*`：共享页面。页面可以直接被 app 路由引用，但不得在包内依赖某个 app 的 store、route 或私有 API 聚合层。
- `src/pages/<page>/index.ts` 是页面公共入口。
- 页面若需要真实业务状态，应通过显式 props、controller hook 返回值或 app 侧适配器接入；不要在共享页面中偷偷绑定某一个 app 的 Redux、路由约定或 API 聚合。

### 认证页面规则

- `src/pages/login` 只承载共享登录视图、字段校验和表单组件装配。
- `src/pages/login` 不得直接调用登录 API、恢复权限、写 Redux、写 localStorage 或执行路由跳转。
- `web` 和 `admin` 的登录后业务流程不同，必须通过 `onSubmit` 等显式接口交给 app 侧处理。
- `src/pages/register` 可以承载共享 auth 账号注册页，但不得初始化某个具体业务 app 的用户身份。

## 3. 本包不适合承载什么

- primitive 原语
- 纯基础 UI
- 应该进入 `ui-components/stable` 的简单稳定组件
- 单个 app 私有的一次性页面结构
- 强绑定某个 app 私有权限、store、菜单或业务流程的页面
- 主题 token 与通用设计变量

## 4. API 原则

- 对外 API 应表达场景语义，而不是暴露底层细节。
- 可以比 `ui-components` 更灵活，但灵活性必须服务场景，而不是给 app 任意改结构。
- 禁止无边界 `...props` 透传。
- 禁止直接 re-export `ui-core` 组件或类型。
- 禁止通过透明包装把 `ui-core` 原样暴露给 app。

## 5. 与 ui-core / ui-components 的关系

- 优先组合 `ui-components` 提供的稳定协议组件。
- 必要时允许直接使用 `ui-core`。
- 但只要使用了 `ui-core`，就必须在本包重新定义清晰的场景协议。
- app 应消费的是 `@workspace/app-kit` 的语义 API，而不是它内部依赖了什么 primitive。

## 6. 语言与文案规则

- `@workspace/app-kit` 内禁止直接写死任何用户可见文案。
- 所有语言相关内容必须统一沉淀到 `packages/locales/lang/*/components.ts`。
- 组件内部只能消费 `@workspace/locales` 暴露的 copy getter，或消费业务层已翻译完成的文案 props。
- 禁止在组件源码里直接写中文、英文按钮文案、placeholder、label、empty text、dialog 文案。
- 禁止通过 `t("key", "fallback")` 在组件里留下英文 fallback 文案；已有 key 就应该直接从 `locales` 取值。
- 唯一例外是语言切换器的语言选项名；它们必须显示各自母语名，例如 `English`、`简体中文`，不跟随当前 locale 翻译。
- 允许保留语言无关的技术字符串，例如 className、DOM attribute 值、数据 key、日志 tag、标准格式标识符。

## 7. AI 默认判断规则

当 AI 需要新增共享能力时，按以下顺序判断：

1. 它是不是 primitive？
   是：去 `ui-core`

2. 它是不是简单、稳定、通用协议？
   是：去 `ui-components`

3. 它是不是复合装配、页面语义、后台场景协议或跨 app 可复用页面？
   是：来 `@workspace/app-kit`

4. 它是不是只给某一个 app 用？
   是：留在对应 app 内

5. 改完后运行：
   `pnpm -C packages/app-kit rules`
   `pnpm -C packages/app-kit typecheck`

## 8. 局部协议

若某个复合组件或共享页面已经成为框架核心件，必须在 `AGENT_PROTOCOL/protocols/components/` 中补充局部协议。

当前重点样例：

- `AGENT_PROTOCOL/protocols/components/data-table.md`

当任务涉及该组件本身，或需要在 app 中新增/修改其用法时，AI 必须继续阅读局部协议，而不能只看本包顶层协议。

## 9. 规则系统

- `tests/rules/*` 用于防止本包对外泄露 primitive 控制面。
- `tests/rules/*` 也应用于防止本包把语言文案直接写死在组件源码里。
- `tests/rules/ast-helpers.ts` 是当前规则基础设施的一部分；新增规则应优先继续使用 AST，而不是退回文本匹配。
- 这些规则的目标不是禁止本包内部实现灵活，而是禁止把这种灵活性直接外放给 app。
- 通过 rules 不等于设计已经完美，只等于当前已定义边界没有被打破。

## 10. 样例原则

- 本包组件首先是 AI 参考样例，其次才是功能集合。
- 重点不是数量，而是让 AI 学会：
  - 如何封装复合场景
  - 如何内部使用 `ui-core`
  - 如何不把底层能力泄露出去

## 11. 完成定义

改动只有在以下条件同时满足时才算完成：

- 组件归属判断正确。
- 没有把基础组件错误塞进本包。
- 没有对外泄露 `ui-core` 原始控制面。
- 如属于核心框架件，局部协议已同步。
- `pnpm -C packages/app-kit rules` 通过。
- `pnpm -C packages/app-kit typecheck` 通过。
