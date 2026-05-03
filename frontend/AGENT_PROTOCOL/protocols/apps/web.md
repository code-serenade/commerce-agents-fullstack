# apps/web 协议

`apps/web` 是业务应用壳层。

它负责把共享能力装配成真实业务页面，但不负责沉淀共享基础设施。

## 1. 宪章

- `web` 是最终业务 app。
- `web` 负责页面、路由、状态编排、业务交互。
- `web` 不是共享组件库，也不是通用服务库。

## 2. 默认消费顺序

AI 在 `web` 中开发页面时，必须按以下顺序查找能力：

1. `@workspace/app-kit`
2. `@workspace/ui-components`
3. `@workspace/services`
4. `web` 本地实现

只有在这些层都不适合时，才允许新增本地实现。

## 3. 本地目录职责

- `api/`：应用侧 API 聚合
- `components/`：`web` 私有组件
- `config/`：应用启动与 env 配置
- `layouts/`：布局
- `pages/`：页面
- `routes/`：路由
- `store/`：跨页面状态
- `test/`：应用级测试

新增代码必须优先进入正确目录，不得随意发明新层。

## 4. 强制边界

- 共享页面、壳层和 UI 优先来自 `@workspace/app-kit` / `ui-components`
- API 请求必须通过应用聚合层
- 通用错误模型与 query client 必须复用共享服务层
- 主题与 i18n 初始化统一放在应用入口

## 4.1 路由与认证布局

`web` 不是纯后台系统，不能用登录状态切换两棵完全不同的应用路由树。

默认规则：

- `/login`、`/register` 等认证页面使用 `AuthLayout`。
- `/login` 使用 `@workspace/app-kit/login` 的共享登录视图/表单协议；`web` 本地只保留登录后的业务流程。
- `/register` 使用 `@workspace/app-kit/register`，注册只创建共享 auth 账号。
- `web` 登录成功后需要按当前业务规则恢复 app user 权限，必要时初始化当前 app user；这个流程只能留在 `web` 的 API/状态编排里，不能塞进共享登录组件。
- 除认证页面外，公开页面和登录后页面都应使用同一套 app layout / shell。
- 页面是否需要登录应通过局部 route guard（例如 `RequireAuth`）控制，而不是在 `App.tsx` 中用 `isLogin ? protectedRoutes : publicRoutes` 替换整棵路由。
- 允许未登录访问的 app 页面必须在登录前后保持同一页面结构；登录状态只做内容增强、账户入口变化或私有区块显隐。
- 只有真正私有的页面，例如 profile、账户数据、用户私有资源，才放在 `RequireAuth` 下。

这和 `admin` 不同：`admin` 可以在未登录时完全隐藏后台壳层和业务路由。

## 5. 页面层禁止事项

- 禁止直接复制共享页面、壳层或组件的内部实现
- 禁止在页面里拼一套新的共享 UI 模式
- 禁止在登录页里重新实现共享登录表单结构；app 只接入 `AuthLoginView` 并提供业务 `onSubmit`
- 禁止因为赶进度绕过现有分层
- 禁止让 `web` 内部长出第二套共享组件系统

## 6. 对 AI 的要求

- `web` 里的代码首先要像“应用装配代码”，而不是“组件库源码”
- 若某段实现未来明显可复用，应停止继续写在 `web`，回到 `packages/*` 重判归属
- 当共享层能力不足时，应优先补共享层，而不是在页面里打补丁

## 7. 门禁

涉及 `apps/web` 的改动至少通过：

- `pnpm -C apps/web lint`
- `pnpm -C apps/web test`
- `pnpm -C apps/web rules`
- `pnpm -C apps/web build`
