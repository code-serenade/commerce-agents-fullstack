# 本地 Dev 联调清单

这个文档是给使用者看的。

它只关心一件事：在本地把 `pnpm dev` 跑起来并对接本地服务端时，最少要确认什么。

这里说的都是本地开发联调，不涉及正式环境、线上部署或 product 配置。

## 适用场景

适合：

- 你希望前端接本地或测试后端进行联调
- 你想确认真实接口在本地开发环境里是否可用
- 你不打算了解正式环境，只想把当前仓库本地跑通

不适合：

- 线上部署
- 正式环境发布
- product 环境排查

这些内容不在这份文档里。

## 开始前先确认

在让 AI 帮你处理之前，至少先确认这几件事：

1. 本机已经安装 Node.js，版本满足 `>=20`
2. 本机已经安装 `pnpm`
3. 你已经在仓库根目录执行过 `pnpm install`
4. 你现在要跑的是本地 `dev`
5. 你手上已经有可联调的后端地址，或者 AI 可以帮你检查还缺哪些地址

## 最少需要确认的环境变量

本地 `dev` 联调至少要分清两类值：

1. 根目录共享代理要转发到哪里
2. 当前 app 自己的 API 代理要转发到哪里

常见最小示例：

```env
VITE_AUTH_PROXY=http://localhost:9000
VITE_FILE_PROXY=http://localhost:9000
```

这些共享值通常放在根目录 `.env.development`。

当前 app 还需要自己的：

```env
VITE_API_PROXY=http://localhost:8000
```

你不一定非要用这些端口，但必须保证：

- `VITE_API_PROXY` 指向当前 app 可用的 API 服务
- `VITE_AUTH_PROXY` 指向可用的认证服务
- `VITE_FILE_PROXY` 指向可用的文件服务

如果这些值缺失或服务没启动，`pnpm dev` 虽然可能能打开页面，但真实联调通常是跑不通的。

可以先检查根目录 `env.shared.example`，再执行：

```bash
pnpm env:init
```

这个命令会生成根目录共享 env，以及 `apps/admin`、`apps/web` 各自只包含 `VITE_API_PROXY` 的 app env。

## 本地 Dev 联调最少检查项

当你要让 AI 帮你准备本地联调环境时，可以要求它至少检查：

1. Node 和 `pnpm` 是否可用
2. 依赖是否已安装
3. 当前仓库缺少哪些环境变量
4. 当前 `dev proxy` 应该指向哪些本地服务
5. `pnpm dev` 是否能成功启动
6. 页面打开后，关键接口是否真的发到了你预期的后端

## 如果你不知道自己缺什么

可以直接把下面这段发给 AI：

```text
先阅读 AGENT_PROTOCOL/PROTOCOL.md。

我只想把本地 dev 联调跑通，不需要你解释正式环境。

请先帮我检查：
1. Node 和 pnpm 是否可用
2. 依赖是否安装完成
3. 当前还缺哪些环境变量
4. dev proxy 还缺哪些目标地址
5. 执行 pnpm dev 后，本地联调还差什么
```

## 什么时候不该继续折腾前端

如果你的目标只是：

- 改页面
- 改布局
- 改样式
- 改表单交互
- 跑通基础前端流程

但本地服务端还没有启动，应该先暂停前端扩展，优先把后端服务和 `.env.development` proxy 补齐。

## 相关文档

- [用户文档首页](./README.md)
- [本地环境说明](./env-setup.md)
- [Web 应用说明](./web-app-guide.md)
- [任务模板](./task-template.md)
