# AI 工作流总入口

> 这是给 AI 的总流程文档。
> 如果 AI 只看一个文件，优先看这个文件。

---

## 一、总原则

AI 在这个项目里必须遵守下面的顺序：

1. 先处理环境问题
2. 再理解业务需求
3. 先输出“服务端开发文档”
4. 等用户确认
5. 再正式开发
6. 开发后再验证和修复

如果没有完成第 3 步和第 4 步，AI 不得直接开始写代码。

---

## 二、标准执行顺序

### Step 0：先处理环境

先检查并处理：

- Rust
- cargo-generate
- Docker
- Docker 镜像源
- PostgreSQL
- `auth` 服务可达性
- 配置文件是否可用
- 如果需求涉及 AI/LLM，检查 `[[llm]]` 配置和模型服务是否可用

如果出错，优先自己排查和修复，不要立即把问题丢给用户。

环境相关规则：

- 中国网络环境下 Docker Hub 可能不可用
- AI 应优先自行处理镜像源问题
- 如果本地数据库不可用，可以改用用户提供的远程数据库连接

### Step 1：理解业务需求

用户给出的内容通常是业务描述，不是技术设计。

AI 需要先提炼出：

- 核心对象
- 用户动作
- 权限边界
- 是否需要登录
- 数据是否按用户隔离
- 是否需要 AI/LLM 调用、图片识别或多模型配置

### Step 2：输出“服务端开发文档”

在编码前，AI 必须先输出一份“服务端开发文档”。

至少包含：

1. 业务理解
2. 数据表设计
3. 字段、类型、约束
4. 表关系
5. 权限设计
6. HTTP 接口设计
7. 与当前登录用户 `user_id` 的关系
8. 如果涉及 AI/LLM：模型配置名称、输入输出格式、失败处理和验证方式
9. 开发顺序

### Step 3：等待用户确认

用户没有明确确认之前，不允许直接开始：

- migration
- repo
- service
- web-server

### Step 4：开始开发

确认后按顺序开发：

1. migration
2. 生成 entity
3. repo
4. service
5. web-server

其中 `service` 阶段增加强制规则：

- 只要一个业务 API 里有多次写操作且要求一致性，必须加事务。
- 事务只能在 `service` 层开启，不能在 `repo` 或 `web-server` 开启。
- AI 对写接口默认按“需要事务”处理，除非用户明确允许部分成功。

### Step 5：验证与修复

开发完成后，AI 应继续完成：

- `cargo check`
- 启动服务
- 按 `../API_DOCS/` 验证关键接口
- 进行必要的自测
- 若失败，优先自行修复

---

## 三、认证相关规则

只要需求涉及：

- 当前用户
- 登录后访问
- 我的数据
- 用户自己的资源

则必须同时阅读：

- `AUTH_INTEGRATION_GUIDE.md`

必须遵守：

1. 认证统一来自外部 `auth`
2. 当前登录用户 `user_id` 一律按 UUID 字符串处理
3. 不得改成整数
4. 不得重复实现登录体系

## 四、LLM 相关规则

只要需求涉及：

- AI 生成
- LLM 调用
- 图片识别
- 多模型切换
- OpenAI-compatible、Ollama 或其它模型服务

则必须同时阅读：

- `LLM_CLIENT_GUIDE.md`

必须遵守：

1. LLM client 统一由 `agent` crate 初始化和持有，web-server 通过 `statics::agent_manager` 运行 agent session。
2. 不允许在 handler 中临时创建 LLM client。
3. 模型、base URL、API key 只能来自 `config/services.toml`。
4. 服务端开发文档必须写清使用哪个 `llm.name`。
5. 开发完成后优先运行 `cargo check --workspace --examples`，涉及 agent/LLM 时运行 `cargo run -p agent --bin ollama_task_graph_smoke`。
6. Agent session 流式输出使用 SSE，普通业务通知继续使用 WebSocket。

---

## 五、数据库相关规则

### 1. migration 不等于 entity

- migration 用来改数据库结构
- entity 用来根据已有表结构生成代码

### 2. 生成 entity 的默认方式

使用：

```bash
./scripts/generate_entity.sh
```

或：

```bash
make entity-generate
```

### 3. `fresh_db.sh` 不是默认动作

这是破坏性操作。

只有在明确允许重建数据库时才能使用。

---

## 六、必须阅读的文档顺序

推荐顺序：

1. `AI_WORKFLOW.md`
2. `TABLE_ADDING_PROTOCOL.md`
3. 如果涉及登录用户：`AUTH_INTEGRATION_GUIDE.md`
4. 如果涉及 AI/LLM：`LLM_CLIENT_GUIDE.md`
5. 分层协议：
   - `MIGRATION_GUIDE.md`
   - `REPO_GUIDE.md`
   - `SERVICE_GUIDE.md`
   - `WEB_SERVER_GUIDE.md`

---

## 七、一句话要求

**先解决环境，先做设计，先让用户确认，再开始开发。**
