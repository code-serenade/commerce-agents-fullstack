import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useSelector } from "react-redux"
import type { RootState } from "@/store"
import type { AgentRuntimeEvent } from "./agent-chat-data"
import {
  createSessionId,
} from "./domain/agent-chat-message-mapping"
import type { AgentChatConversationItem, AgentChatMessage } from "./types"
import {
  deleteAgentConversation,
  loadLatestAgentConversation,
  renameAgentConversation,
  selectAgentConversation,
} from "./view-model/agent-chat-conversation-actions"
import { sendAgentChatMessage } from "./view-model/agent-chat-send-message"

export function useAgentChatPage() {
  const { t } = useTranslation()
  const fallbackConversationTitle = t("agentChat.sidebar.newChat")
  const missingStructuredOutput = t("agentChat.missingStructuredOutput")
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
  const actionState = useMemo(
    () => ({
      createWelcomeMessage,
      fallbackConversationTitle,
      missingStructuredOutput,
      sessionIdRef,
      setConversationId,
      setConversations,
      setError,
      setInput,
      setMessages,
      setRuntimeEvents,
      tError: t("agentChat.error"),
    }),
    [createWelcomeMessage, fallbackConversationTitle, missingStructuredOutput, t]
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

    void loadLatestAgentConversation(actionState, () => cancelled)

    return () => {
      cancelled = true
    }
  }, [
    createWelcomeMessage,
    actionState,
    user,
  ])

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
      await selectAgentConversation(nextConversationId, actionState)
    },
    [actionState, conversationId, isStreaming, user]
  )

  const renameConversation = useCallback(
    async (targetConversationId: string, title: string) => {
      if (!user || isStreaming) return
      await renameAgentConversation(targetConversationId, title, actionState)
    },
    [actionState, isStreaming, user]
  )

  const deleteConversation = useCallback(
    async (targetConversationId: string) => {
      if (!user || isStreaming) return
      await deleteAgentConversation(targetConversationId, conversationId, actionState)
    },
    [actionState, conversationId, isStreaming, user]
  )

  const sendMessage = useCallback(async () => {
    const content = input.trim()
    if (!content || !user || isStreaming) return

    const controller = await sendAgentChatMessage({
      content,
      conversationId,
      fallbackConversationTitle,
      missingStructuredOutput,
      onController: (controller) => {
        abortRef.current = controller
      },
      sessionIdRef,
      setConversationId,
      setConversations,
      setError,
      setInput,
      setIsStreaming,
      setMessages,
      setRuntimeEvents,
      tError: t("agentChat.error"),
    })

    if (abortRef.current === controller) {
      abortRef.current = null
    }
  }, [
    conversationId,
    fallbackConversationTitle,
    input,
    isStreaming,
    missingStructuredOutput,
    t,
    user,
  ])

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
    deleteConversation,
    error,
    input,
    isLoggedIn: Boolean(user),
    isStreaming,
    messages,
    runtimeEvents,
    renameConversation,
    sendMessage,
    selectConversation,
    setInput,
    startNewConversation,
    stopStreaming,
  }
}
