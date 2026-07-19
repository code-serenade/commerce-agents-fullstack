import {
  createAgentChatPageViewModel,
  type AgentChatPageViewModelInput,
} from "./agent-chat-page-view-model"
import type { runtimeSelectionForStrategy } from "../domain/agent-runtime-options"
import type { getAgentChatStrategy } from "../domain/agent-chat-strategy"
import type { useAgentChatSessionState } from "./use-agent-chat-session-state"
import type { useAgentChatCampaignContext } from "./use-agent-chat-campaign-context"
import type { useAgentChatProductContext } from "./use-agent-chat-product-context"
import type { useAgentChatSourceContext } from "./use-agent-chat-source-context"
import type { useAgentChatSourceLibrary } from "./use-agent-chat-source-library"
import type { useAgentChatPlaybooks } from "./use-agent-chat-playbooks"
import type { useAgentRuntimeOptions } from "./use-agent-runtime-options"
import type { useAgentChatQuota } from "./use-agent-chat-quota"
import type { useAgentRunFeedback } from "./use-agent-run-feedback"
import type { useAgentChatContextPreview } from "./use-agent-chat-context-preview"
import type { useAgentChatPageCommands } from "./use-agent-chat-page-commands"
import type { useAgentChatStrategyRuntime } from "./use-agent-chat-strategy-runtime"
import type { useAgentChatSourceActions } from "./use-agent-chat-source-actions"
import type { useAgentChatAttachmentUploads } from "./use-agent-chat-attachment-uploads"
import type { useAgentChatSend } from "./use-agent-chat-send"

type SessionState = ReturnType<typeof useAgentChatSessionState>
type RuntimeSelection = ReturnType<typeof runtimeSelectionForStrategy>
type SelectedStrategy = ReturnType<typeof getAgentChatStrategy>

export function createAgentChatPageModel({
  campaignContext,
  canSend,
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
}: {
  campaignContext: ReturnType<typeof useAgentChatCampaignContext>
  canSend: boolean
  commands: ReturnType<typeof useAgentChatPageCommands>
  contextPreview: ReturnType<typeof useAgentChatContextPreview>
  feedback: ReturnType<typeof useAgentRunFeedback>
  playbookContext: ReturnType<typeof useAgentChatPlaybooks>
  productContext: ReturnType<typeof useAgentChatProductContext>
  quotaContext: ReturnType<typeof useAgentChatQuota>
  runtimeContext: ReturnType<typeof useAgentRuntimeOptions>
  selectedRuntime: RuntimeSelection
  selectedStrategy: SelectedStrategy
  sendMessage: ReturnType<typeof useAgentChatSend>
  sessionState: SessionState
  setStrategyAndRuntime: ReturnType<typeof useAgentChatStrategyRuntime>
  sourceActions: ReturnType<typeof useAgentChatSourceActions>
  sourceContext: ReturnType<typeof useAgentChatSourceContext>
  sourceLibraryContext: ReturnType<typeof useAgentChatSourceLibrary>
  uploadAttachments: ReturnType<typeof useAgentChatAttachmentUploads>
}) {
  const input: AgentChatPageViewModelInput = {
    attachments: sessionState.attachments,
    availableCampaigns: campaignContext.availableCampaigns,
    availableProducts: productContext.availableProducts,
    availableSources: sourceContext.availableSources,
    canSend,
    clearSelectedCampaigns: campaignContext.clearSelectedCampaigns,
    clearSelectedProducts: productContext.clearSelectedProducts,
    clearSelectedSources: sourceContext.clearSelectedSources,
    conversationId: sessionState.conversationId,
    conversations: sessionState.conversations,
    contextPreview,
    commands,
    error: sessionState.error,
    feedback,
    input: sessionState.input,
    isLoggedIn: sessionState.isLoggedIn,
    isLoadingCampaigns: campaignContext.isLoadingCampaigns,
    isLoadingPlaybooks: playbookContext.isLoadingPlaybooks,
    isLoadingProducts: productContext.isLoadingProducts,
    isLoadingQuota: quotaContext.isLoadingQuota,
    isLoadingRuntimeOptions: runtimeContext.isLoadingRuntimeOptions,
    isLoadingSourceLibrary: sourceLibraryContext.isLoadingSourceLibrary,
    isStreaming: sessionState.isStreaming,
    messages: sessionState.messages,
    playbooks: playbookContext.playbooks,
    quota: quotaContext.quota,
    runtimeEvents: sessionState.runtimeEvents,
    runtimeOptions: runtimeContext.runtimeOptions,
    runtimeSelection: selectedRuntime,
    selectedCampaigns: campaignContext.selectedCampaigns,
    selectedProducts: productContext.selectedProducts,
    selectedSources: sourceContext.selectedSources,
    selectedStrategy,
    sendMessage: () => quotaContext.refreshQuotaAfter(sendMessage),
    sourceLibrary: sourceLibraryContext.sourceLibrary,
    sourceLibraryError: sourceLibraryContext.sourceLibraryError,
    setInput: sessionState.setInput,
    setRuntimeCollaborationMode: runtimeContext.setCollaborationMode,
    setRuntimeLearningLevel: runtimeContext.setLearningLevel,
    setRuntimePrimaryRole: runtimeContext.setPrimaryRole,
    setRuntimeRoutingMode: runtimeContext.setRoutingMode,
    setStrategyId: setStrategyAndRuntime,
    strategyId: sessionState.strategyId,
    toggleCampaignContext: campaignContext.toggleCampaignContext,
    toggleProductContext: productContext.toggleProductContext,
    toggleRuntimeRole: runtimeContext.toggleRole,
    toggleSourceContext: sourceContext.toggleSourceContext,
    deleteSelectedSources: sourceActions.deleteSelectedSources,
    attachExistingSource: sourceActions.attachExistingSource,
    removeSourcesByObjects: sourceActions.removeSourcesByObjects,
    uploadAttachments,
  }

  return createAgentChatPageViewModel(input)
}
