# AI 使用指南

这个模板推荐的使用方式，不是让 AI 一上来就直接写代码。

正确流程是：

**用户需求 -> 服务端实现文档 -> 用户确认 -> AI 按确认结果开发**

这样做的原因很简单：

- 用户通常更容易判断“业务设计对不对”
- 但很难直接判断“代码写得对不对”
- 如果 AI 没先把表结构、接口、权限边界设计清楚，就很容易返工

所以本项目要求 AI 先做设计，再做实现。

---

## 标准工作流程

### 第一步：用户先描述业务

用户不需要直接描述数据库字段、DTO、repo、handler。

用户只需要描述业务，例如：

- 这是个什么系统
- 要管理哪些对象
- 用户能做什么操作
- 哪些数据只能看自己的
- 哪些数据需要管理员才能操作

例如：

```text
我要做一个设备管理系统。

需求：
1. 用户可以登记设备
2. 用户可以查看自己登记的设备
3. 管理员可以查看所有设备
4. 每个设备要有编号、名称、类型、状态、创建时间
5. 接口需要登录后访问
```

---

## 第二步：AI 不要立刻写代码，先输出“服务端实现文档”

这一步最重要。

AI 在真正开发前，必须先阅读：

- `AI_PROTOCOLS/TABLE_ADDING_PROTOCOL.md`
- 如果涉及登录用户：`AI_PROTOCOLS/AUTH_INTEGRATION_GUIDE.md`
- 如果涉及 AI/LLM：`AI_PROTOCOLS/LLM_CLIENT_GUIDE.md`

然后先输出一份“服务端实现文档”，供用户确认。

这份文档至少应包含：

1. 业务理解
2. 数据表设计
3. 字段说明
4. 表之间的关系
5. 哪些字段与当前登录用户 `user_id` 相关
6. 需要提供哪些 HTTP 接口
7. 每个接口的作用
8. 哪些接口需要登录
9. 哪些接口只能查询当前用户自己的数据
10. 开发顺序
11. 如果涉及 AI/LLM：模型配置名称、输入输出格式、失败处理和验证方式

这一阶段 **禁止直接开始写代码**。

---

## 第三步：用户确认服务端实现文档

AI 输出文档后，用户需要先确认：

- 表结构是否合理
- 字段是否缺失
- 命名是否符合业务习惯
- 接口设计是否符合预期
- 数据权限是否正确

如果用户没有确认，AI 不应该直接进入开发阶段。

---

## 第四步：AI 按确认过的文档开发

当用户确认后，AI 才开始正式开发。

开发时必须严格按照：

- 已确认的服务端实现文档
- `AI_PROTOCOLS/TABLE_ADDING_PROTOCOL.md`
- 如果涉及登录用户：`AI_PROTOCOLS/AUTH_INTEGRATION_GUIDE.md`
- 如果涉及 AI/LLM：`AI_PROTOCOLS/LLM_CLIENT_GUIDE.md`

推荐顺序：

1. migration
2. 用户确认表结构无误
3. 生成 entity
4. repo
5. service
6. web-server

---

## 推荐给 AI 的第一轮提示词

当用户刚描述完需求时，推荐先这样要求 AI：

```text
请先不要直接写代码。

先阅读 AI_PROTOCOLS/TABLE_ADDING_PROTOCOL.md。
如果需求涉及登录用户，请继续阅读 AI_PROTOCOLS/AUTH_INTEGRATION_GUIDE.md。
如果需求涉及 AI/LLM、图片识别或模型调用，请继续阅读 AI_PROTOCOLS/LLM_CLIENT_GUIDE.md。

然后根据我的业务需求，先输出一份“服务端实现文档”，内容至少包括：
1. 业务理解
2. 数据表设计
3. 每张表的字段、类型、约束
4. 表之间的关系
5. 哪些字段需要关联当前登录用户 user_id
6. 需要提供的 HTTP 接口
7. 每个接口的权限要求
8. 如果涉及 AI/LLM：使用哪个 llm.name、输入输出格式、失败处理和验证方式
9. 开发顺序

不要开始写代码，先等我确认。
```

---

## 推荐给 AI 的第二轮提示词

当用户确认表结构和接口设计以后，再这样要求 AI：

```text
现在开始开发。

请严格按照已经确认的服务端实现文档执行。
同时遵守 AI_PROTOCOLS/TABLE_ADDING_PROTOCOL.md。
如果涉及登录用户，同时遵守 AI_PROTOCOLS/AUTH_INTEGRATION_GUIDE.md。
如果涉及 AI/LLM，同时遵守 AI_PROTOCOLS/LLM_CLIENT_GUIDE.md。

按以下顺序完成：
1. migration
2. 提示我生成 entity
3. repo
4. service
5. web-server

不要擅自改动已经确认的表结构；如果发现设计问题，先停止并说明原因。
```

---

## 最推荐的协作方式

你可以把整个过程理解成两次对话：

### 第一次对话：只做设计

目标：

- 让 AI 把用户需求翻译成服务端实现文档

输出：

- 表结构
- 字段
- 接口
- 权限

### 第二次对话：只做开发

目标：

- 按确认后的文档生成代码

输出：

- migration
- repo
- service
- web-server

这种方式比“直接让 AI 改代码”稳定很多。

---

## 什么时候必须引入认证协议

只要需求里出现下面这些意思，就必须让 AI 同时阅读 `AUTH_INTEGRATION_GUIDE.md`：

- 登录后访问
- 当前用户
- 我的数据
- 只能看自己创建的数据
- 按用户隔离
- 创建人 / owner / created_by

## 什么时候必须引入 LLM 协议

只要需求里出现下面这些意思，就必须让 AI 同时阅读 `LLM_CLIENT_GUIDE.md`：

- AI 生成
- 大模型 / LLM
- 图片识别
- OCR 或视觉理解
- OpenAI-compatible provider
- Ollama
- 多模型切换

当前项目的 LLM client 统一配置在 `config/services.toml` 的 `[[llm]]`，并通过 `crates/web-server/src/statics/llm_client.rs` 获取。

---

## 与 AI 协作时的注意事项

1. 不要一开始就让 AI 直接写完整功能。
2. 先要设计文档，再做开发。
3. 表结构一定要先确认。
4. 如果 AI 想跳过确认阶段直接编码，要打断它。
5. 每次都强调 `禁止修改 db_core`、`禁止在 repo 外使用 sea-orm`。

---

## 建议阅读顺序

1. `user_docs/core_design.md`
2. `user_docs/requirement_to_backend_doc.md`
3. `AI_PROTOCOLS/TABLE_ADDING_PROTOCOL.md`
4. `AI_PROTOCOLS/AUTH_INTEGRATION_GUIDE.md`
5. `AI_PROTOCOLS/LLM_CLIENT_GUIDE.md`
