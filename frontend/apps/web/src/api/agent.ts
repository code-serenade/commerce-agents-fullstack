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
    model?: {
      name?: string
    }
    strategy?: {
      collaborationMode?: string
      id?: string
      learningLevel?: string
      primaryRole?: string
      responseMode?: string
      selectedRoles?: string[]
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
  | ({ type: "rag_context_loaded" } & AgentRagContextLoaded)
  | { type: "finished"; session_id?: string }
  | { type: "failed"; reason: string; session_id?: string }
  | { type: "output"; content: string; session_id?: string }
  | { type: "llm_chunk"; content: string; session_id?: string }
  | { type: "commander_thinking" }
  | {
      type: "route_decided"
      collaboration_mode?: string | null
      collaborationMode?: string | null
      learning_level?: string | null
      learningLevel?: string | null
      model_name?: string | null
      modelName?: string | null
      primary_runtime_role?: string | null
      primaryRuntimeRole?: string | null
      response_mode?: string | null
      responseMode?: string | null
      route_mode: string
      routeMode?: string
      strategy_id?: string | null
      strategyId?: string | null
      support_runtime_roles: string[]
      supportRuntimeRoles?: string[]
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
  | {
      type: "workspace_run_linked"
      createdWorkspace?: boolean
      created_workspace?: boolean
      run: AgentWorkspaceRun
      workspace: AgentWorkspace
    }
  | { type: "workspace_run_tasks_linked"; tasks: AgentWorkspaceRunTask[] }
  | ({ type: "workspace_run_task_updated" } & AgentWorkspaceRunTask)
  | ({ type: "workspace_run_completed" } & AgentWorkspaceRun)
  | ({ type: "workspace_run_failed" } & AgentWorkspaceRun)
  | ({ type: "workspace_artifact_created" } & AgentWorkspaceArtifact)
  | ({ type: "quality_checked" } & AgentQualityEvaluation)
  | ({ type: "approval_required" } & AgentApprovalRequired)
  | {
      type: "workspace_run_persistence_error"
      action: string
      code?: number
      message: string
    }

type AgentWrappedId = string | [string] | { 0: string }

const AGENT_RUNTIME_EVENT_TYPES = new Set<string>([
  "started",
  "memory_loaded",
  "context_loaded",
  "rag_context_loaded",
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
  "workspace_run_linked",
  "workspace_run_tasks_linked",
  "workspace_run_task_updated",
  "workspace_run_completed",
  "workspace_run_failed",
  "workspace_artifact_created",
  "quality_checked",
  "approval_required",
  "workspace_run_persistence_error",
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

export interface AgentWorkspace {
  config?: Record<string, unknown>
  createdAt?: string
  created_at?: string
  id: string
  status?: string
  title: string
  updatedAt?: string
  updated_at?: string
  userId?: string
  user_id?: string
  workspaceType?: string
  workspace_type?: string
}

export interface AgentWorkspaceRun {
  createdAt?: string
  created_at?: string
  goal?: string
  id: string
  metadata?: Record<string, unknown>
  report?: Record<string, unknown>
  requirements?: unknown
  source?: string
  status: string
  summary?: string | null
  updatedAt?: string
  updated_at?: string
  userId?: string
  user_id?: string
  workspaceId?: string
  workspace_id?: string
}

export interface AgentWorkspaceRunTask {
  acceptanceNotes?: string | null
  acceptancePassed?: boolean
  acceptanceScore?: number
  acceptance_notes?: string | null
  acceptance_passed?: boolean
  acceptance_score?: number
  attempts?: number
  createdAt?: string
  created_at?: string
  executionNotes?: string | null
  execution_notes?: string | null
  id: string
  metadata?: Record<string, unknown>
  ownerRole?: string | null
  owner_role?: string | null
  runId?: string
  run_id?: string
  seq?: number
  status: string
  updatedAt?: string
  updated_at?: string
  userId?: string
  user_id?: string
  workspaceId?: string
  workspaceTaskId?: string | null
  workspace_id?: string
  workspace_task_id?: string | null
}

export interface AgentWorkspaceArtifact {
  artifactType?: string
  artifact_type?: string
  bucket?: string | null
  content?: string | null
  createdAt?: string
  created_at?: string
  fileName?: string | null
  fileSize?: number | null
  file_name?: string | null
  file_size?: number | null
  format?: string
  id: string
  metadata?: Record<string, unknown>
  mimeType?: string | null
  mime_type?: string | null
  objectKey?: string | null
  object_key?: string | null
  preview?: string | null
  runId?: string | null
  runTaskId?: string | null
  run_id?: string | null
  run_task_id?: string | null
  sha256?: string | null
  source?: string
  status: string
  storageKind?: string
  storage_kind?: string
  title: string
  updatedAt?: string
  updated_at?: string
  userId?: string
  user_id?: string
  validationReport?: Record<string, unknown>
  validationStatus?: string
  validation_report?: Record<string, unknown>
  validation_status?: string
  workspaceId?: string
  workspaceTaskId?: string | null
  workspace_id?: string
  workspace_task_id?: string | null
}

export interface AgentQualityEvaluation {
  learningRecords?: AgentLearningRecord[]
  learning_records?: AgentLearningRecord[]
  qualityCheck?: AgentQualityCheck
  quality_check?: AgentQualityCheck
  trustUpdate?: AgentTrustUpdate
  trust_update?: AgentTrustUpdate
}

export interface AgentQualityCheck {
  artifactId?: string | null
  artifact_id?: string | null
  createdAt?: string
  created_at?: string
  dimensions?: Record<string, unknown>
  id: string
  issues?: string[]
  metadata?: Record<string, unknown>
  passed: boolean
  role: string
  runId?: string
  run_id?: string
  score: number
  source?: string
  suggestions?: string[]
  userId?: string
  user_id?: string
  workspaceId?: string
  workspace_id?: string
}

export interface AgentTrustUpdate {
  delta: number
  newLevel?: string
  newScore?: number
  new_level?: string
  new_score?: number
  previousLevel?: string
  previousScore?: number
  previous_level?: string
  previous_score?: number
  reason?: string
  role: string
  skillId?: string | null
  skill_id?: string | null
}

export interface AgentLearningRecord {
  category: string
  confidence: number
  content: string
  createdAt?: string
  created_at?: string
  feedbackComment?: string | null
  feedbackRating?: number | null
  feedback_comment?: string | null
  feedback_rating?: number | null
  id: string
  metadata?: Record<string, unknown>
  qualityCheckId?: string | null
  quality_check_id?: string | null
  role: string
  runId?: string | null
  run_id?: string | null
  sourceAction?: string
  source_action?: string
  status?: string
  updatedAt?: string
  updated_at?: string
  userId?: string
  user_id?: string
  workspaceId?: string | null
  workspace_id?: string | null
}

export interface AgentRagContextLoaded {
  degraded?: boolean
  degradedReason?: string | null
  degraded_reason?: string | null
  externalSearchAvailable?: boolean
  external_search_available?: boolean
  knowledgeCount?: number
  knowledge_count?: number
  mode?: string
  productIds?: string[]
  product_ids?: string[]
  queryTerms?: string[]
  query_terms?: string[]
  sourceCount?: number
  source_count?: number
  sources?: AgentRagContextSource[]
  summary?: string | null
}

export interface AgentRagContextSource {
  id: string
  productId?: string | null
  product_id?: string | null
  sourceType?: string
  source_type?: string
  title: string
}

export interface AgentApprovalRequired {
  alert?: AgentAlert | null
  approval?: AgentToolApproval | null
  reason?: string | null
  requestId?: string | null
  request_id?: string | null
  riskLevel?: string | null
  risk_level?: string | null
  role?: string | null
  skillId?: string | null
  skill_id?: string | null
}

export interface AgentToolApproval {
  actionKey?: string | null
  action_key?: string | null
  id: string
  reason?: string | null
  requestRole?: string | null
  request_role?: string | null
  riskLevel?: string | null
  risk_level?: string | null
  skillName?: string
  skill_name?: string
  status: string
  toolCallId?: string | null
  tool_call_id?: string | null
}

export interface AgentAlert {
  alertType?: string
  alert_type?: string
  dedupKey?: string | null
  dedup_key?: string | null
  id: string
  message?: string
  role?: string | null
  severity?: string
  status?: string
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
  if (message.event === "done") {
    const event = readAgentRuntimeEvent(message.data)
    if (event) handlers.onAgentEvent?.(event)
    return {
      completed: true,
      fallbackOutput: "",
      finished: true,
      receivedDelta: false,
    }
  }

  if (message.event === "finished") {
    const event = readAgentRuntimeEvent(message.data)
    if (event) handlers.onAgentEvent?.(event)
    return {
      completed: false,
      fallbackOutput: "",
      finished: false,
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
  copyIfMissing(normalized, raw, "collaborationMode", "collaboration_mode")
  copyIfMissing(normalized, raw, "learningLevel", "learning_level")
  copyIfMissing(normalized, raw, "modelName", "model_name")
  copyIfMissing(normalized, raw, "primaryRuntimeRole", "primary_runtime_role")
  copyIfMissing(normalized, raw, "responseMode", "response_mode")
  copyIfMissing(normalized, raw, "routeMode", "route_mode")
  copyIfMissing(normalized, raw, "strategyId", "strategy_id")
  copyIfMissing(normalized, raw, "supportRuntimeRoles", "support_runtime_roles")
  copyIfMissing(normalized, raw, "primaryWorkerId", "primary_worker_id")
  copyIfMissing(normalized, raw, "supportWorkerIds", "support_worker_ids")
  copyIfMissing(normalized, raw, "productIds", "product_ids")
  copyIfMissing(normalized, raw, "contextRefs", "refs")
  copyIfMissing(normalized, raw, "workspaceId", "workspace_id")
  copyIfMissing(normalized, raw, "workspaceTaskId", "workspace_task_id")
  copyIfMissing(normalized, raw, "runId", "run_id")
  copyIfMissing(normalized, raw, "runTaskId", "run_task_id")
  copyIfMissing(normalized, raw, "ownerRole", "owner_role")
  copyIfMissing(normalized, raw, "acceptanceScore", "acceptance_score")
  copyIfMissing(normalized, raw, "acceptancePassed", "acceptance_passed")
  copyIfMissing(normalized, raw, "acceptanceNotes", "acceptance_notes")
  copyIfMissing(normalized, raw, "executionNotes", "execution_notes")
  copyIfMissing(normalized, raw, "artifactType", "artifact_type")
  copyIfMissing(normalized, raw, "storageKind", "storage_kind")
  copyIfMissing(normalized, raw, "objectKey", "object_key")
  copyIfMissing(normalized, raw, "fileName", "file_name")
  copyIfMissing(normalized, raw, "fileSize", "file_size")
  copyIfMissing(normalized, raw, "mimeType", "mime_type")
  copyIfMissing(normalized, raw, "validationStatus", "validation_status")
  copyIfMissing(normalized, raw, "validationReport", "validation_report")
  copyIfMissing(normalized, raw, "createdWorkspace", "created_workspace")
  copyIfMissing(normalized, raw, "qualityCheck", "quality_check")
  copyIfMissing(normalized, raw, "trustUpdate", "trust_update")
  copyIfMissing(normalized, raw, "learningRecords", "learning_records")
  copyIfMissing(normalized, raw, "queryTerms", "query_terms")
  copyIfMissing(normalized, raw, "knowledgeCount", "knowledge_count")
  copyIfMissing(normalized, raw, "sourceCount", "source_count")
  copyIfMissing(normalized, raw, "degradedReason", "degraded_reason")
  copyIfMissing(normalized, raw, "externalSearchAvailable", "external_search_available")
  copyIfMissing(normalized, raw, "skillId", "skill_id")
  copyIfMissing(normalized, raw, "requestId", "request_id")
  copyIfMissing(normalized, raw, "riskLevel", "risk_level")
  copyIfMissing(normalized, raw, "createdAt", "created_at")
  copyIfMissing(normalized, raw, "updatedAt", "updated_at")

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
