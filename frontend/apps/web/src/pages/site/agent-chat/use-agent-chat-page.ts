import { useEffect, useMemo, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useSelector } from "react-redux"
import type { RootState } from "@/store"
import type { AgentRuntimeEvent } from "./agent-chat-data"
import { createSessionId } from "./domain/agent-chat-message-mapping"
import { DEFAULT_AGENT_CHAT_STRATEGY_ID, getAgentChatStrategy, type AgentChatStrategyId } from "./domain/agent-chat-strategy"
import { runtimeSelectionForStrategy } from "./domain/agent-runtime-options"
import type { AgentChatAttachmentItem, AgentChatConversationItem, AgentChatMessage } from "./types"
import { loadLatestAgentConversation } from "./view-model/agent-chat-conversation-actions"
import { resetGuestAgentChatState } from "./view-model/agent-chat-page-actions"
import { useAgentChatAttachmentUploads } from "./view-model/use-agent-chat-attachment-uploads"
import { createAgentChatPageViewModel } from "./view-model/agent-chat-page-view-model"
import { useAgentChatActionState } from "./view-model/use-agent-chat-action-state"
import { useAgentChatPageCommands } from "./view-model/use-agent-chat-page-commands"
import { useAgentChatPlaybooks } from "./view-model/use-agent-chat-playbooks"
import { useAgentChatProductContext } from "./view-model/use-agent-chat-product-context"
import { useAgentRuntimeOptions } from "./view-model/use-agent-runtime-options"
import { useAgentRunFeedback } from "./view-model/use-agent-run-feedback"
import { useAgentChatSend } from "./view-model/use-agent-chat-send"
import { useAgentChatSourceContext } from "./view-model/use-agent-chat-source-context"

export function useAgentChatPage() {
  const { t } = useTranslation()
  const fallbackConversationTitle = t("agentChat.sidebar.newChat")
  const loginRequiredMessage = t("agentChat.loginRequired.message")
  const messageFormatCopy = useMemo(
    () => ({
      boundaryNotice: t("agentChat.boundaryNotice"),
      missingStructuredOutput: t("agentChat.missingStructuredOutput"),
      streamTimedOut: t("agentChat.streamTimedOut"),
    }),
    [t]
  )
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
  const [attachments, setAttachments] = useState<AgentChatAttachmentItem[]>([])
  const [isStreaming, setIsStreaming] = useState(false)
  const [runtimeEvents, setRuntimeEvents] = useState<AgentRuntimeEvent[]>([])
  const [error, setError] = useState<string | null>(null)
  const abortRef = useRef<AbortController | null>(null)
  const sessionIdRef = useRef(createSessionId())
  const productContext = useAgentChatProductContext({ enabled: Boolean(user), isStreaming })
  const sourceContext = useAgentChatSourceContext({ attachments, isStreaming })
  const playbookContext = useAgentChatPlaybooks({ enabled: Boolean(user) })
  const runtimeContext = useAgentRuntimeOptions({ enabled: Boolean(user) })
  const feedback = useAgentRunFeedback({
    isLoggedIn: Boolean(user),
    isStreaming,
    runtimeEvents,
  })
  const { actionState } = useAgentChatActionState({
    abortRef,
    clearSelectedProducts: productContext.clearSelectedProducts,
    clearSelectedSources: sourceContext.clearSelectedSources,
    fallbackConversationTitle,
    messageFormatCopy,
    resetRuntimeSelection: runtimeContext.resetSelection,
    restoreRuntimeSelection: runtimeContext.restoreSelection,
    restoreSelectedProducts: productContext.restoreSelectedProducts,
    restoreSelectedSources: sourceContext.restoreSelectedSources,
    sessionIdRef,
    setAttachments,
    setConversationId,
    setConversations,
    setError,
    setInput,
    setIsStreaming,
    setMessages,
    setRuntimeEvents,
    setStrategyId,
    tError: t("agentChat.error"),
    welcomeMessage: t("agentChat.welcomeMessage"),
  })
  const actionStateRef = useRef(actionState)
  const userBootstrapKey = user?.id ?? null

  useEffect(() => {
    const currentAbortRef = abortRef
    return () => currentAbortRef.current?.abort()
  }, [])

  useEffect(() => {
    actionStateRef.current = actionState
  }, [actionState])

  useEffect(() => {
    let cancelled = false
    const latestActionState = actionStateRef.current

    if (!userBootstrapKey) {
      resetGuestAgentChatState(latestActionState, () => cancelled)
    } else {
      void loadLatestAgentConversation(latestActionState, () => cancelled)
    }

    return () => {
      cancelled = true
    }
  }, [userBootstrapKey])

  const canSend = input.trim().length > 0 && !isStreaming
  const selectedStrategy = useMemo(
    () => getAgentChatStrategy(strategyId),
    [strategyId]
  )
  const selectedRuntime = useMemo(
    () => runtimeSelectionForStrategy(runtimeContext.runtimeSelection, strategyId),
    [runtimeContext.runtimeSelection, strategyId]
  )
  const commands = useAgentChatPageCommands({
    actionState,
    conversationId,
    isLoggedIn: Boolean(user),
    isStreaming,
    setInput,
    setStrategyId,
  })
  const uploadAttachments = useAgentChatAttachmentUploads({
    conversationId,
    fallbackConversationTitle,
    isLoggedIn: Boolean(user),
    isStreaming,
    messageFormatCopy,
    sessionIdRef,
    setAttachments,
    setConversationId,
    setConversations,
    setError,
    tError: t("agentChat.error"),
  })

  const sendMessage = useAgentChatSend({
    abortRef,
    conversationId,
    fallbackConversationTitle,
    inputValue: input,
    isLoggedIn: Boolean(user),
    isStreaming,
    loginRequiredMessage,
    messageFormatCopy,
    runtimeSelection: selectedRuntime,
    selectedProducts: productContext.selectedProducts,
    selectedSources: sourceContext.selectedSources,
    sessionIdRef,
    setConversationId,
    setConversations,
    setError,
    setInput,
    setIsStreaming,
    setMessages,
    setRuntimeEvents,
    strategy: selectedStrategy,
    tError: t("agentChat.error"),
  })
  return createAgentChatPageViewModel({
    attachments,
    availableProducts: productContext.availableProducts,
    availableSources: sourceContext.availableSources,
    canSend,
    clearSelectedProducts: productContext.clearSelectedProducts,
    clearSelectedSources: sourceContext.clearSelectedSources,
    conversationId,
    conversations,
    commands,
    error,
    feedback,
    input,
    isLoggedIn: Boolean(user),
    isLoadingPlaybooks: playbookContext.isLoadingPlaybooks,
    isLoadingProducts: productContext.isLoadingProducts,
    isLoadingRuntimeOptions: runtimeContext.isLoadingRuntimeOptions,
    isStreaming,
    messages,
    playbooks: playbookContext.playbooks,
    runtimeOptions: runtimeContext.runtimeOptions,
    runtimeSelection: selectedRuntime,
    runtimeEvents,
    selectedProducts: productContext.selectedProducts,
    selectedStrategy,
    selectedSources: sourceContext.selectedSources,
    sendMessage,
    setInput,
    setRuntimeCollaborationMode: runtimeContext.setCollaborationMode,
    setRuntimeLearningLevel: runtimeContext.setLearningLevel,
    setRuntimeModelName: runtimeContext.setModelName,
    setRuntimePrimaryRole: runtimeContext.setPrimaryRole,
    setStrategyId,
    strategyId,
    toggleProductContext: productContext.toggleProductContext,
    toggleRuntimeRole: runtimeContext.toggleRole,
    toggleSourceContext: sourceContext.toggleSourceContext,
    uploadAttachments,
  })
}
