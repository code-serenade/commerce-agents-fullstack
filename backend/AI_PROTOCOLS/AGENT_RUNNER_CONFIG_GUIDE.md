# Agent Runner 配置化协议

本文档约束 `agent` crate 里的 worker runner。runner 的业务能力必须优先通过配置完成，Rust 代码只允许保留通用执行引擎。

## 核心原则

1. runner 是一次任务执行过程中的短生命周期组件，由外部 agent manager 监管；任务完成后释放。
2. runner 不负责维护长期业务状态，不重新设计 memory、cognition、skill 或 manager。
3. 角色、任务图、autosuggest、inference、输出策略和 prompt 都必须优先放在 `backend/config/` 下。
4. 普通业务调整只允许修改配置文件，不允许修改 runner Rust 代码。
5. Rust 侧只保留“读取配置、匹配配置、执行 skill、组装上下文、校验输出、降级处理”等通用能力。

## 配置入口

业务配置入口统一在：

- `backend/config/json/agent/roles.json`
- `backend/config/json/agent/task-graph/rules.json`
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

只有下面情况可以修改 runner Rust 代码：

1. 新增一种通用配置字段，并同步更新配置说明。
2. 新增一种通用模板占位符求值器，例如把配置里的 `{{field}}` 解析为结构化 skill input。
3. 修复通用执行 bug、错误处理、日志或性能问题。
4. 接入新的通用 skill executor 边界。
5. 删除已经被配置完全覆盖的旧代码。

即使需要修改 Rust，也必须保持代码与业务解耦：Rust 里不能写具体行业方案、角色话术、业务关键词列表、平台规则文案或 skill 路由规则。

## 禁止事项

1. 不允许在 runner Rust 文件里新增大段业务 if/else、match 或关键词数组。
2. 不允许把 role prompt、回答模板、教学深度、角色边界、输出格式写死进 Rust。
3. 不允许把 autosuggest skill 路由写死进 Rust。
4. 不允许把 inference 的别名、默认值、分类标签、搜索关键词写死进 Rust。
5. 不允许在 runner 里重新实现 cognition；cognition 已有独立目录，prompt 从 `backend/config/prompts/` 读取。
6. 不允许在 runner 里重新实现 memory；长期记忆和 session 记忆应走既有 agent session/memory 边界。
7. 不允许把 `old_runner` 里的旧业务代码复制回新 runner。

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
