# Agent Swarm Configuration

This project keeps the existing commerce agent runtime and exposes it through a Kimi Agent Swarm style UI.
The runtime is configuration-driven. Business changes should be made in `backend/config/`, not in runner Rust code.

## LLM

Runtime LLMs are configured in `backend/config/services.toml`.

```toml
[agent_llm_usage]
default = "external-main"
fast = "external-main"
thinking = "external-main"

[[llm]]
name = "external-main"
provider = "chat_completions"
base_url = "https://CHANGE_ME_PUBLIC_OPENAI_COMPATIBLE_BASE_URL"
model = "CHANGE_ME_MODEL_NAME"
api_key = "CHANGE_ME"
max_tokens = 20000
temperature = 0.2
```

Rules:

- `base_url` must be reachable from the backend machine.
- `provider` selects the model-gateway-rs client: `chat_completions`, `ollama`, or `doubao`.
- If the `chat_completions` provider path is not the default, set `chat_completions_endpoint`.
- `[agent_llm_usage]` is the only model routing entry. `default` selects the fallback `[[llm]].name`; if omitted, the first `[[llm]]` entry is used.
- Model selection is server-side only. Frontend requests do not expose or override LLM config.
- Local Ollama is supported with `provider = "ollama"` and a root URL such as `base_url = "http://127.0.0.1:11434"`.
- Doubao is available only when a real Ark `[[llm]]` entry and API key exist in `services.toml`. The frontend must not display a fake Doubao option when the backend config does not contain one.

Doubao Ark example:

```toml
[[llm]]
name = "doubao-external"
provider = "doubao"
base_url = "https://ark.cn-beijing.volces.com/api/v3"
model = "doubao-seed-2-0-pro-260215"
api_key = "CHANGE_ME"
max_tokens = 20000
temperature = 0.2
```

The current local config in this repository uses Ollama-compatible remote entries:

```toml
[[llm]]
name = "ollama-gpt-oss-20b"
base_url = "http://119.29.232.53:16814/v1"
model = "gpt-oss:20b"

[[llm]]
name = "ollama-gemma4"
base_url = "http://119.29.232.53:16814/v1"
model = "gemma4:26b"
```

If a deployment needs Doubao, add the Ark entry on that machine and map `[agent_llm_usage]` to that `llm.name`. Do not commit real keys.

## Role

Roles are configured in `backend/config/json/agent/roles.json`.

Each role declares:

- `runtime_role`: the role id used by graph nodes and worker routing.
- `keywords`: terms that help the commander route tasks.
- `required_capabilities`: hard capability boundary.
- `optional_capabilities`: supporting capability hints.

Example:

```json
{
  "id": "role.douyin-ads",
  "name": "千川投放专员",
  "runtime_role": "douyin_ads",
  "keywords": ["千川", "投流", "巨量千川", "ROI", "人群包"],
  "required_capabilities": [
    "ads.optimization",
    "roi.control",
    "audience.segmentation",
    "campaign.pacing"
  ],
  "optional_capabilities": [
    "data.analysis",
    "creative.content",
    "budget.planning",
    "conversion.optimization"
  ]
}
```

## Graph

Task graph rules are configured through `backend/config/json/agent/task-graph/index.json`.
Each graph lives in its own file under `backend/config/json/agent/task-graph/graphs/`.

The current commerce team analysis graph maps a user task into:

```text
data
  -> creative
  -> service
  -> engineering
  -> douyin_ads/accounting/design/web when matched
all non-final nodes
  -> ops
```

The frontend swarm panel reads the live SSE `task_graph_planned` event, so new graph nodes will appear without hardcoding as long as the backend emits them.

## Worker Runner

Worker runner business behavior is configured in `backend/config/json/agent/`.

| Config | Purpose |
| --- | --- |
| `worker-autosuggest/` | Automatically requests skills from worker text, role, and inferred context. |
| `worker-inference/` | Parses aliases, numeric metrics, labels, defaults, search inputs, and selected-product context. |
| `worker-output-policy.json` | Fills default role output, category boundaries, fallback findings, recommendations, and risks. |
| `worker-output-normalization-policy.json` | Cleans runtime noise, downgrades missing data safely, and prevents unsupported benchmark claims. |

Runner Rust should only load config, match rules, render templates, execute skills, and normalize generic output.
Do not add business keywords, role scripts, platform policy text, or skill routing branches to runner Rust.

For the full protocol and editing guide, see:

- `backend/AI_PROTOCOLS/AGENT_RUNNER_CONFIG_GUIDE.md`
- `backend/config/json/agent/README.md`

## Prompt

Role prompts are stored in `backend/config/prompts/zh/roles/`.

Use role prompts to define:

- Role boundary.
- Required inputs.
- Output shape.
- What to do when required data is missing.
- How to populate `role_output.summary`, `findings`, `recommendations`, `evidence`, `risks`, and `open_questions`.

For the Douyin role, see `backend/config/prompts/zh/roles/douyin_ads.md`.

## Frontend Event Flow

The web UI listens to `/api/sse/agent/session/stream`.

Request body:

```json
{
  "input": "我是做咖啡运营的...",
  "agentId": "web-agent",
  "sessionId": "web-session-001",
  "workspaceId": "workspace-001"
}
```

The backend uses `[agent_llm_usage]` from `backend/config/services.toml`. If `default` is omitted, runtime options and Agent runtime default to the first configured `[[llm]]`.

Important SSE events:

- `task_graph_planned`: renders the dynamic agent graph.
- `task_node_started`: marks an agent as working.
- `task_node_reported`: shows `report.content` or structured `role_output`.
- `task_node_rework_requested`: marks retry state.
- `task_node_failed`: marks failed state and explains the blocker.
- `task_graph_finished`: closes the execution run.

Final Markdown output must stay readable across answer modes and use the existing answer renderer's supported shapes: Chinese headings, blank lines, bullet points, bracket headings, colon headings, and Markdown pipe tables when tabular delivery is useful.

```text
## 综合回答
## 关键判断
## 执行方案
## 风险边界
## 下一步
```

The exact headings are chosen by the current deliverable. Product-material answers may also use headings such as `【核心业务结论与定价判断】`, `适合部分：`, and `核验动作：`; execution answers may use `## 执行方案` plus a Markdown pipe table; teaching answers may use `## 判断逻辑` and `## 下一步练习`. The answer should not mechanically force every mode into a fixed six-section skeleton.

`metadata.strategy.responseMode` changes density inside that layout:

- `quick`: shorter answer, direct reasons, compact role table, no full execution table.
- `learning`: student-friendly explanations, concepts, boundaries, and follow-up learning prompts.
- `execution`: richer action table with owner, acceptance criteria, risk boundary, and review cadence.

`metadata.strategy.learningLevel` further adjusts explanation depth inside teaching-oriented sections. Low depth reduces terminology and long process details; high depth adds reasoning, term explanation, risk boundary, and next-question suggestions.

The Kimi-style panel is implemented in:

```text
frontend/apps/web/src/pages/site/agent-chat/components/agent-chat-swarm-panel.tsx
```
