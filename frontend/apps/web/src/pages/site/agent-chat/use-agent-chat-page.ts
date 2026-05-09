import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useSelector } from "react-redux"
import type { RootState } from "@/store"
import type { AgentRuntimeEvent } from "./agent-chat-data"
import { createSessionId } from "./domain/agent-chat-message-mapping"
import {
  DEFAULT_AGENT_CHAT_STRATEGY_ID,
  getAgentChatStrategy,
  type AgentChatStrategyId,
} from "./domain/agent-chat-strategy"
import type { AgentChatConversationItem, AgentChatMessage } from "./types"
import { loadLatestAgentConversation } from "./view-model/agent-chat-conversation-actions"
import {
  resetGuestAgentChatState,
} from "./view-model/agent-chat-page-actions"
import { sendAgentChatMessage } from "./view-model/agent-chat-send-message"
import { useAgentChatPageCommands } from "./view-model/use-agent-chat-page-commands"
import { useAgentChatPlaybooks } from "./view-model/use-agent-chat-playbooks"
import { useAgentChatProductContext } from "./view-model/use-agent-chat-product-context"
import { useAgentRunFeedback } from "./view-model/use-agent-run-feedback"

export function useAgentChatPage() {
  const { t } = useTranslation()
  const fallbackConversationTitle = t("agentChat.sidebar.newChat")
  const loginRequiredMessage = t("agentChat.loginRequired.message")
  const missingStructuredOutput = t("agentChat.missingStructuredOutput")
  const user = useSelector((state: RootState) => state.auth.user)
  const [input, setInput] = useState("")
  const [strategyId, setStrategyId] = useState<AgentChatStrategyId>(
    DEFAULT_AGENT_CHAT_STRATEGY_ID
  )
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
  const productContext = useAgentChatProductContext({
    enabled: Boolean(user),
    isStreaming,
  })
  const playbookContext = useAgentChatPlaybooks({
    enabled: Boolean(user),
  })
  const feedback = useAgentRunFeedback({
    isLoggedIn: Boolean(user),
    isStreaming,
    runtimeEvents,
  })

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
      abortRef,
      createWelcomeMessage,
      fallbackConversationTitle,
      missingStructuredOutput,
      sessionIdRef,
      setConversationId,
      setConversations,
      setError,
      setInput,
      setIsStreaming,
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
      resetGuestAgentChatState(actionState, () => cancelled)
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

  const canSend = input.trim().length > 0 && !isStreaming
  const selectedStrategy = useMemo(
    () => getAgentChatStrategy(strategyId),
    [strategyId]
  )
  const commands = useAgentChatPageCommands({
    actionState,
    conversationId,
    isLoggedIn: Boolean(user),
    isStreaming,
    setInput,
    setStrategyId,
  })

  const sendMessage = useCallback(async () => {
    const content = input.trim()
    if (!content || isStreaming) return
    if (!user) {
      setError(loginRequiredMessage)
      return
    }

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
      selectedProducts: productContext.selectedProducts,
      strategy: selectedStrategy,
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
    loginRequiredMessage,
    missingStructuredOutput,
    productContext.selectedProducts,
    selectedStrategy,
    t,
    user,
  ])

  return {
    availableProducts: productContext.availableProducts,
    canSend,
    conversationId,
    conversations,
    deleteConversation: commands.deleteConversation,
    error,
    feedback,
    input,
    isLoggedIn: Boolean(user),
    isLoadingPlaybooks: playbookContext.isLoadingPlaybooks,
    isLoadingProducts: productContext.isLoadingProducts,
    isStreaming,
    messages,
    playbooks: playbookContext.playbooks,
    runtimeEvents,
    selectPlaybook: commands.selectPlaybook,
    selectedProducts: productContext.selectedProducts,
    renameConversation: commands.renameConversation,
    sendMessage,
    selectConversation: commands.selectConversation,
    selectedStrategy,
    setInput,
    setStrategyId,
    startNewConversation: commands.startNewConversation,
    stopStreaming: commands.stopStreaming,
    strategyId,
    toggleProductContext: productContext.toggleProductContext,
  }
}
