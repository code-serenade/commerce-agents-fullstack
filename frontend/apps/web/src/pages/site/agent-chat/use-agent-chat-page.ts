import { useCallback, useEffect, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useSelector } from "react-redux"
import {
  addConversationMessageApi,
  createConversationApi,
  getConversationApi,
  listConversationsApi,
  streamAgentSessionApi,
  type AgentSessionEvent,
  type ConversationDetailResponse,
  type ConversationMessageResponse,
} from "@/api"
import type { RootState } from "@/store"
import type {
  AgentActivityItem,
  AgentChatConversationItem,
  AgentChatMessage,
} from "./types"

const AGENT_ID = "web-agent"
const WORKSPACE_ID = "workspace-001"

export function useAgentChatPage() {
  const { t } = useTranslation()
  const user = useSelector((state: RootState) => state.auth.user)
  const [input, setInput] = useState("")
  const [messages, setMessages] = useState<AgentChatMessage[]>(() => [
    {
      id: "welcome",
      role: "assistant",
      content: t("agentChat.welcomeMessage"),
    },
  ])
  const [conversationId, setConversationId] = useState<string | null>(null)
  const [conversations, setConversations] = useState<AgentChatConversationItem[]>([])
  const [isStreaming, setIsStreaming] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const abortRef = useRef<AbortController | null>(null)
  const sessionIdRef = useRef(createSessionId())

  const createWelcomeMessage = useCallback(
    (): AgentChatMessage => ({
      id: "welcome",
      role: "assistant",
      content: t("agentChat.welcomeMessage"),
    }),
    [t]
  )

  useEffect(() => {
    return () => abortRef.current?.abort()
  }, [])

  useEffect(() => {
    let cancelled = false

    if (!user) {
      queueMicrotask(() => {
        if (cancelled) return
        setConversationId(null)
        setConversations([])
        sessionIdRef.current = createSessionId()
        setMessages([createWelcomeMessage()])
      })
      return
    }

    async function loadLatestConversation() {
      try {
        const conversations = await listConversationsApi()
        if (cancelled) return

        setConversations(conversations.map(mapConversationItem))
        const latest = conversations[0]
        if (!latest) {
          if (!cancelled) {
            setConversationId(null)
            sessionIdRef.current = createSessionId()
            setMessages([createWelcomeMessage()])
          }
          return
        }

        const detail = await getConversationApi(latest.id)
        if (cancelled) return

        setConversationId(detail.conversation.id)
        sessionIdRef.current = detail.conversation.id
        setMessages(mapConversationMessages(detail, createWelcomeMessage()))
      } catch (err) {
        if (cancelled) return
        const message = err instanceof Error ? err.message : t("agentChat.error")
        setError(message)
      }
    }

    void loadLatestConversation()

    return () => {
      cancelled = true
    }
  }, [createWelcomeMessage, t, user])

  const canSend = Boolean(user) && input.trim().length > 0 && !isStreaming

  const startNewConversation = useCallback(() => {
    abortRef.current?.abort()
    abortRef.current = null
    setConversationId(null)
    sessionIdRef.current = createSessionId()
    setInput("")
    setError(null)
    setIsStreaming(false)
    setMessages([createWelcomeMessage()])
  }, [createWelcomeMessage])

  const selectConversation = useCallback(
    async (nextConversationId: string) => {
      if (!user || nextConversationId === conversationId || isStreaming) return

      setError(null)
      try {
        const detail = await getConversationApi(nextConversationId)
        setConversationId(detail.conversation.id)
        sessionIdRef.current = detail.conversation.id
        setMessages(mapConversationMessages(detail, createWelcomeMessage()))
      } catch (err) {
        const message = err instanceof Error ? err.message : t("agentChat.error")
        setError(message)
      }
    },
    [conversationId, createWelcomeMessage, isStreaming, t, user]
  )

  const sendMessage = useCallback(async () => {
    const content = input.trim()
    if (!content || !user || isStreaming) return

    const assistantId = createMessageId("assistant")
    setInput("")
    setError(null)
    setIsStreaming(true)
    setMessages((current) => [
      ...current,
      {
        id: createMessageId("user"),
        role: "user",
        content,
      },
      {
        id: assistantId,
        role: "assistant",
        content: "",
        status: "streaming",
      },
    ])

    const controller = new AbortController()
    abortRef.current = controller

    try {
      let activeConversationId = conversationId
      if (!activeConversationId) {
        const detail = await createConversationApi({
          title: createConversationTitle(content),
          firstMessage: content,
          agentId: AGENT_ID,
          modelName: null,
        })
        activeConversationId = detail.conversation.id
        setConversationId(activeConversationId)
        setConversations((current) => [
          mapConversationItem(detail.conversation),
          ...current.filter((item) => item.id !== detail.conversation.id),
        ])
        sessionIdRef.current = activeConversationId
      } else {
        await addConversationMessageApi(activeConversationId, {
          role: "user",
          contentText: content,
          contentJson: null,
        })
      }

      let assistantContent = ""
      let assistantActivities: AgentActivityItem[] = []
      let persistAssistant: Promise<void> | null = null

      await streamAgentSessionApi(
        {
          input: content,
          agentId: AGENT_ID,
          sessionId: sessionIdRef.current,
          workspaceId: WORKSPACE_ID,
        },
        {
          signal: controller.signal,
          onDelta: (chunk) => {
            assistantContent += chunk
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? { ...message, content: message.content + chunk }
                  : message
              )
            )
          },
          onEvent: (event) => {
            const activity = createActivityItem(event)
            if (!activity) return
            assistantActivities = [...assistantActivities, activity]
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? {
                      ...message,
                      activities: [...(message.activities ?? []), activity],
                    }
                  : message
              )
            )
          },
          onDone: () => {
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? { ...message, status: undefined }
                  : message
              )
            )
            setIsStreaming(false)
            abortRef.current = null
            if (assistantContent) {
              persistAssistant = addConversationMessageApi(activeConversationId, {
                role: "assistant",
                contentText: assistantContent,
                contentJson: createAssistantContentJson(assistantActivities),
              }).then(() => undefined)
            }
          },
          onError: (message) => {
            setError(message)
            setMessages((current) =>
              current.map((item) =>
                item.id === assistantId
                  ? {
                      ...item,
                      content: item.content || message,
                      status: "error",
                    }
                  : item
              )
            )
            setIsStreaming(false)
            abortRef.current = null
          },
        }
      )
      await persistAssistant
    } catch (err) {
      if (controller.signal.aborted) return
      const message = err instanceof Error ? err.message : t("agentChat.error")
      setError(message)
      setIsStreaming(false)
      abortRef.current = null
    }
  }, [conversationId, input, isStreaming, t, user])

  const stopStreaming = useCallback(() => {
    abortRef.current?.abort()
    abortRef.current = null
    setIsStreaming(false)
    setMessages((current) =>
      current.map((message) =>
        message.status === "streaming"
          ? { ...message, status: undefined }
          : message
      )
    )
  }, [])

  return {
    canSend,
    conversationId,
    conversations,
    error,
    input,
    isLoggedIn: Boolean(user),
    isStreaming,
    messages,
    sendMessage,
    selectConversation,
    setInput,
    startNewConversation,
    stopStreaming,
  }
}

function mapConversationItem(conversation: {
  id: string
  title: string | null
  updatedAt: string
}): AgentChatConversationItem {
  return {
    id: conversation.id,
    title: conversation.title || "New chat",
    updatedAt: conversation.updatedAt,
  }
}

function mapConversationMessages(
  detail: ConversationDetailResponse,
  fallback: AgentChatMessage
): AgentChatMessage[] {
  const messages = detail.messages
    .filter(isVisibleMessage)
    .map((message) => ({
      id: message.id,
      role: message.role,
      content: message.contentText ?? "",
      activities: readSavedActivities(message.contentJson),
    }))

  return messages.length > 0 ? messages : [fallback]
}

function isVisibleMessage(
  message: ConversationMessageResponse
): message is ConversationMessageResponse & { role: "user" | "assistant" } {
  return (
    (message.role === "user" || message.role === "assistant") &&
    Boolean(message.contentText)
  )
}

function createConversationTitle(content: string) {
  return content.length > 40 ? `${content.slice(0, 40)}...` : content
}

function createSessionId() {
  return `web-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`
}

function createMessageId(prefix: string) {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`
}

function createActivityItem(event: AgentSessionEvent): AgentActivityItem | null {
  if (event.event === "delta" || event.event === "llm_chunk") return null

  const data = isRecord(event.data) ? event.data : {}
  const title = getActivityTitle(event.event)
  const description = getActivityDescription(event.event, data)
  const status = getActivityStatus(event.event)

  return {
    id: createMessageId("activity"),
    event: event.event,
    kind: getActivityKind(event.event),
    title,
    description,
    workerId: readString(data, "worker_id") || readString(data, "primary_worker_id"),
    taskId: readString(data, "task_id"),
    nodeId: readString(data, "node_id"),
    graphId: readString(data, "graph_id"),
    status,
    createdAt: new Date().toISOString(),
    raw: event.data,
  }
}

function createAssistantContentJson(activities: AgentActivityItem[]) {
  return activities.length > 0 ? { agentActivities: activities } : null
}

function readSavedActivities(contentJson: unknown): AgentActivityItem[] | undefined {
  if (!isRecord(contentJson)) return undefined
  const activities = contentJson.agentActivities
  if (!Array.isArray(activities)) return undefined

  const normalized = activities.filter(isAgentActivityItem)
  return normalized.length > 0 ? normalized : undefined
}

function isAgentActivityItem(value: unknown): value is AgentActivityItem {
  return (
    isRecord(value) &&
    typeof value.id === "string" &&
    typeof value.event === "string" &&
    typeof value.kind === "string" &&
    typeof value.title === "string" &&
    typeof value.createdAt === "string"
  )
}

function getActivityTitle(event: string) {
  const titles: Record<string, string> = {
    started: "agentChat.activity.events.started",
    memory_loaded: "agentChat.activity.events.memoryLoaded",
    memory_persisted: "agentChat.activity.events.memoryPersisted",
    memory_compacted: "agentChat.activity.events.memoryCompacted",
    commander_thinking: "agentChat.activity.events.commanderThinking",
    worker_started: "agentChat.activity.events.workerStarted",
    worker_finished: "agentChat.activity.events.workerFinished",
    collaboration_started: "agentChat.activity.events.collaborationStarted",
    collaboration_worker_started: "agentChat.activity.events.collaborationWorkerStarted",
    collaboration_worker_finished: "agentChat.activity.events.collaborationWorkerFinished",
    collaboration_finished: "agentChat.activity.events.collaborationFinished",
    task_graph_planned: "agentChat.activity.events.taskGraphPlanned",
    task_node_ready: "agentChat.activity.events.taskNodeReady",
    task_node_started: "agentChat.activity.events.taskNodeStarted",
    task_node_reported: "agentChat.activity.events.taskNodeReported",
    task_node_evaluated: "agentChat.activity.events.taskNodeEvaluated",
    task_node_rework_requested: "agentChat.activity.events.taskNodeReworkRequested",
    task_node_passed: "agentChat.activity.events.taskNodePassed",
    task_node_failed: "agentChat.activity.events.taskNodeFailed",
    task_graph_finished: "agentChat.activity.events.taskGraphFinished",
    output: "agentChat.activity.events.output",
    failed: "agentChat.activity.events.failed",
    finished: "agentChat.activity.events.finished",
    error: "agentChat.activity.events.error",
    done: "agentChat.activity.events.done",
  }

  return titles[event] ?? "agentChat.activity.events.other"
}

function getActivityDescription(event: string, data: Record<string, unknown>) {
  if (event === "memory_loaded") {
    const count = readNumber(data, "count")
    const kinds = Array.isArray(data.kinds) ? data.kinds.join(", ") : ""
    return count ? `${count}${kinds ? ` / ${kinds}` : ""}` : kinds
  }

  if (event === "memory_persisted") {
    const count = readNumber(data, "count")
    return count ? `${count}` : undefined
  }

  if (event === "memory_compacted") {
    const beforeChars = readNumber(data, "before_chars")
    const afterChars = readNumber(data, "after_chars")
    return beforeChars && afterChars ? `${beforeChars} -> ${afterChars}` : undefined
  }

  if (event === "collaboration_started") {
    const primary = readString(data, "primary_worker_id")
    const support = Array.isArray(data.support_worker_ids)
      ? data.support_worker_ids.join(", ")
      : ""
    return [primary, support].filter(Boolean).join(" / ")
  }

  if (event === "task_node_started") {
    const attempt = readNumber(data, "attempt")
    return attempt ? `attempt ${attempt}` : undefined
  }

  if (event === "task_node_rework_requested") {
    return readString(data, "instruction")
  }

  if (event === "task_node_failed" || event === "failed") {
    return readString(data, "reason")
  }

  if (event === "collaboration_worker_finished") {
    return readString(data, "content")
  }

  if (event === "task_node_reported") {
    return readReportSummary(data.report)
  }

  if (event === "task_node_evaluated") {
    return readEvaluationSummary(data.evaluation)
  }

  if (event === "task_graph_planned") {
    return readTaskGraphSummary(data.graph)
  }

  return undefined
}

function readReportSummary(report: unknown) {
  if (!isRecord(report)) return undefined
  return (
    readString(report, "summary") ||
    readString(report, "content") ||
    readString(report, "message")
  )
}

function readEvaluationSummary(evaluation: unknown) {
  if (!isRecord(evaluation)) return undefined
  return (
    readString(evaluation, "summary") ||
    readString(evaluation, "feedback") ||
    readString(evaluation, "reason")
  )
}

function readTaskGraphSummary(graph: unknown) {
  if (!isRecord(graph)) return undefined
  const nodes = Array.isArray(graph.nodes) ? graph.nodes.length : 0
  return nodes > 0 ? `${nodes} nodes` : readString(graph, "id")
}

function getActivityKind(event: string): AgentActivityItem["kind"] {
  if (event.startsWith("memory_")) return "memory"
  if (event.startsWith("worker_")) return "worker"
  if (event.startsWith("collaboration_")) return "collaboration"
  if (event.startsWith("task_graph_")) return "task_graph"
  if (event.startsWith("task_node_")) return "task_node"
  if (event === "commander_thinking") return "commander"
  if (event === "output" || event === "finished" || event === "done") return "output"
  if (event === "failed" || event === "error") return "error"
  if (event === "started") return "session"
  return "other"
}

function getActivityStatus(event: string): AgentActivityItem["status"] {
  if (event.includes("failed") || event === "error" || event === "failed") return "error"
  if (event.includes("finished") || event.includes("passed") || event === "done") {
    return "success"
  }
  if (event.includes("started") || event === "commander_thinking") return "running"
  return "info"
}

function readString(data: Record<string, unknown>, key: string) {
  const value = data[key]
  return typeof value === "string" ? value : ""
}

function readNumber(data: Record<string, unknown>, key: string) {
  const value = data[key]
  return typeof value === "number" ? value : 0
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null
}
