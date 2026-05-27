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
import { useAgentChatCampaignContext } from "./view-model/use-agent-chat-campaign-context"
import { useAgentChatProductContext } from "./view-model/use-agent-chat-product-context"
import { useAgentRuntimeOptions } from "./view-model/use-agent-runtime-options"
import { useAgentRunFeedback } from "./view-model/use-agent-run-feedback"
import { useAgentChatSend } from "./view-model/use-agent-chat-send"
import { useAgentChatSourceContext } from "./view-model/use-agent-chat-source-context"
import { useAgentChatSourceActions } from "./view-model/use-agent-chat-source-actions"
import { useAgentChatSourceLibrary } from "./view-model/use-agent-chat-source-library"

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
  const [strategyId, setStrategyId] = useState<AgentChatStrategyId>(DEFAULT_AGENT_CHAT_STRATEGY_ID)
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
  const isLoggedIn = Boolean(user)
  const productContext = useAgentChatProductContext({ enabled: isLoggedIn, isStreaming })
  const campaignContext = useAgentChatCampaignContext({ enabled: isLoggedIn, isStreaming })
  const sourceContext = useAgentChatSourceContext({ attachments, isStreaming })
  const { isLoadingSourceLibrary, sourceLibrary, sourceLibraryError } = useAgentChatSourceLibrary({ availableSources: sourceContext.availableSources, enabled: isLoggedIn })
  const playbookContext = useAgentChatPlaybooks({ enabled: isLoggedIn })
  const runtimeContext = useAgentRuntimeOptions({ enabled: isLoggedIn })
  const feedback = useAgentRunFeedback({
    isLoggedIn: Boolean(user),
    isStreaming,
    runtimeEvents,
  })
  const { actionState } = useAgentChatActionState({
    abortRef,
    clearSelectedCampaigns: campaignContext.clearSelectedCampaigns,
    clearSelectedProducts: productContext.clearSelectedProducts,
    clearSelectedSources: sourceContext.clearSelectedSources,
    fallbackConversationTitle,
    messageFormatCopy,
    resetRuntimeSelection: runtimeContext.resetSelection,
    restoreRuntimeSelection: runtimeContext.restoreSelection,
    restoreSelectedCampaigns: campaignContext.restoreSelectedCampaigns,
    restoreSelectedProducts: productContext.restoreSelectedProducts,
    restoreSelectedSources: sourceContext.restoreSelectedSources,
    sessionIdRef,
    setAttachments, setConversationId, setConversations, setError, setInput,
    setIsStreaming, setMessages, setRuntimeEvents, setStrategyId,
    tError: t("agentChat.error"),
    welcomeMessage: t("agentChat.welcomeMessage"),
  })
  const actionStateRef = useRef(actionState)
  const userBootstrapKey = user?.id ?? null

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
  const selectedStrategy = useMemo(() => getAgentChatStrategy(strategyId), [strategyId])
  const selectedRuntime = useMemo(() => runtimeSelectionForStrategy(runtimeContext.runtimeSelection), [runtimeContext.runtimeSelection])
  const commands = useAgentChatPageCommands({
    actionState,
    conversationId,
    isLoggedIn,
    isStreaming,
    setInput,
    setStrategyId,
  })
  const uploadAttachments = useAgentChatAttachmentUploads({
    conversationId,
    fallbackConversationTitle,
    isLoggedIn,
    isStreaming,
    messageFormatCopy,
    restoreSelectedSources: sourceContext.restoreSelectedSources,
    selectedSources: sourceContext.selectedSources,
    sessionIdRef,
    setAttachments, setConversationId, setConversations, setError,
    tError: t("agentChat.error"),
  })
  const sourceActions = useAgentChatSourceActions({
    conversationId,
    fallbackConversationTitle,
    isLoggedIn,
    isStreaming,
    restoreSelectedSources: sourceContext.restoreSelectedSources,
    selectedSources: sourceContext.selectedSources,
    sessionIdRef,
    setAttachments, setConversationId, setConversations, setError,
  })

  const sendMessage = useAgentChatSend({
    abortRef,
    attachments,
    conversationId,
    fallbackConversationTitle,
    inputValue: input,
    isLoggedIn,
    isStreaming,
    loginRequiredMessage,
    messageFormatCopy,
    runtimeSelection: selectedRuntime,
    selectedCampaigns: campaignContext.selectedCampaigns,
    selectedProducts: productContext.selectedProducts,
    selectedSources: sourceContext.selectedSources,
    sessionIdRef,
    setConversationId, setConversations, setError, setInput, setIsStreaming,
    setMessages, setRuntimeEvents,
    strategy: selectedStrategy,
    tError: t("agentChat.error"),
  })
  return createAgentChatPageViewModel({
    attachments,
    availableCampaigns: campaignContext.availableCampaigns,
    availableProducts: productContext.availableProducts,
    availableSources: sourceContext.availableSources,
    canSend,
    clearSelectedCampaigns: campaignContext.clearSelectedCampaigns,
    clearSelectedProducts: productContext.clearSelectedProducts,
    clearSelectedSources: sourceContext.clearSelectedSources,
    conversationId,
    conversations,
    commands,
    error,
    feedback,
    input,
    isLoggedIn,
    isLoadingCampaigns: campaignContext.isLoadingCampaigns,
    isLoadingPlaybooks: playbookContext.isLoadingPlaybooks,
    isLoadingProducts: productContext.isLoadingProducts,
    isLoadingRuntimeOptions: runtimeContext.isLoadingRuntimeOptions,
    isLoadingSourceLibrary,
    isStreaming,
    messages,
    playbooks: playbookContext.playbooks,
    runtimeOptions: runtimeContext.runtimeOptions,
    runtimeSelection: selectedRuntime,
    runtimeEvents,
    selectedProducts: productContext.selectedProducts,
    selectedCampaigns: campaignContext.selectedCampaigns,
    selectedStrategy,
    selectedSources: sourceContext.selectedSources,
    sendMessage,
    sourceLibrary,
    sourceLibraryError,
    setInput,
    setRuntimeCollaborationMode: runtimeContext.setCollaborationMode,
    setRuntimeLearningLevel: runtimeContext.setLearningLevel,
    setRuntimePrimaryRole: runtimeContext.setPrimaryRole,
    setRuntimeRoutingMode: runtimeContext.setRoutingMode,
    setStrategyId,
    strategyId,
    toggleProductContext: productContext.toggleProductContext,
    toggleCampaignContext: campaignContext.toggleCampaignContext,
    toggleRuntimeRole: runtimeContext.toggleRole,
    toggleSourceContext: sourceContext.toggleSourceContext,
    deleteSelectedSources: sourceActions.deleteSelectedSources,
    attachExistingSource: sourceActions.attachExistingSource,
    removeSourcesByObjects: sourceActions.removeSourcesByObjects,
    uploadAttachments,
  })
}
