import type { AgentRuntimeEvent } from "../agent-chat-data"
import type { CurrentTeamUsageQuotaResponse } from "@/api"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
  AgentChatMessage,
} from "../types"
import type {
  AgentChatCampaignContextSelection,
  AgentChatProductContextSelection,
  AgentChatSourceContextSelection,
  AgentChatSourceLibraryItem,
} from "../domain/agent-chat-context-selection"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type {
  AgentChatStrategy,
  AgentChatStrategyId,
} from "../domain/agent-chat-strategy"
import type { AgentRunFeedbackState } from "./use-agent-run-feedback"
import type { AgentChatContextPreviewState } from "./use-agent-chat-context-preview"

export interface AgentChatPageViewModelInput {
  attachments: AgentChatAttachmentItem[]
  availableCampaigns: AgentChatCampaignContextSelection[]
  availableProducts: AgentChatProductContextSelection[]
  availableSources: AgentChatSourceContextSelection[]
  canSend: boolean
  conversationId: string | null
  conversations: AgentChatConversationItem[]
  contextPreview: AgentChatContextPreviewState
  commands: {
    deleteConversation: (targetConversationId: string) => Promise<void>
    renameConversation: (
      targetConversationId: string,
      title: string
    ) => Promise<void>
    selectConversation: (nextConversationId: string) => Promise<void>
    selectPlaybook: (playbook: AgentChatPlaybookSelection) => void
    startNewConversation: () => void
    stopStreaming: () => void
  }
  error: string | null
  feedback: AgentRunFeedbackState
  input: string
  isLoggedIn: boolean
  isLoadingCampaigns: boolean
  isLoadingPlaybooks: boolean
  isLoadingProducts: boolean
  isLoadingQuota: boolean
  isLoadingRuntimeOptions: boolean
  isLoadingSourceLibrary: boolean
  isStreaming: boolean
  messages: AgentChatMessage[]
  playbooks: AgentChatPlaybookSelection[]
  runtimeEvents: AgentRuntimeEvent[]
  quota: CurrentTeamUsageQuotaResponse | null
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  selectedCampaigns: AgentChatCampaignContextSelection[]
  selectedProducts: AgentChatProductContextSelection[]
  selectedSources: AgentChatSourceContextSelection[]
  selectedStrategy: AgentChatStrategy
  sendMessage: () => Promise<void>
  sourceLibrary: AgentChatSourceLibraryItem[]
  sourceLibraryError: string | null
  setInput: (value: string) => void
  setRuntimeCollaborationMode: (value: string) => void
  setRuntimeLearningLevel: (value: string) => void
  setRuntimePrimaryRole: (value: string) => void
  setRuntimeRoutingMode: (value: string) => void
  setStrategyId: (value: AgentChatStrategyId) => void
  strategyId: AgentChatStrategyId
  toggleProductContext: (product: AgentChatProductContextSelection) => void
  toggleCampaignContext: (campaign: AgentChatCampaignContextSelection) => void
  toggleRuntimeRole: (value: string) => void
  toggleSourceContext: (source: AgentChatSourceContextSelection) => void
  deleteSelectedSources: () => Promise<void>
  attachExistingSource: (fileId: string) => Promise<void>
  removeSourcesByObjects: (objects: { bucket: string; objectKey: string }[]) => void
  clearSelectedCampaigns: () => void
  clearSelectedProducts: () => void
  clearSelectedSources: () => void
  uploadAttachments: (files: File[]) => Promise<void>
}

export function createAgentChatPageViewModel(input: AgentChatPageViewModelInput) {
  return {
    attachments: input.attachments,
    availableCampaigns: input.availableCampaigns,
    availableProducts: input.availableProducts,
    availableSources: input.availableSources,
    canSend: input.canSend,
    clearSelectedCampaigns: input.clearSelectedCampaigns,
    clearSelectedProducts: input.clearSelectedProducts,
    clearSelectedSources: input.clearSelectedSources,
    conversationId: input.conversationId,
    conversations: input.conversations,
    contextPreview: input.contextPreview,
    deleteConversation: input.commands.deleteConversation,
    deleteSelectedSources: input.deleteSelectedSources,
    error: input.error,
    feedback: input.feedback,
    input: input.input,
    isLoggedIn: input.isLoggedIn,
    isLoadingCampaigns: input.isLoadingCampaigns,
    isLoadingPlaybooks: input.isLoadingPlaybooks,
    isLoadingProducts: input.isLoadingProducts,
    isLoadingQuota: input.isLoadingQuota,
    isLoadingRuntimeOptions: input.isLoadingRuntimeOptions,
    isLoadingSourceLibrary: input.isLoadingSourceLibrary,
    isStreaming: input.isStreaming,
    messages: input.messages,
    playbooks: input.playbooks,
    renameConversation: input.commands.renameConversation,
    quota: input.quota,
    runtimeEvents: input.runtimeEvents,
    runtimeOptions: input.runtimeOptions,
    runtimeSelection: input.runtimeSelection,
    selectConversation: input.commands.selectConversation,
    selectPlaybook: input.commands.selectPlaybook,
    selectedProducts: input.selectedProducts,
    selectedCampaigns: input.selectedCampaigns,
    selectedSources: input.selectedSources,
    selectedStrategy: input.selectedStrategy,
    sendMessage: input.sendMessage,
    sourceLibrary: input.sourceLibrary,
    sourceLibraryError: input.sourceLibraryError,
    setInput: input.setInput,
    setRuntimeCollaborationMode: input.setRuntimeCollaborationMode,
    setRuntimeLearningLevel: input.setRuntimeLearningLevel,
    setRuntimePrimaryRole: input.setRuntimePrimaryRole,
    setRuntimeRoutingMode: input.setRuntimeRoutingMode,
    setStrategyId: input.setStrategyId,
    startNewConversation: input.commands.startNewConversation,
    stopStreaming: input.commands.stopStreaming,
    strategyId: input.strategyId,
    toggleProductContext: input.toggleProductContext,
    toggleCampaignContext: input.toggleCampaignContext,
    toggleRuntimeRole: input.toggleRuntimeRole,
    toggleSourceContext: input.toggleSourceContext,
    attachExistingSource: input.attachExistingSource,
    removeSourcesByObjects: input.removeSourcesByObjects,
    uploadAttachments: input.uploadAttachments,
  }
}
