# 如何使用 AI

这个仓库的 `README.md` 是给使用者看的，不是给 AI 看的。

如果你要让 AI 帮你分析、回答问题或开发代码，请先明确告诉 AI：

- 先阅读 [AGENT_PROTOCOL/PROTOCOL.md](./AGENT_PROTOCOL/PROTOCOL.md)
- 必须按这个仓库现有的分层和规则工作
- 不允许脱离仓库结构随意发挥

如果你想先看给使用者准备的文档，请从这里开始：

- [user_docs/README.md](./user_docs/README.md)

## 获取模板

不要直接使用 `git clone`。

更适合普通使用者的方式，是用 `npx` 下载一份模板副本：

```bash
npx degit atlas-orien/react-mono-template your-project-name
cd your-project-name
pnpm install
```

这里的 `your-project-name` 只是示例占位名。
请把它替换成你自己想要的项目目录名，例如 `mall-admin`、`my-company-web`、`demo-app`。

这样做的好处是：

- 不会继承这个模板仓库的 `.git` 历史
- 不会默认保留模板仓库的 remote 地址
- 你可以把它当成自己的新项目重新创建 git 仓库

如果你的机器暂时不能使用 `npx`，再退回到 `git clone`：

```bash
git clone https://github.com/atlas-orien/react-mono-template.git your-project-name
cd your-project-name
rm -rf .git
git init
pnpm install
```

这样也可以避免继续连接到模板仓库的 remote。

如果你需要，可以在新目录里自己执行：

```bash
git add .
git commit -m "init from template"
```

如果你不会执行这些命令，也可以直接把仓库地址和你的目标目录告诉 AI，让 AI 帮你完成：

```text
请先帮我把这个模板下载到本地并完成环境准备。

模板地址：
https://github.com/atlas-orien/react-mono-template.git

目标目录：
请改成你自己的目录名，例如 mall-admin

要求：
1. 优先使用 npx 方式获取模板
2. 如果当前环境不能使用 npx，再改用 git clone
3. 不要保留模板仓库的 git remote
4. 完成后继续帮我安装依赖，并告诉我本地 dev 怎么启动
```

## 推荐使用方式

每次给 AI 发任务前，先带上这段前置说明：

```text
先阅读仓库根目录 AGENT_PROTOCOL/PROTOCOL.md，再开始回答和修改代码。
你的判断、建议、实现和目录落点，都必须遵守其中映射到的协议文件。
如果仓库已有明确规则，按现有规则执行，不要另起一套方案。
```

如果当前仓库还没有安装好环境，AI 的第一件事不是改代码，而是先完成环境准备。

## 你给 AI 的需求最好包含

- 目标：你要做什么
- 位置：影响哪个 app、页面、组件或包
- 结果：你希望最终界面或行为变成什么样
- 验收：你希望怎么检查结果

如果你已经理解这个框架，也可以补充：

- 偏好：你更希望优先改页面、共享组件、主题，还是服务层
- 限制：哪些部分你暂时不想动

## 怎么理解“位置”

这个项目主要分成两个大分支：

- `apps/*`
  放最终应用，也就是你实际会打开、使用、看到页面的地方。

- `packages/*`
  放被多个应用复用的共享能力，也就是公共底层模块。

如果你不知道该怎么描述位置，可以先按下面理解：

### `apps/*` 是“应用层”

适合描述这类需求：

- 我要改某个页面
- 我要加一个路由
- 我要调整某个业务流程
- 我要改登录页、首页、个人中心页

当前你最常接触的是：

- `apps/web`
  主业务应用。页面、路由、布局、登录、业务交互，一般都在这里。

- `showcases/guide`
  组件和框架能力展示应用。不是业务页面，主要用于验证共享组件效果和给 AI 提供合法用法参考。

### `packages/*` 是“共享层”

适合描述这类需求：

- 我要新增一个多个页面都能复用的组件
- 我要调整全局主题颜色或主题模式
- 我要修改通用 API 请求能力
- 我要改公共服务能力，而不是某一个页面里的临时代码

当前主要包括：

- `packages/ui-components`
  给业务页面直接使用的基础共享 UI 组件。
  应优先按分类路径导入，例如 `@workspace/ui-components/stable/button`。
  通知能力也应通过产品语义入口使用，例如 `@workspace/ui-components/stable/toast`。

- `packages/app-kit`
  给业务 app 直接使用的共享应用能力。
  这里放共享页面、应用壳层、页面装配、布局语义、业务组合类组件，不要求固定通用格式。

- `packages/ui-core`
  更底层的 primitive 基础组件能力，一般不是业务页面直接改的第一选择。

- `packages/ui-theme`
  全局主题变量、light/dark 主题能力。

- `packages/services`
  通用服务层，例如 API、query、错误模型。
  当前请求路径应在服务代码中显式完整定义，环境变量主要用于 dev proxy。

- `packages/locales`
  统一语言资源层。
  放共享翻译资源、i18n 装配、语言归一化和组件 copy。
  默认不再把共享语言文件散落到 `apps/*/public/locales`。

## 环境文件约定

当前仓库环境文件按“共享层 + app 层”拆分：

- 模板源：
  - `env.shared.example`
- 根目录：
  - `.env.development`
  - `.env.production`
- 每个 app 自己维护：
  - `apps/<app>/.env.development`
  - `apps/<app>/.env.production`

其中：

- 根目录只保留共享代理，例如 `VITE_AUTH_PROXY`、`VITE_FILE_PROXY`
- 每个 app 自己提供 `VITE_API_PROXY`
- `env.shared.example` 同时写 `production` 和 `development` 两套示例值
- `pnpm env:init` 会从 `env.shared.example` 生成根目录和两个 app 的本地 env 文件
- `pnpm dev` 会同时读取根目录共享 env 和当前 app 的 `.env.development`
- 不再使用 `VITE_*_URL` 做请求前缀拼接

## 用户不会判断位置也没关系

如果你不知道需求应该落在 `apps` 还是 `packages`，可以直接这样告诉 AI：

```text
先阅读 AGENT_PROTOCOL/PROTOCOL.md。

我不确定这个需求应该改 apps 还是 packages。
请你先根据当前项目分层判断正确落点，再开始回答或修改。
```

## 先看框架思想

如果你还不熟悉这个仓库为什么这样分层，先看这里：

- [user_docs/framework-thinking.md](./user_docs/framework-thinking.md)

这个文件不是教你写代码，而是帮助你理解：

- 为什么项目要分成 `apps` 和 `packages`
- 什么需求更适合改页面，什么需求更适合沉淀为共享能力
- 为什么 AI 不能在业务页面里随便复制一份公共实现
- 为什么这个框架要求先判断落点，再开始修改

## 常见场景

### 1. 先让 AI 回答问题

```text
先阅读 AGENT_PROTOCOL/PROTOCOL.md。

我先不让你改代码。
请先基于这个仓库当前结构，回答这个问题：
……
```

### 2. 先让 AI 安装和准备环境

```text
先阅读 AGENT_PROTOCOL/PROTOCOL.md。

先不要改代码，先完成这个仓库的环境准备：
1. 检查 Node 和 pnpm
2. 安装依赖
3. 告诉我本地 dev 怎么启动
4. 告诉我需要哪些环境变量
5. 汇报当前环境是否可用
```

### 3. 让 AI 开发具体需求

```text
先阅读 AGENT_PROTOCOL/PROTOCOL.md。

我要改 apps/web 的登录页：
1. 保持现有路由结构不变
2. UI 风格沿用当前设计系统
3. 不要新造一套共享组件
4. 完成前执行对应协议要求的检查
```

### 4. 使用本地服务端联调

```text
先阅读 AGENT_PROTOCOL/PROTOCOL.md。

这次使用本地服务端联调，不使用 mock。

请先告诉我：
1. 应该运行什么命令
2. 需要哪些本地后端服务
3. 当前 `.env.development` 还缺哪些 proxy 地址
```

## 说明

- 根目录 `README.md` 只负责告诉你怎么使用 AI。
- 写给使用者看的文档统一放在 `user_docs/` 目录。
- AI 必须阅读的工程规则统一放在 `AGENT_PROTOCOL/PROTOCOL.md`，细分协议统一放在 `AGENT_PROTOCOL/protocols/`。
- 如果后续新增局部协议，应补充到 `AGENT_PROTOCOL/protocols/`，并在 `AGENT_PROTOCOL/PROTOCOL.md` 中登记入口。
