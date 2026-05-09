const AGENT_STREAM_URL = "/api/sse/agent/session/stream"
const AGENT_STREAM_IDLE_TIMEOUT_MS = 60_000
const AUTH_REFRESH_URL = "/auth/session/refresh_token"

export interface AgentStreamRequest {
  input: string
  agentId?: string
  metadata?: {
    context?: {
      contextRefs?: string[]
      productIds?: string[]
    }
    strategy?: {
      collaborationMode?: string
      id?: string
      responseMode?: string
    }
  }
  sessionId?: string
  workspaceId?: string
}

export interface AgentStreamHandlers {
  onDelta: (content: string) => void
  onDone: () => void
  onError: (message: string) => void
  onAgentEvent?: (event: AgentRuntimeEvent) => void
  signal?: AbortSignal
}

interface SseMessage {
  event: string
  data: string
}

export type AgentRuntimeEvent =
  | { type: "started"; session_id?: string }
  | { type: "memory_loaded"; count?: number; kinds?: string[]; session_id?: string }
  | {
      type: "context_loaded"
      contextRefs?: string[]
      knowledge_count?: number
      product_count?: number
      product_ids?: string[]
      productIds?: string[]
      refs?: string[]
      session_id?: string
      source_count?: number
      sources_json?: string | null
      summary?: string | null
    }
  | { type: "finished"; session_id?: string }
  | { type: "failed"; reason: string; session_id?: string }
  | { type: "output"; content: string; session_id?: string }
  | { type: "llm_chunk"; content: string; session_id?: string }
  | { type: "commander_thinking" }
  | {
      type: "route_decided"
      collaboration_mode?: string | null
      primary_runtime_role?: string | null
      response_mode?: string | null
      route_mode: string
      strategy_id?: string | null
      support_runtime_roles: string[]
    }
  | {
      type: "worker_started"
      workerId?: string
      worker_id?: string
      taskId?: string
      task_id?: string
    }
  | {
      type: "worker_finished"
      workerId?: string
      worker_id?: string
      taskId?: string
      task_id?: string
    }
  | {
      type: "skill_requested"
      workerId?: string
      worker_id?: string
      taskId?: string
      task_id?: string
      request_id: string
      skill_id: string
      role: string
      reason?: string | null
      session_id?: string
    }
  | {
      type: "skill_result"
      workerId?: string
      worker_id?: string
      taskId?: string
      task_id?: string
      request_id: string
      skill_id: string
      role: string
      status: string
      summary?: string | null
      error?: string | null
      execution_mode?: string | null
      risk_level?: string | null
      approval_required?: boolean | null
      session_id?: string
    }
  | {
      type: "collaboration_started"
      primaryWorkerId?: string
      primary_worker_id?: string
      supportWorkerIds?: string[]
      support_worker_ids?: string[]
    }
  | {
      type: "collaboration_worker_started"
      workerId?: string
      worker_id?: string
      taskId?: string
      task_id?: string
    }
  | {
      type: "collaboration_worker_finished"
      workerId?: string
      worker_id?: string
      taskId?: string
      task_id?: string
      content?: string | null
    }
  | { type: "collaboration_finished" }
  | { type: "task_graph_planned"; graph: AgentTaskGraph }
  | {
      type: "task_node_ready"
      graphId?: string
      graph_id?: string
      nodeId?: string
      node_id?: string
      workerId?: string
      worker_id?: string
    }
  | {
      type: "task_node_started"
      graphId?: string
      graph_id?: string
      nodeId?: string
      node_id?: string
      workerId?: string
      worker_id?: string
      attempt: number
    }
  | {
      type: "task_node_reported"
      graphId?: string
      graph_id?: string
      nodeId?: string
      node_id?: string
      workerId?: string
      worker_id?: string
      report: AgentWorkerReport
    }
  | {
      type: "task_node_evaluated"
      graphId?: string
      graph_id?: string
      evaluation: AgentTaskEvaluation
    }
  | {
      type: "task_node_rework_requested"
      graphId?: string
      graph_id?: string
      nodeId?: string
      node_id?: string
      workerId?: string
      worker_id?: string
      attempt: number
      instruction: string
    }
  | {
      type: "task_node_passed"
      graphId?: string
      graph_id?: string
      nodeId?: string
      node_id?: string
    }
  | {
      type: "task_node_failed"
      graphId?: string
      graph_id?: string
      nodeId?: string
      node_id?: string
      reason: string
    }
  | { type: "task_graph_finished"; graphId?: string; graph_id?: string }

type AgentWrappedId = string | [string] | { 0: string }

const AGENT_RUNTIME_EVENT_TYPES = new Set<string>([
  "started",
  "memory_loaded",
  "context_loaded",
  "finished",
  "failed",
  "output",
  "llm_chunk",
  "commander_thinking",
  "route_decided",
  "worker_started",
  "worker_finished",
  "skill_requested",
  "skill_result",
  "collaboration_started",
  "collaboration_worker_started",
  "collaboration_worker_finished",
  "collaboration_finished",
  "task_graph_planned",
  "task_node_ready",
  "task_node_started",
  "task_node_reported",
  "task_node_evaluated",
  "task_node_rework_requested",
  "task_node_passed",
  "task_node_failed",
  "task_graph_finished",
])

export interface AgentTaskGraph {
  graph_id: AgentWrappedId
  root_task: string
  nodes: AgentTaskNode[]
}

export interface AgentTaskNode {
  id: AgentWrappedId
  role: string
  worker_id: AgentWrappedId
  title: string
  objective: string
  input_refs: AgentWrappedId[]
  acceptance: string[]
  max_attempts: number
}

export interface AgentWorkerReport {
  graph_id: AgentWrappedId
  node_id: AgentWrappedId
  task_id: AgentWrappedId
  worker_id: AgentWrappedId
  role: string
  content: string
  role_output?: {
    summary: string
    findings: string[]
    recommendations: string[]
    evidence: string[]
    risks: string[]
    open_questions: string[]
  } | null
  evidence: string[]
  risks: string[]
  open_questions: string[]
  status: string
}

export interface AgentTaskEvaluation {
  node_id: AgentWrappedId
  passed: boolean
  score: number
  reasons: string[]
  rework_instruction?: string | null
}

export async function streamAgentSessionApi(
  body: AgentStreamRequest,
  handlers: AgentStreamHandlers
) {
  let settled = false
  let receivedDelta = false
  let fallbackOutput = ""
  let reader: ReadableStreamDefaultReader<Uint8Array> | null = null
  const idleTimeout = window.setTimeout(() => {
    if (settled) return
    settled = true
    handlers.onError("Agent stream timed out without output.")
    void reader?.cancel()
  }, AGENT_STREAM_IDLE_TIMEOUT_MS)

  let response = await fetchAgentStream(body, handlers.signal)
  if (response.status === 401 && (await refreshAgentStreamToken())) {
    response = await fetchAgentStream(body, handlers.signal)
  }

  if (!response.ok || !response.body) {
    window.clearTimeout(idleTimeout)
    handlers.onError(`Agent stream failed: ${response.status}`)
    return
  }

  reader = response.body.getReader()
  const decoder = new TextDecoder("utf-8")
  let buffer = ""

  try {
    while (true) {
      const { value, done } = await reader.read()
      if (done) break

      buffer += decoder.decode(value, { stream: true })
      const boundary = buffer.lastIndexOf("\n\n")
      if (boundary === -1) continue

      const completed = buffer.slice(0, boundary)
      buffer = buffer.slice(boundary + 2)

      for (const block of completed.split("\n\n")) {
        const result = handleSseMessage(parseSseMessage(block), handlers)
        receivedDelta = receivedDelta || result.receivedDelta
        fallbackOutput += result.fallbackOutput
        if (result.finished) {
          if (!receivedDelta && fallbackOutput) {
            handlers.onDelta(fallbackOutput)
          }
          if (result.completed) {
            handlers.onDone()
          }
          settled = true
          return
        }
      }
    }

    if (buffer.trim()) {
      const result = handleSseMessage(parseSseMessage(buffer), handlers)
      receivedDelta = receivedDelta || result.receivedDelta
      fallbackOutput += result.fallbackOutput
      if (result.finished) {
        if (!receivedDelta && fallbackOutput) {
          handlers.onDelta(fallbackOutput)
        }
        if (result.completed) {
          handlers.onDone()
        }
        settled = true
        return
      }
    }

    if (!settled) {
      if (!receivedDelta && fallbackOutput) {
        handlers.onDelta(fallbackOutput)
      }
      handlers.onDone()
      settled = true
    }
  } finally {
    window.clearTimeout(idleTimeout)
    if (!receivedDelta && fallbackOutput && !settled && !handlers.signal?.aborted) {
      handlers.onDelta(fallbackOutput)
    }
    if (!receivedDelta && !fallbackOutput && !settled && !handlers.signal?.aborted) {
      handlers.onError("Agent stream ended without output.")
    }
  }
}

function fetchAgentStream(body: AgentStreamRequest, signal?: AbortSignal) {
  const token = localStorage.getItem("token")

  return fetch(AGENT_STREAM_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify(body),
    signal,
  })
}

async function refreshAgentStreamToken(): Promise<boolean> {
  const refreshToken = localStorage.getItem("refreshToken")
  if (!refreshToken) return false

  const response = await fetch(AUTH_REFRESH_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ refreshToken }),
  })
  if (!response.ok) return false

  const json = (await response.json().catch(() => ({}))) as {
    data?: { accessToken?: string; access_token?: string }
  }
  const token = json.data?.accessToken ?? json.data?.access_token
  if (!token) return false

  localStorage.setItem("token", token)
  return true
}

function parseSseMessage(block: string): SseMessage {
  let event = "message"
  const data: string[] = []

  for (const line of block.split("\n")) {
    const trimmed = line.trimEnd()
    if (trimmed.startsWith("event:")) {
      event = trimmed.slice("event:".length).trim()
    }
    if (trimmed.startsWith("data:")) {
      data.push(trimmed.slice("data:".length).trimStart())
    }
  }

  return {
    event,
    data: data.join("\n"),
  }
}

function handleSseMessage(
  message: SseMessage,
  handlers: AgentStreamHandlers
): {
  completed: boolean
  fallbackOutput: string
  finished: boolean
  receivedDelta: boolean
} {
  if (message.event === "done" || message.event === "finished") {
    const event = readAgentRuntimeEvent(message.data)
    if (event) handlers.onAgentEvent?.(event)
    return {
      completed: true,
      fallbackOutput: "",
      finished: true,
      receivedDelta: false,
    }
  }

  if (!message.data) {
    return {
      completed: false,
      fallbackOutput: "",
      finished: false,
      receivedDelta: false,
    }
  }

  if (message.event === "error") {
    handlers.onError(readJsonField(message.data, "message") || "Agent failed")
    return {
      completed: false,
      fallbackOutput: "",
      finished: true,
      receivedDelta: false,
    }
  }

  if (message.event === "delta") {
    const content = readJsonField(message.data, "content")
    if (content) handlers.onDelta(content)
    return {
      completed: false,
      fallbackOutput: "",
      finished: false,
      receivedDelta: Boolean(content),
    }
  }

  if (message.event === "output") {
    const event = readAgentRuntimeEvent(message.data)
    if (event) handlers.onAgentEvent?.(event)
    return {
      completed: false,
      fallbackOutput: readJsonField(message.data, "content"),
      finished: false,
      receivedDelta: false,
    }
  }

  if (message.event === "agent_event" || AGENT_RUNTIME_EVENT_TYPES.has(message.event)) {
    const event = readAgentRuntimeEvent(message.data)
    if (event) handlers.onAgentEvent?.(event)
  }

  return {
    completed: false,
    fallbackOutput: "",
    finished: false,
    receivedDelta: false,
  }
}

function readJsonField(data: string, field: string): string {
  try {
    const parsed = JSON.parse(data)
    const value = parsed?.[field]
    return typeof value === "string" ? value : ""
  } catch {
    return ""
  }
}

function readAgentRuntimeEvent(data: string): AgentRuntimeEvent | null {
  try {
    const parsed = JSON.parse(data)
    return isAgentRuntimeEvent(parsed) ? normalizeAgentRuntimeEvent(parsed) : null
  } catch {
    return null
  }
}

function normalizeAgentRuntimeEvent(event: AgentRuntimeEvent): AgentRuntimeEvent {
  const normalized = { ...event } as AgentRuntimeEvent & Record<string, unknown>
  const raw = event as AgentRuntimeEvent & Record<string, unknown>

  copyIfMissing(normalized, raw, "workerId", "worker_id")
  copyIfMissing(normalized, raw, "taskId", "task_id")
  copyIfMissing(normalized, raw, "graphId", "graph_id")
  copyIfMissing(normalized, raw, "nodeId", "node_id")
  copyIfMissing(normalized, raw, "primaryWorkerId", "primary_worker_id")
  copyIfMissing(normalized, raw, "supportWorkerIds", "support_worker_ids")
  copyIfMissing(normalized, raw, "productIds", "product_ids")
  copyIfMissing(normalized, raw, "contextRefs", "refs")

  return normalized
}

function copyIfMissing(
  target: Record<string, unknown>,
  source: Record<string, unknown>,
  camelKey: string,
  snakeKey: string
) {
  if (target[camelKey] === undefined && source[snakeKey] !== undefined) {
    target[camelKey] = source[snakeKey]
  }
}

function isAgentRuntimeEvent(value: unknown): value is AgentRuntimeEvent {
  return (
    typeof value === "object" &&
    value !== null &&
    "type" in value &&
    typeof (value as { type?: unknown }).type === "string" &&
    AGENT_RUNTIME_EVENT_TYPES.has((value as { type: string }).type)
  )
}
