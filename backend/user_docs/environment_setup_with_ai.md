# 环境问题处理指南

这个文档不是教用户手工处理环境，而是告诉用户：

**环境有问题时，优先让 AI 自己排查和修复。**

---

## 一、环境阶段应该让 AI 做什么

在真正开发业务之前，先让 AI 处理这些事情：

1. 检查 Rust 是否可用
2. 检查 `cargo-generate` 是否可用
3. 检查 Docker 是否可用
4. 处理 Docker 镜像拉取问题
5. 检查 PostgreSQL 是否可用
6. 检查 `auth` 服务是否可访问
7. 检查配置文件是否正确
8. 如果配置了 `[[llm]]`，检查模型服务是否可访问
9. 启动当前服务并测试接口文档中的关键接口

---

## 二、如果 Docker 拉取失败

中国网络环境下，Docker Hub 经常不可用。

这类问题默认应由 AI 自己处理，包括：

- 检查 Docker 是否正常
- 检查镜像拉取是否失败
- 配置可用镜像源
- 改用可访问的镜像仓库
- 重新拉取 PostgreSQL 镜像

用户不需要自己研究镜像源。

你可以直接对 AI 说：

```text
如果 Docker 拉取镜像失败，请你自己处理镜像源或可访问仓库，不要把这个问题直接丢给我。
```

---

## 三、如果本地数据库起不来

如果 AI 没法把本地 PostgreSQL 跑起来，不要卡在这里。

可以找技术同事要一个已经可用的 PostgreSQL 连接字符串，例如：

```text
postgres://username:password@host:5432/database_name
```

然后把这个连接发给 AI，让它自己完成：

1. 测试连接是否可用
2. 修改项目配置文件
3. 重新执行初始化、迁移和启动测试
4. 如果失败，继续自己排查和修复

你可以直接发给 AI：

```text
如果本地数据库无法初始化，我现在给你一个可用的 PostgreSQL 连接。

请你自己完成：
1. 先测试连接是否可用
2. 如果可用，自动修改配置文件
3. 重新执行初始化、迁移和启动测试
4. 如果仍有错误，请继续自己排查和修复
5. 最后告诉我项目是否已经可以正常运行

数据库连接如下：
<把数据库连接放这里>
```

---

## 四、如果出现其它错误

环境阶段的原则是：

**优先让 AI 自己处理，用户只在 AI 明确需要外部信息时再介入。**

例如只有这些情况才需要用户协助：

- 需要管理员权限
- 需要公司内部网络地址
- 需要技术同事提供数据库连接
- 需要提供 `auth` 服务地址或 token
- 需要提供远程 LLM 服务地址、模型名或 API key

除此之外，优先让 AI 自己解决。

---

## 五、推荐给 AI 的环境提示词

```text
请先不要开发业务功能。

先帮我完成环境检查和启动测试：
1. 检查 Rust、cargo-generate、Docker 是否可用，如缺少请协助安装
2. 如果 Docker 拉取镜像失败，请自行处理镜像源或可访问仓库
3. 检查 config/services.toml 是否合理
4. 检查 PostgreSQL 是否可用，如没有则协助启动
5. 如果本地数据库不可用，请告诉我是否需要提供远程数据库连接
6. 检查 auth 服务是否可访问
7. 运行数据库初始化和迁移
8. 启动当前服务
9. 如果配置了 `[[llm]]`，检查模型服务是否可访问；如果是本地 Ollama，可运行 `cargo run -p web-server --example llm_vision_smoke`
10. 核对 `../API_DOCS/` 下的 Markdown 接口文档是否与当前代码一致
11. 告诉我当前环境是否已经可以开始开发

如果中间发现问题，请先自行排查和修复，不要直接开始写业务代码。
```

---

## 六、如果 LLM 或 Ollama 不可用

当前项目支持通过 `config/services.toml` 配置多个 LLM client：

```toml
[[llm]]
name = "ollama-gemma4"
base_url = "http://127.0.0.1:11434"
model = "gemma4:26b"
max_tokens = 20000
temperature = 0.2
```

如果本地 Ollama 不可用，AI 应先自己检查：

1. `curl http://127.0.0.1:11434/api/tags` 是否可访问
2. 目标模型是否存在，例如 `gemma4:26b`
3. `config/services.toml` 中 `base_url` 和 `model` 是否正确
4. `cargo run -p web-server --example llm_vision_smoke` 是否可以识别测试图片

如果用户改用远程模型服务，AI 应让用户提供：

- OpenAI-compatible base URL
- model 名称
- API key，如果 provider 需要

API key 只能写入配置文件或环境化配置，不允许写死到代码里。
