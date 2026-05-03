# apps 协议

`apps/*` 是最终消费层，不是自由发挥层。

本目录的目标是让 AI 在真实应用中按框架装配代码，而不是在页面里重新发明组件库、表格系统、布局系统或服务层。非程序员应该能通过需求描述驱动 app 开发，而不需要识别 AI 是否正在闭门造车。

## 1. 总原则

app 只负责最终应用装配：

- 路由
- 页面目录
- 页面数据 hook
- 应用侧 API 聚合
- app shell wiring
- app 私有 store
- app 私有业务流程

app 不负责沉淀共享能力：

- 不写通用 UI primitive
- 不写第二套 stable components
- 不写 DataTable / MetricCards / shell / toast / route feedback 的本地替代品
- 不复制 `packages/*` 的内部实现
- 不因为页面着急就绕过共享层

## 2. App 开发硬流程

AI 在 `apps/*` 中新增或修改页面、布局、组件、表单、表格、导航、反馈状态前，必须按顺序执行：

1. 读 `AGENT_PROTOCOL/PROTOCOL.md`。
2. 读本文件。
3. 读目标 app 的 `AGENT_PROTOCOL/protocols/apps/<app>.md`。
4. 如果是页面，读 `AGENT_PROTOCOL/protocols/page-types/page-directory-structure.md` 和 `AGENT_PROTOCOL/protocols/page-types/page-index-usage.md`。
5. 如果涉及特定页面类型，读对应页面协议，例如 `shared-query-page`、`datatable-local-usage`、`metric-cards-usage`。
6. 检查共享库存：`@workspace/app-kit`、`@workspace/ui-components`、`@workspace/services`、`@workspace/locales`、`@workspace/ui-theme`。
7. 找到合法参考实现后再动手。
8. 如果没有合法能力，先判断应补哪个 package，而不是直接在 app 中私造。

这不是建议，是 app 层开发顺序。

## 3. 共享库存优先级

app 中需要 UI 或页面装配时，默认消费顺序是：

1. `@workspace/app-kit`
   用于页面级和场景级能力：`DataTable`、`MetricCards`、`SidebarShell`、`TopBar`、`PageLoading`、`RouteErrorBoundary`、`PageErrorState`、`AppNotice`、`CopyableText`、profile page、date/time picker、file upload 等。

2. `@workspace/ui-components`
   用于稳定基础产品组件：`Button`、`Input`、`Select`、`Dialog`、`Card`、`Badge`、`Tabs`、`Table`、`ToastProvider`、`toast`、`TreeView`、`SearchInput` 等。

3. `@workspace/services`
   用于请求、query、错误模型、i18n 初始化等共享服务能力。

4. app 本地实现
   只允许承载当前 app 私有业务组合。

AI 不得默认从 `@workspace/ui-core` 起手写 app 页面。app 若需要大量直接使用 primitive，通常说明能力应该先进入 `ui-components` 或 `app-kit`。

## 4. 写本地代码前的库存检查

在 app 中新增以下任意内容前，必须先检查已有出口和样例：

- button/input/select/dialog/card/table/tabs/toast/tree/search 等基础 UI
- data table、metric cards、filter header、row actions、bulk actions
- sidebar、top bar、mobile nav、route loading/error boundary
- profile、upload、copyable text、date/time picker
- 表单布局、空状态、错误状态、通知状态

检查路径：

- `packages/ui-components/package.json#exports`
- `packages/ui-components/src/index.ts`
- `packages/ui-components/src/components/stable/*`
- `packages/app-kit/package.json#exports`
- `packages/app-kit/src/index.ts`
- `packages/app-kit/src/components/*`
- `packages/app-kit/src/pages/*`
- `showcases/guide` 中的合法示例

如果已有能力能覆盖 70% 以上需求，应优先使用已有能力。缺少的 30% 如果具有复用价值，应补共享包，而不是在 app 页面旁路实现。

## 5. 允许留在 app 的内容

以下内容可以留在 app：

- 只服务该 app 的页面目录
- 页面级数据查询与业务映射
- 当前 app 的 API 聚合入口
- 当前 app 的路由、导航配置、权限映射
- 当前 app 的 store slice
- 当前页面私有的 table columns、query fields、row actions
- 当前页面私有的 dialog 内容和提交逻辑
- 当前页面私有的业务说明块

但即使留在 app，也必须按页面协议拆分，不允许堆进单个 `index.tsx`。

## 6. 必须上移的信号

出现以下任一信号，AI 必须暂停在 app 内扩写，并重新判断 package 落点：

- 第二个 app 也会用
- 第二个页面也会复制同一结构
- 你正在命名一个通用组件 API
- 你正在写与业务无关的视觉/交互基础件
- 你正在复制 app-kit 或 ui-components 的内部结构
- 你为了一个页面重写 table header、pagination、row action、toast、shell、loading、empty、error 等通用模式
- 你需要暴露 `className`、`style`、slot、render prop 来维持所谓灵活性

上移方向：

- primitive 行为：`packages/ui-core`
- 简单稳定产品组件：`packages/ui-components`
- 页面级/场景级复合能力：`packages/app-kit`
- 请求、错误、query、i18n 等服务：`packages/services`
- 文案和翻译：`packages/locales`
- 本地服务端联调：`packages/services` + app `.env.*` dev proxy

## 7. 页面目录规则

app 页面不是单文件实现。

页面工作必须继续遵守：

- `AGENT_PROTOCOL/protocols/page-types/page-directory-structure.md`
- `AGENT_PROTOCOL/protocols/page-types/page-index-usage.md`

如果使用 `DataTable`、`MetricCards` 或共享查询模式，还必须继续遵守对应页面类型协议。

页面入口 `index.tsx` 的目标是让人和 AI 看见页面结构，不是承载全部逻辑。

## 8. 本目录下 app 的角色

- `apps/web`：真实业务应用。允许公开页面和登录后页面共用 app shell。
- `apps/admin`：商用后台应用。未登录时不暴露后台 shell 和业务路由。
- `apps/rules`：app 层共享规则测试包，不是业务 app。

## 8.1 认证页面边界

认证页面不是 app 随手重写表单的例外。

默认规则：

- 登录表单结构、字段校验、Field/Input/Button 装配优先来自 `@workspace/app-kit/login`。
- 注册页面优先来自 `@workspace/app-kit/register`。
- app 登录页只保留登录后的业务编排，例如调用哪个 API、写入哪个 store、加载哪套权限、跳转到哪里。
- `web` 和 `admin` 的登录 API/权限恢复流程不同，不能合并成同一个绑定 API 的共享页面。
- 共享层只能提供 `AuthLoginView` 这类视图/表单协议，不得在 `app-kit` 中偷偷绑定某个 app 的私有 API、Redux slice 或权限模型。

app 登录页禁止重新实现：

- `react-hook-form` 表单 wiring
- `zodResolver` schema 接入
- `Input + label + error` 的本地表单结构
- 注册/登录共享视觉骨架

## 9. 共享 rules

app 层通用规则测试统一放在 `apps/rules/*`。

规则：

- 单个业务 app 不再维护自己的 `tests/rules/*` 副本。
- 每个业务 app 的 `package.json` 都应提供 `rules` 脚本，并指向 `apps/rules`。
- 新增业务 app 时，默认接入同一套 app rules。
- 只有确实属于某 app 私有协议时，才新增本地规则。

## 10. AI 交付前自检

涉及 app 的非平凡改动，AI 完成前必须能回答：

1. 我读了哪些协议？
2. 我检查了哪些共享出口？
3. 为什么这段代码应该留在 app，而不是进入 package？
4. 我有没有复制已有共享能力的内部结构？
5. 如果下一个页面要同样能力，它应该复用哪里？
6. 我运行了哪些门禁？

如果回答不了，说明实现还没有进入框架轨道。

## 11. 完成定义

只有同时满足以下条件，app 改动才算完成：

- 分层判断正确。
- 优先复用了 `app-kit`、`ui-components`、`services` 等现有能力。
- 没有把共享能力错误地下沉到 app。
- 没有让 app 长出第二套组件系统。
- 页面按页面协议拆分。
- 对应 app 的本地门禁通过。
