import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import { useLocation, useNavigate, useParams } from "react-router"
import { getAgentChatStrategy } from "./domain/agent-chat-strategy"
import { runtimeSelectionForStrategy } from "./domain/agent-runtime-options"
import { createAgentChatPageModel } from "./view-model/agent-chat-page-model-composer"
import { useAgentChatActionState } from "./view-model/use-agent-chat-action-state"
import { useAgentChatBootstrap, useAgentChatRouteActions } from "./view-model/use-agent-chat-bootstrap"
import { useAgentChatAttachmentUploads } from "./view-model/use-agent-chat-attachment-uploads"
import { useAgentChatCampaignContext } from "./view-model/use-agent-chat-campaign-context"
import { useAgentChatContextPreview } from "./view-model/use-agent-chat-context-preview"
import { useAgentChatConversationIdentity } from "./view-model/use-agent-chat-conversation-identity"
import { useAgentChatPageCommands } from "./view-model/use-agent-chat-page-commands"
import { useAgentChatPlaybooks } from "./view-model/use-agent-chat-playbooks"
import { useAgentChatProductContext } from "./view-model/use-agent-chat-product-context"
import { useAgentChatQuota } from "./view-model/use-agent-chat-quota"
import { useAgentChatSend } from "./view-model/use-agent-chat-send"
import { useAgentChatSessionState } from "./view-model/use-agent-chat-session-state"
import { useAgentChatSourceActions } from "./view-model/use-agent-chat-source-actions"
import { useAgentChatSourceContext } from "./view-model/use-agent-chat-source-context"
import { useAgentChatSourceLibrary } from "./view-model/use-agent-chat-source-library"
import { useAgentChatStrategyRuntime } from "./view-model/use-agent-chat-strategy-runtime"
import { useAgentRunFeedback } from "./view-model/use-agent-run-feedback"
import { useAgentRuntimeOptions } from "./view-model/use-agent-runtime-options"
import { isAgentChatPath } from "./view-model/agent-chat-route-path"

export function useAgentChatPage() {
  const { t } = useTranslation()
  const location = useLocation()
  const navigate = useNavigate()
  const { conversationId: routeConversationId } = useParams<{ conversationId?: string }>()
  const sessionState = useAgentChatSessionState(t("agentChat.welcomeMessage"))
  const { isLoggedIn, isStreaming } = sessionState
  const fallbackConversationTitle = t("agentChat.sidebar.newChat")
  const loginRequiredMessage = t("agentChat.loginRequired.message")
  const messageFormatCopy = useMemo(
    () => ({
      boundaryNotice: t("agentChat.boundaryNotice"),
      missingStructuredOutput: t("agentChat.missingStructuredOutput"),
      streamTimedOut: t("agentChat.streamTimedOut"),
      teamUsageDisabled: t("agentChat.quota.disabledHint"),
      teamUsageQuotaExceeded: t("agentChat.quota.exhaustedHint"),
    }),
    [t]
  )
  const isAgentChatRoute = isAgentChatPath(location.pathname)
  const productContext = useAgentChatProductContext({
    enabled: isLoggedIn,
    isStreaming,
  })
  const campaignContext = useAgentChatCampaignContext({
    enabled: isLoggedIn,
    isStreaming,
  })
  const sourceContext = useAgentChatSourceContext({
    attachments: sessionState.attachments,
    isStreaming,
  })
  const sourceLibraryContext = useAgentChatSourceLibrary({
    availableSources: sourceContext.availableSources,
    enabled: isLoggedIn,
  })
  const playbookContext = useAgentChatPlaybooks({ enabled: isLoggedIn })
  const runtimeContext = useAgentRuntimeOptions({ enabled: isLoggedIn })
  const quotaContext = useAgentChatQuota({ enabled: isLoggedIn })
  const feedback = useAgentRunFeedback({
    isLoggedIn,
    isStreaming,
    runtimeEvents: sessionState.runtimeEvents,
  })
  const { actionState } = useAgentChatActionState({
    abortRef: sessionState.abortRef,
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
    sessionIdRef: sessionState.sessionIdRef,
    setAttachments: sessionState.setAttachments,
    setConversationId: sessionState.setActiveConversationId,
    setConversations: sessionState.setConversations,
    setError: sessionState.setError,
    setInput: sessionState.setInput,
    setIsStreaming: sessionState.setIsStreaming,
    setMessages: sessionState.setMessages,
    setRuntimeEvents: sessionState.setRuntimeEvents,
    setStrategyId: sessionState.setStrategyId,
    tError: t("agentChat.error"),
    welcomeMessage: t("agentChat.welcomeMessage"),
  })
  useAgentChatBootstrap({
    actionState,
    conversationId: sessionState.conversationId,
    conversationIdRef: sessionState.conversationIdRef,
    isAgentChatRoute,
    routeConversationId,
    tError: t("agentChat.error"),
    userBootstrapKey: sessionState.user?.id ?? null,
  })
  const { replaceWithNewConversationRoute, syncConversationRoute } =
    useAgentChatRouteActions({ isAgentChatRoute, navigate })
  const { ensureConversation, promoteConversationTitle } =
    useAgentChatConversationIdentity({
      conversationIdRef: sessionState.conversationIdRef,
      fallbackConversationTitle,
      onConversationCreated: syncConversationRoute,
      setConversationId: sessionState.setActiveConversationId,
      setConversations: sessionState.setConversations,
    })
  const selectedStrategy = useMemo(() => getAgentChatStrategy(sessionState.strategyId), [sessionState.strategyId])
  const selectedRuntime = useMemo(
    () => runtimeSelectionForStrategy(runtimeContext.runtimeSelection),
    [runtimeContext.runtimeSelection]
  )
  const activeConversationTitle = sessionState.conversations.find((item) => item.id === sessionState.conversationId)?.title ?? null
  const setStrategyAndRuntime = useAgentChatStrategyRuntime({
    runtimeSelection: runtimeContext.runtimeSelection,
    setRoutingMode: runtimeContext.setRoutingMode,
    setStrategyId: sessionState.setStrategyId,
  })
  const commands = useAgentChatPageCommands({
    actionState,
    conversationId: sessionState.conversationId,
    isLoggedIn,
    isStreaming,
    onActiveConversationDeleted: replaceWithNewConversationRoute,
    setInput: sessionState.setInput,
    setStrategyId: setStrategyAndRuntime,
  })
  const uploadAttachments = useAgentChatAttachmentUploads({
    conversationId: sessionState.conversationId,
    ensureConversation,
    fallbackConversationTitle,
    isLoggedIn,
    isStreaming,
    messageFormatCopy,
    restoreSelectedSources: sourceContext.restoreSelectedSources,
    selectedSources: sourceContext.selectedSources,
    sessionIdRef: sessionState.sessionIdRef,
    setAttachments: sessionState.setAttachments,
    setError: sessionState.setError,
    tError: t("agentChat.error"),
  })
  const sourceActions = useAgentChatSourceActions({
    conversationId: sessionState.conversationId,
    ensureConversation,
    fallbackConversationTitle,
    isLoggedIn,
    isStreaming,
    restoreSelectedSources: sourceContext.restoreSelectedSources,
    selectedSources: sourceContext.selectedSources,
    sessionIdRef: sessionState.sessionIdRef,
    setAttachments: sessionState.setAttachments,
    setError: sessionState.setError,
  })
  const contextPreview = useAgentChatContextPreview({
    conversationId: sessionState.conversationId,
    inputValue: sessionState.input,
    isLoggedIn,
    isStreaming,
    selectedCampaigns: campaignContext.selectedCampaigns,
    selectedProducts: productContext.selectedProducts,
    selectedSources: sourceContext.selectedSources,
    tError: t("agentChat.composer.contextPreview.error"),
  })
  const sendMessage = useAgentChatSend({
    abortRef: sessionState.abortRef,
    attachments: sessionState.attachments,
    conversationId: sessionState.conversationId,
    conversationTitle: activeConversationTitle,
    fallbackConversationTitle,
    inputValue: sessionState.input,
    isLoggedIn,
    isStreaming,
    loginRequiredMessage,
    messageFormatCopy,
    ensureConversation,
    promoteConversationTitle,
    runtimeSelection: selectedRuntime,
    selectedCampaigns: campaignContext.selectedCampaigns,
    selectedProducts: productContext.selectedProducts,
    selectedSources: sourceContext.selectedSources,
    sessionIdRef: sessionState.sessionIdRef,
    setError: sessionState.setError,
    setInput: sessionState.setInput,
    setIsStreaming: sessionState.setIsStreaming,
    setMessages: sessionState.setMessages,
    setRuntimeEvents: sessionState.setRuntimeEvents,
    strategy: selectedStrategy,
    tError: t("agentChat.error"),
  })

  return createAgentChatPageModel({
    campaignContext,
    canSend: sessionState.input.trim().length > 0 && !isStreaming,
    commands,
    contextPreview,
    feedback,
    playbookContext,
    productContext,
    quotaContext,
    runtimeContext,
    selectedRuntime,
    selectedStrategy,
    sendMessage,
    sessionState,
    setStrategyAndRuntime,
    sourceActions,
    sourceContext,
    sourceLibraryContext,
    uploadAttachments,
  })
}
