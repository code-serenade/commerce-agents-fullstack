# Agent Swarm Configuration

This project keeps the existing commerce agent runtime and exposes it through a Kimi Agent Swarm style UI.
The runtime is configured in four layers: LLM, role, graph, and prompt.

## LLM

Runtime LLMs are configured in `backend/config/services.toml`.

```toml
default_llm = "external-main"

[[llm]]
name = "external-main"
base_url = "https://CHANGE_ME_PUBLIC_OPENAI_COMPATIBLE_BASE_URL"
model = "CHANGE_ME_MODEL_NAME"
api_key = "CHANGE_ME"
max_tokens = 20000
temperature = 0.2
```

Rules:

- `base_url` must be reachable from the backend machine.
- The client calls an OpenAI-compatible chat completions API.
- If the provider path is not the default, set `chat_completions_endpoint`. When `base_url` already ends with `/v1`, `/api/v1`, or `/api/v3`, the client automatically calls `chat/completions`.
- `default_llm` selects one entry by `name`. If omitted, runtime options prefer a real configured Doubao entry whose `name` or `model` contains `doubao`; otherwise the first `[[llm]]` entry is used.
- Model selection is server-side only. Frontend requests do not expose or override LLM config.
- Local Ollama is still supported, for example `base_url = "http://127.0.0.1:11434"`.

Doubao Ark example:

```toml
[[llm]]
name = "doubao-external"
base_url = "https://ark.cn-beijing.volces.com/api/v3"
model = "doubao-seed-2-0-pro-260215"
api_key = "CHANGE_ME"
max_tokens = 20000
temperature = 0.2
```

The current local config uses:

```toml
[[llm]]
name = "ollama-gemma4"
base_url = "http://127.0.0.1:11434"
model = "gemma4:26b"
```

If there is no service on that address, agent execution will plan the task graph but worker nodes will fail when they request `/v1/chat/completions`.

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

Task graph rules are configured in `backend/config/json/agent/task-graph-rules.json`.

The current Douyin holiday conversion rule maps a user task into:

```text
data
  -> accounting
  -> douyin_ads
  -> creative
accounting + douyin_ads + creative
  -> ops
```

The frontend swarm panel reads the live SSE `task_graph_planned` event, so new graph nodes will appear without hardcoding as long as the backend emits them.

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

The backend uses `default_llm` from `backend/config/services.toml`. If it is omitted and a real Doubao entry is configured, the runtime options default to that Doubao entry; otherwise they default to the first configured LLM.

Important SSE events:

- `task_graph_planned`: renders the dynamic agent graph.
- `task_node_started`: marks an agent as working.
- `task_node_reported`: shows `report.content` or structured `role_output`.
- `task_node_rework_requested`: marks retry state.
- `task_node_failed`: marks failed state and explains the blocker.
- `task_graph_finished`: closes the execution run.

The Kimi-style panel is implemented in:

```text
frontend/apps/web/src/pages/site/agent-chat/components/agent-chat-swarm-panel.tsx
```
