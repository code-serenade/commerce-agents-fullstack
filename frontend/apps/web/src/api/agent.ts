const AGENT_STREAM_URL = "/api/sse/agent/session/stream"

export interface AgentStreamRequest {
  input: string
  agentId?: string
  modelName?: string
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
  | { type: "commander_thinking" }
  | { type: "worker_started"; workerId: string; taskId: string }
  | { type: "worker_finished"; workerId: string; taskId: string }
  | {
      type: "collaboration_started"
      primaryWorkerId: string
      supportWorkerIds: string[]
    }
  | { type: "collaboration_worker_started"; workerId: string; taskId: string }
  | {
      type: "collaboration_worker_finished"
      workerId: string
      taskId: string
      content?: string | null
    }
  | { type: "collaboration_finished" }
  | { type: "task_graph_planned"; graph: AgentTaskGraph }
  | {
      type: "task_node_ready"
      graphId: string
      nodeId: string
      workerId: string
    }
  | {
      type: "task_node_started"
      graphId: string
      nodeId: string
      workerId: string
      attempt: number
    }
  | {
      type: "task_node_reported"
      graphId: string
      nodeId: string
      workerId: string
      report: AgentWorkerReport
    }
  | {
      type: "task_node_evaluated"
      graphId: string
      evaluation: AgentTaskEvaluation
    }
  | {
      type: "task_node_rework_requested"
      graphId: string
      nodeId: string
      workerId: string
      attempt: number
      instruction: string
    }
  | { type: "task_node_passed"; graphId: string; nodeId: string }
  | {
      type: "task_node_failed"
      graphId: string
      nodeId: string
      reason: string
    }
  | { type: "task_graph_finished"; graphId: string }

type AgentWrappedId = string | [string] | { 0: string }

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
  const token = localStorage.getItem("token")
  const response = await fetch(AGENT_STREAM_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify(body),
    signal: handlers.signal,
  })

  if (!response.ok || !response.body) {
    handlers.onError(`Agent stream failed: ${response.status}`)
    return
  }

  const reader = response.body.getReader()
  const decoder = new TextDecoder("utf-8")
  let buffer = ""

  while (true) {
    const { value, done } = await reader.read()
    if (done) break

    buffer += decoder.decode(value, { stream: true })
    const boundary = buffer.lastIndexOf("\n\n")
    if (boundary === -1) continue

    const completed = buffer.slice(0, boundary)
    buffer = buffer.slice(boundary + 2)

    for (const block of completed.split("\n\n")) {
      if (handleSseMessage(parseSseMessage(block), handlers)) return
    }
  }

  if (buffer.trim()) {
    handleSseMessage(parseSseMessage(buffer), handlers)
    return
  }

  handlers.onDone()
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
): boolean {
  if (!message.data) return false

  if (message.event === "done") {
    handlers.onDone()
    return true
  }

  if (message.event === "error") {
    handlers.onError(readJsonField(message.data, "message") || "Agent failed")
    return true
  }

  if (message.event === "delta") {
    const content = readJsonField(message.data, "content")
    if (content) handlers.onDelta(content)
  }

  if (message.event === "agent_event") {
    const event = readAgentRuntimeEvent(message.data)
    if (event) handlers.onAgentEvent?.(event)
  }

  return false
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
    return isAgentRuntimeEvent(parsed) ? parsed : null
  } catch {
    return null
  }
}

function isAgentRuntimeEvent(value: unknown): value is AgentRuntimeEvent {
  return (
    typeof value === "object" &&
    value !== null &&
    "type" in value &&
    typeof (value as { type?: unknown }).type === "string"
  )
}
