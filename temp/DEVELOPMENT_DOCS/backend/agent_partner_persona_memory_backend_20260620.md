# AI 电商搭档人格与记忆方向后端开发计划

## 需求来源

本需求来自当前聊天确认：客户反馈现有产品“界面全是问题，体感不太好”，且希望 AI 角色更深刻、更人性化，让用户愿意持续聊，并从中得到情绪和专业价值。

本阶段采用用户确认后的第一期方案：先做稳定人格和回答风格优化，不做普通用户驱动的 skill 自进化，不新增数据库和 API。

## 业务理解

当前问题不是角色数量不足，而是用户感受到：

1. AI 像工具、老师或岗位报告，不像平等伙伴。
2. AI 容易先追问大量信息，用户有被审问的压力。
3. 多岗位能力存在，但前台表达割裂，缺少统一人格。
4. 用户需要“先被接住，再被推进”，不是先被要求补全资料。

后端侧目标是让现有固定岗位 Agent 服从统一的“AI 电商搭档”人格底色：平等、专业、不说教、少审问、先给价值，再问关键补充。

## 本阶段范围

### 做

- 调整中文 prompt 和最终合成规则。
- 调整用户可见输出清洗策略，减少内部 runtime/岗位报告感。
- 约束回答：先判断/方向，再最多追问 1-2 个关键问题。
- 保持固定岗位 Agent 和 skill 的稳定性。

### 不做

- 不新增用户记忆数据库表。
- 不新增或修改 HTTP API。
- 不修改 SSE 协议。
- 不让普通用户对话自动修改 prompt、skill、角色目录或 task graph 内核。
- 不接入多平台网关、MCP 或模型市场。

## 数据库设计

本阶段无数据库变更。

长期记忆属于第二阶段，需要另行设计用户级记忆表、记忆类型枚举、授权/删除/关闭机制和召回规则。

## API 设计

本阶段无新增 API，无修改现有 API。

因此本阶段不更新 `API_DOCS/`。

## 鉴权设计

本阶段不新增接口，沿用现有 Agent session 鉴权和上下文。

## LLM 与 Agent 配置设计

模型调用仍沿用现有 `backend/config/services.toml` 的 `[agent_llm_usage]`：

- `fast`：轻量对话、短答和快速路由。
- `thinking`：复杂岗位分析和最终合成。
- `default`：未命中特定用途时 fallback。

不新增 `llm.name`，不在 web-server handler 中创建 LLM client。

## 预计修改文件

- `backend/config/prompts/zh/roles/chat.md`
- `backend/config/prompts/zh/roles/ops.md`
- `backend/config/prompts/zh/roles/data.md`
- `backend/config/prompts/zh/roles/creative.md`
- `backend/config/prompts/zh/roles/service.md`
- `backend/config/prompts/zh/roles/accounting.md`
- `backend/config/prompts/zh/cognition/chat_worker.md`
- `backend/config/prompts/zh/cognition/final_synthesis.md`
- `backend/config/json/agent/user-facing-normalization-policy.json`
- 必要时小范围调整 `worker-output-policy.json`

## 回答风格规则

1. 平视用户：少用“你必须”“正确做法是”，多用“我们先看”“可以先这样试”。
2. 先接住：信息不完整时，先基于已有信息给初步方向。
3. 少追问：第一轮最多问 1-2 个关键问题。
4. 专业但不端着：指出问题时使用可推进表达，不做批改式否定。
5. 不空泛安慰：情绪价值来自帮用户拆小下一步。
6. 不暴露内部词：用户可见回答不出现 worker、graph、role_output、task graph 等术语。

## 验收标准

- 用户问“这个商品能不能卖”时，AI 先给判断框架或初步方向，而不是先列大量必填字段。
- 用户问“帮我改标题”时，AI 先指出标题问题或给改法，不先审问平台、竞品、规则。
- 用户说“老师说我方案太空”时，AI 不用批改语气，而是帮助补证据、补动作、补表达。
- 最终回答不出现内部岗位报告、runtime ID、worker/graph/role_output 等内部术语。
- 不影响现有固定岗位 Agent 和 skill 的版本稳定性。

## 验证计划

- JSON 配置变更后运行 JSON 解析检查。
- 后端配置/prompt 变更后优先运行 `cargo fmt`、`cargo check`、必要时 `cargo clippy`。
- 结合前端改造后进行真实问题手动 smoke。

