const AGENT_STREAM_URL = "/api/sse/agent/session/stream"

export interface AgentStreamRequest {
  input: string
  agentId?: string
  sessionId?: string
  workspaceId?: string
}

export interface AgentStreamHandlers {
  onDelta: (content: string) => void
  onDone: () => void
  onError: (message: string) => void
  onEvent?: (event: AgentSessionEvent) => void
  signal?: AbortSignal
}

export interface AgentSessionEvent {
  event: string
  data: unknown
}

interface SseMessage {
  event: string
  data: string
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

  const parsedData = parseJson(message.data)

  if (message.event === "done") {
    handlers.onEvent?.({ event: message.event, data: parsedData })
    handlers.onDone()
    return true
  }

  if (message.event === "error") {
    handlers.onEvent?.({ event: message.event, data: parsedData })
    handlers.onError(readJsonField(parsedData, "message") || "Agent failed")
    return true
  }

  handlers.onEvent?.({ event: message.event, data: parsedData })

  if (message.event === "delta") {
    const content = readJsonField(parsedData, "content")
    if (content) handlers.onDelta(content)
  }

  return false
}

function parseJson(data: string): unknown {
  try {
    return JSON.parse(data)
  } catch {
    return data
  }
}

function readJsonField(data: unknown, field: string): string {
  if (!data || typeof data !== "object") return ""
  const value = (data as Record<string, unknown>)[field]
  return typeof value === "string" ? value : ""
}
