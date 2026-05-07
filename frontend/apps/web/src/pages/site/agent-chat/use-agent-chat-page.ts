import { useCallback, useEffect, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useSelector } from "react-redux"
import {
  addConversationMessageApi,
  createConversationApi,
  getConversationApi,
  listConversationsApi,
  streamAgentSessionApi,
  type AgentRuntimeEvent,
  type ConversationDetailResponse,
  type ConversationMessageResponse,
} from "@/api"
import type { RootState } from "@/store"
import type { AgentChatConversationItem, AgentChatMessage } from "./types"

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
  const [runtimeEvents, setRuntimeEvents] = useState<AgentRuntimeEvent[]>([])
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
        const message = formatAgentUiMessage(
          err instanceof Error ? err.message : t("agentChat.error")
        )
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
    setRuntimeEvents([])
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
        const message = formatAgentUiMessage(
          err instanceof Error ? err.message : t("agentChat.error")
        )
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
    setRuntimeEvents([])
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
          onAgentEvent: (event) => {
            setRuntimeEvents((current) => [...current, event])
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
                contentJson: null,
              }).then(() => undefined)
            }
          },
          onError: (message) => {
            const displayMessage = formatAgentUiMessage(message)
            setError(displayMessage)
            setMessages((current) =>
              current.map((item) =>
                item.id === assistantId
                  ? {
                      ...item,
                      content: item.content || displayMessage,
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
      const message = formatAgentUiMessage(
        err instanceof Error ? err.message : t("agentChat.error")
      )
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
    runtimeEvents,
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
      content: formatAgentUiMessage(message.contentText ?? ""),
    }))

  return messages.length > 0 ? messages : [fallback]
}

function isVisibleMessage(
  message: ConversationMessageResponse
): message is ConversationMessageResponse & { role: "user" | "assistant" } {
  const content = formatAgentUiMessage(message.contentText ?? "")
  return (
    (message.role === "user" || message.role === "assistant") &&
    Boolean(content)
  )
}

function formatAgentUiMessage(content: string) {
  const text = content.trim()
  if (!text || text === "missing") return ""
  if (/structured role output is missing/i.test(text)) {
    return "上一轮任务未产出完整结构化结果。可以直接重新发送任务，我会重新规划执行计划，并把过程同步到经营工作区。"
  }
  return text
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
