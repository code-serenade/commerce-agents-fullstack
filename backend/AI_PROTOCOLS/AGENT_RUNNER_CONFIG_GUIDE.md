# Agent Runner 与 Task Graph 配置化协议

本文档约束 `agent` crate 里的 commander、task graph 和 worker runner。agent 的业务编排和 worker 业务能力必须优先通过配置完成，Rust 代码只允许保留通用编排引擎和通用执行引擎。

## 核心原则

1. worker runner 是一次任务执行过程中的短生命周期组件，由外部 agent manager 和 commander 监管；任务完成后释放。
2. commander 负责任务入口、task graph 规划、节点调度、重试、收集 worker report 和完成 graph；worker 只执行当前分配给自己的单个节点。
3. task graph 是跨角色业务编排的唯一配置入口；worker 不决定多角色协作顺序，也不决定最终汇总节点。
4. runner 不负责维护长期业务状态，不重新设计 memory、cognition、skill 或 manager。
5. 角色、任务图、autosuggest、inference、输出策略和 prompt 都必须优先放在 `backend/config/` 下。
6. 普通业务调整只允许修改配置文件，不允许修改 commander/task_graph/worker runner Rust 代码。
7. Rust 侧只保留“读取配置、匹配配置、构建图、调度节点、执行 skill、组装上下文、校验输出、降级处理”等通用能力。
8. task graph 节点和动态角色配置可以配置 `llm_usage`，取值只使用 `fast`、`thinking` 或 `default`，用于路由到 `services.toml` 的 `[agent_llm_usage]`。轻量短答、上游草案或显式岗位协作的动态角色可以用 `fast`；最终综合、财务测算或复杂证据综合的节点保留默认 `thinking`。不要在 Rust 里按行业或业务关键词硬编码模型选择。
9. task graph 节点和动态角色配置可以配置 `skill_autosuggest`。显式多岗位轻量协作、最终润色或不需要外部适配器的节点可以设为 `false`，避免自动技能请求拖慢和污染输出；需要后台、搜索、投放、财务等真实工具能力的深度图保持默认开启。
10. task graph 的 `match.min_explicit_role_aliases` 是通用匹配条件，用于要求用户文本至少命中若干个 `role-aliases.json` 中的岗位别名。它只表达“是否真的点名了岗位”，不在 Rust 里写业务岗位词。

## 配置入口

业务配置入口统一在：

- `backend/config/json/agent/roles.json`
- `backend/config/json/agent/task-graph/index.json`
- `backend/config/json/agent/worker-autosuggest/`
- `backend/config/json/agent/worker-inference/`
- `backend/config/json/agent/worker-output-policy.json`
- `backend/config/json/agent/worker-output-normalization-policy.json`
- `backend/config/prompts/`

旧的单文件策略：

- `backend/config/json/agent/worker-autosuggest-policy.json`
- `backend/config/json/agent/worker-inference-policy.json`

只作为兼容 fallback，不作为新增业务规则的首选位置。

## 允许修改 Rust 的情况

只有下面情况可以修改 commander、task_graph 或 runner Rust 代码：

1. 新增一种通用配置字段，并同步更新配置说明。
2. 新增一种通用模板占位符求值器，例如把配置里的 `{{field}}` 解析为结构化 skill input。
3. 修复通用执行 bug、错误处理、日志或性能问题。
4. 接入新的通用 skill executor 边界。
5. 删除已经被配置完全覆盖的旧代码。

即使需要修改 Rust，也必须保持代码与业务解耦：Rust 里不能写具体行业方案、角色话术、业务关键词列表、平台规则文案、跨角色业务流程或 skill 路由规则。

## 禁止事项

1. 不允许在 commander/task_graph/runner Rust 文件里新增大段业务 if/else、match 或关键词数组。
2. 不允许把 role prompt、回答模板、教学深度、角色边界、输出格式写死进 Rust。
3. 不允许把 autosuggest skill 路由写死进 Rust。
4. 不允许把 inference 的别名、默认值、分类标签、搜索关键词写死进 Rust。
5. 不允许在 runner 里重新实现 cognition；cognition 已有独立目录，prompt 从 `backend/config/prompts/` 读取。
6. 不允许在 runner 里重新实现 memory；长期记忆和 session 记忆应走既有 agent session/memory 边界。
7. 不允许把 `old_runner` 里的旧业务代码复制回新 runner。
8. 不允许把 task graph 的业务编排写进 `commander/task_graph/*.rs` 或 `commander/runner/*.rs`。
9. 不允许让 worker 根据业务意图自行选择其它 worker、插入新 graph 节点或决定最终汇总策略。

## Commander 与 Task Graph 边界

commander 是 graph 编排运行时，只负责通用调度：

- 根据配置选择或构建 task graph。
- 把 ready node 转换成 `WorkerAssignment` 发给对应 worker。
- 接收 `WorkerReport`，更新 graph runtime。
- 根据 graph 配置决定重试、rework、完成和最终输出。
- 发出 session event，记录 graph 进度。

task graph Rust 只允许做通用配置加载、校验、模板渲染、节点构建和依赖计算。新增业务编排时，必须修改：

- `backend/config/json/agent/task-graph/index.json`
- `backend/config/json/agent/task-graph/defaults.json`
- `backend/config/json/agent/task-graph/role-aliases.json`
- `backend/config/json/agent/task-graph/role-defaults.json`
- `backend/config/json/agent/task-graph/graphs/*.json`

新增一套业务流程时，优先在 `graphs/` 下新增单独 JSON 文件，并在 `index.json` 里登记。不要恢复旧的 `task-graph-rules.json`，不要在 Rust 里维护第二份 graph。

## Worker Runner 边界

worker runner 是单节点执行器，只负责当前 `WorkerAssignment`：

- 接收 assignment 或单 worker task。
- 构建 cognition input。
- 执行显式或 autosuggest 生成的 skill request。
- 解析 worker 输出并生成 `WorkerReport`。
- 把 `graph_id`、`node_id`、上游 inputs 和 assignment metadata 透传给 cognition/skill context。

worker 允许读取 assignment 的 graph 信息，但只能作为上下文使用。worker 不允许：

- 选择下一节点。
- 调度其它 worker。
- 改写 graph 依赖关系。
- 判断整张 graph 是否完成。
- 生成跨 worker 的最终汇总结构。

如果发现 worker 里出现这些职责，必须迁移到 task graph 配置或 commander 的通用 graph runtime。

## Autosuggest 规则

autosuggest 的业务路由在 `backend/config/json/agent/worker-autosuggest/`。

Rust 只负责：

- 读取 `index.json`。
- 合并分片配置。
- 根据 role、关键词和依赖 skill 匹配规则。
- 处理 `fallback` 规则。
- 渲染 `input` 里的模板占位符。

新增业务触发、修改关键词、调整 skill input、增加链式 search，都应修改 JSON 分片。

## Inference 规则

inference 的业务解析在 `backend/config/json/agent/worker-inference/`。

Rust 只负责：

- 通用数字解析。
- 通用 metric 解析。
- 根据配置读取 alias、defaults、labels、rules。
- 为 autosuggest 模板提供结构化值。

新增别名、分类标签、默认预算、默认天数、搜索参数、平台/行业识别，优先修改 JSON 配置。

## Output 规则

输出补全、边界说明、默认 findings/recommendations/risks、normalization 清洗规则，分别放在：

- `backend/config/json/agent/worker-output-policy.json`
- `backend/config/json/agent/worker-output-normalization-policy.json`

回答风格、角色边界、教学深度和输出格式优先放在 prompt 或 output policy，不写进 runner。

## Prompt 规则

prompt 放在：

- `backend/config/prompts/zh/cognition/`
- `backend/config/prompts/zh/roles/`
- `backend/config/prompts/en/cognition/`

不同 worker 需要不同 prompt 时，应增加或修改对应 prompt 文件，并通过配置选择；不要向 cognition Rust 代码里塞业务 prompt。

## 修改后的验证

修改 runner 配置后至少运行：

```bash
jq empty config/json/agent/*.json config/json/agent/worker-autosuggest/*.json config/json/agent/worker-inference/*.json
cargo fmt
cargo check
cargo clippy -- -D warnings
```

如果只修改 Markdown 文档，可以不运行 cargo；如果修改 JSON 配置，必须至少运行 `jq empty`。
