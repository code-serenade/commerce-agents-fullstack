# 用户文档

这个目录放的是写给使用者看的文档。

这里的目标不是教你写代码，而是帮助你更稳定地和 AI 协作，让 AI 按这个项目现有框架来回答问题和开发。

## 建议阅读顺序

1. [framework-thinking.md](./framework-thinking.md)
   先理解这个框架为什么分成 `apps` 和 `packages`，以及为什么 AI 需要先判断落点。

2. [web-app-guide.md](./web-app-guide.md)
   了解当前主业务应用 `apps/web` 的用途、运行方式、环境文件约定和适用场景。

3. [env-setup.md](./env-setup.md)
   先看本地 `dev` 怎么启动，以及它需要哪些本地后端服务和环境变量。

4. [dev-checklist.md](./dev-checklist.md)
   如果你要跑本地 `pnpm dev` 联调，先看这份清单。

5. [task-template.md](./task-template.md)
   直接复制任务模板给 AI，用来提问题、提需求、报 Bug。

6. [design-input-spec.md](./design-input-spec.md)
   如果你要按设计稿开发，先看这份输入规范，避免信息不完整。

7. [acceptance-checklist.md](./acceptance-checklist.md)
   AI 完成后，用这份清单做验收。

## 说明

- 这里只保留中文版文档。
- 根目录 [README.md](../README.md) 负责总入口说明。
- AI 执行规则不在这里，而在 [AGENT_PROTOCOL/PROTOCOL.md](../AGENT_PROTOCOL/PROTOCOL.md)。
- 当前共享 UI 默认按分类子路径消费，例如 `@workspace/ui-components/stable/button`。
- 当前通知能力默认通过 `@workspace/ui-components/stable/toast` 使用。
- 当前本地联调主要依赖 `VITE_*_PROXY`，不再使用 `VITE_*_URL` 作为请求前缀拼接配置。
- `pnpm dev` 会读取根目录共享 env 和当前 app 自己的 `.env.development`。
- `pnpm env:init` 会从根目录 `env.shared.example` 生成根目录共享 env 和两个 app 的 API env。
