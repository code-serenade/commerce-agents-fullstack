# 本地环境说明

这个文档是给使用者看的。

当前框架不再提供浏览器 mock 模式。前端开发默认通过 `pnpm dev` 对接本地服务端。

对于不会命令行的使用者，最重要的一条是：

- 先把模板地址交给 AI
- 让 AI 先完成下载、依赖安装和环境检查
- 环境没准备好之前，不要先让 AI 直接改业务代码

## 建议的开始方式

推荐顺序：

1. 优先用 `npx` 下载模板
2. 让 AI 先安装依赖并检查环境
3. 启动本地后端服务
4. 检查 `env.shared.example` 里的开发和生产代理地址
5. 执行 `pnpm env:init` 生成根目录和 app 目录的 env 文件
6. 运行 `pnpm dev`

如果你自己不会下载模板，也可以直接把仓库地址发给 AI，让 AI 先执行这件事。

## 本地 dev 联调

适合：

- 你要联调本地或测试后端
- 你要验证真实接口行为
- 你要检查真实登录、权限、文件上传等流程

常用命令：

```bash
pnpm install
pnpm dev
```

如果只启动某一个 app，可以让 AI 根据当前脚本选择合适命令，例如：

```bash
pnpm -C apps/web dev
pnpm -C apps/admin dev
```

## 环境文件

模板源是根目录 `env.shared.example`。

默认读取的环境文件是：

- 根目录 `.env.development`
- 当前 app 的 `.env.development`

生产构建对应读取：

- 根目录 `.env.production`
- 当前 app 的 `.env.production`

可以用下面命令从 `env.shared.example` 初始化 6 个本地文件：

```bash
pnpm env:init
```

生成结果是：

- 根目录 `.env.development`
- 根目录 `.env.production`
- `apps/admin/.env.development`
- `apps/admin/.env.production`
- `apps/web/.env.development`
- `apps/web/.env.production`

根目录 env 只放共享服务代理：

```env
VITE_AUTH_PROXY=http://localhost:9000
VITE_FILE_PROXY=http://localhost:9000
```

每个 app env 只放自己的业务 API 代理：

```env
VITE_API_PROXY=http://localhost:8000
```

你不一定非要用这些端口，但必须保证：

- `VITE_API_PROXY` 指向当前 app 可用的 API 服务
- `VITE_AUTH_PROXY` 指向可用的认证服务
- `VITE_FILE_PROXY` 指向可用的文件服务

如果这些值缺失或服务没启动，页面可能能打开，但登录、权限、文件和业务接口通常跑不通。

## 后端服务边界

这个前端框架假设本地服务端已经提供：

- auth：登录、注册、会话恢复、refresh token、个人资料
- file：上传签名、访问签名、删除签名
- web app API：普通用户侧业务入口和权限接口
- admin API：后台账号、角色、权限、菜单和用户管理接口

admin 的权限管理是标准基础能力，不是演示代码。普通用户很难自己设计好权限模型，所以框架默认保留这套后台基础功能。

## 你可以怎样让 AI 检查环境

```text
先阅读 AGENT_PROTOCOL/PROTOCOL.md。

请帮我检查本地 dev 联调环境：
1. Node 和 pnpm 是否可用
2. 依赖是否安装完成
3. 当前还缺哪些环境变量
4. dev proxy 还缺哪些目标地址
5. 本地后端服务是否需要先启动
6. 执行 pnpm dev 后还差什么
```

## 相关文档

- [用户文档首页](./README.md)
- [本地 Dev 联调清单](./dev-checklist.md)
- [Web 应用说明](./web-app-guide.md)
- [任务模板](./task-template.md)
- [Codex 协议入口](../AGENT_PROTOCOL/PROTOCOL.md)
