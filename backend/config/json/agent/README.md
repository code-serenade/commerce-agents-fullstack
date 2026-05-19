# Agent 配置说明

这个目录保存 agent runner 的业务配置。未来普通业务调整只改这里和 `backend/config/prompts/`，不要修改 runner Rust 代码。

## 文件职责

| 路径 | 作用 |
| --- | --- |
| `roles.json` | 定义可用角色、角色关键词、能力边界和 runtime role。 |
| `task-graph/` | 定义 commander 如何把用户任务拆成角色任务图。入口是 `task-graph/index.json`，单个图放在 `task-graph/graphs/`。 |
| `worker-autosuggest/` | 定义 worker 在执行时自动追加哪些 skill request。 |
| `worker-inference/` | 定义从用户文本里解析指标、别名、默认值、分类标签和搜索参数。 |
| `worker-output-policy.json` | 定义 worker 输出缺省补全、角色默认 findings/recommendations/risks 和边界说明。 |
| `worker-output-normalization-policy.json` | 定义输出清洗、风险降级、禁止伪造数据和文本替换策略。 |

旧文件 `worker-autosuggest-policy.json` 和 `worker-inference-policy.json` 只作为兼容 fallback，新规则不要继续写进去。旧的 `task-graph-rules.json` 已删除，不再维护第二份 graph 配置。

## Task Graph 配置

入口文件：

```text
task-graph/index.json
```

目录结构：

```text
task-graph/
  index.json
  defaults.json
  role-aliases.json
  role-defaults.json
  graphs/
    commerce-team-analysis.json
```

`index.json` 定义：

- `defaults_file`：全局默认尝试次数、验收要求、默认 role order 和显式团队关键词。
- `role_aliases_file`：用户文本里角色别名到 runtime role 的映射。
- `role_defaults_file`：每个 role 的默认 title/objective。
- `files`：按顺序加载的 graph rule 文件。每个文件只放一个图。

新增业务编排时，优先在 `graphs/` 下新增一个 JSON 文件，然后把路径加入 `index.json`。不要把业务编排写进 Rust。

## Prompts

prompt 不在本目录，统一放在：

```text
backend/config/prompts/
```

常用路径：

- `backend/config/prompts/zh/cognition/`
- `backend/config/prompts/zh/roles/`
- `backend/config/prompts/en/cognition/`

回答风格、角色边界、教学深度、输出格式和 worker 的语言习惯，应修改 prompt 或 output policy，不写进 Rust。

## Autosuggest 配置

入口文件：

```text
worker-autosuggest/index.json
```

`index.json` 定义：

- `role_routes`：每个 runtime role 使用哪种 autosuggest 模式。
- `files`：按顺序加载哪些分片。

业务规则写在各角色分片，例如：

- `worker-autosuggest/ops.json`
- `worker-autosuggest/data.json`
- `worker-autosuggest/douyin_ads.json`
- `worker-autosuggest/search.json`

规则字段：

| 字段 | 说明 |
| --- | --- |
| `id` | 规则唯一标识，方便日志和排查。 |
| `roles` | 规则适用的 runtime role。为空表示不限角色。 |
| `fallback` | 只在没有普通规则命中时使用。 |
| `mode` | `append` 表示可继续追加；`first` 表示命中后停止。 |
| `match.any_keywords` | 任一关键词命中即可。 |
| `match.all_keywords` | 所有关键词都必须命中。 |
| `match.none_keywords` | 命中任一关键词则排除。 |
| `match.requires_any_skill_ids` | 只有前面已经生成过这些 skill 之一时才命中，用于链式追加。 |
| `match.requires_no_skill_ids` | 前面已经生成过这些 skill 时排除。 |
| `skill_id` | 要自动请求的 skill。 |
| `input` | 传给 skill 的 JSON input，可使用模板占位符。 |
| `reason` | 可选，覆盖默认 autosuggest reason。 |

示例：

```json
{
  "id": "ops_promo_policy_search",
  "roles": ["ops"],
  "match": {
    "any_keywords": ["平台规则", "活动规则"],
    "requires_any_skill_ids": ["ops_promo_planning"]
  },
  "skill_id": "search_platform_policy",
  "input": {
    "platform": "{{search_platform}}",
    "policy_type": "{{search_policy_type}}",
    "days": "{{search_days}}"
  }
}
```

## Inference 配置

入口文件：

```text
worker-inference/index.json
```

分片按领域拆分：

- `labels.json`：标签和通用分类。
- `commerce.json`：价格、成本、预算、库存天数等经营参数。
- `data.json`：数据指标、诊断焦点、AB 测试、预测等。
- `inventory.json`：库存与补货参数。
- `search.json`：搜索 query、行业、平台、topic、days 等。
- `context.json`：上下文和已选商品相关配置。
- `douyin-ads.json`：抖音投放指标别名和默认值。
- `memory-task.json`：记忆和任务协同解析。
- `numbers.json`：数字解析默认值和限制。

可以调整的内容包括：

- `aliases`：指标别名，比如“消耗 / 花费 / ad_spend”。
- `defaults`：默认预算、默认天数、默认平台等。
- `labels`：分类标签和关键词。
- `rules`：解析限制、过滤条件、clamp 范围等。

## Output 配置

`worker-output-policy.json` 负责“缺省补全”和“角色输出边界”。

常见调整：

- 某个角色默认要补几条 findings。
- 没有真实数据时如何解释边界。
- 哪些 skill 只作为边界记录，不视为失败。
- 类目级问题不能注入默认商品的提示文案。

`worker-output-normalization-policy.json` 负责“清洗和降级”。

常见调整：

- 禁止无来源行业基准。
- 清理运行时错误噪音。
- 将 skill failure 降级为可读边界。
- 避免未选商品时输出具体商品身份。

## 修改规则

允许：

- 增删关键词。
- 调整 skill input。
- 调整 fallback 文案。
- 调整默认值。
- 调整 prompt 文本。
- 增加新的 autosuggest JSON 规则。

不允许：

- 把业务规则写进 runner Rust。
- 把 prompt 文案写进 runner Rust。
- 把平台规则、行业方案、角色话术写进 Rust。
- 修改 skill 代码来绕过配置。
- 修改旧 fallback 单文件来新增业务。

如果现有配置能力不够，先扩展通用配置字段，再更新本文档和 `backend/AI_PROTOCOLS/AGENT_RUNNER_CONFIG_GUIDE.md`。

## 校验

只改 JSON 后至少运行：

```bash
cd backend
jq empty config/json/agent/*.json config/json/agent/worker-autosuggest/*.json config/json/agent/worker-inference/*.json
```

如果改了 Rust 或新增配置字段，再运行：

```bash
cd backend
cargo fmt
cargo check
cargo clippy -- -D warnings
```
