import type {
  AgentChatCampaignContextSelection,
  AgentChatProductContextSelection,
  AgentChatSourceContextSelection,
  AgentChatSourceLibraryItem,
} from "../domain/agent-chat-context-selection"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatAttachmentItem } from "../types"
import type { AgentChatContextPreviewState } from "../view-model/use-agent-chat-context-preview"
import type { AgentChatComposerPanelId } from "../view-model/use-agent-chat-panel-state"
import type { ReactNode } from "react"

export interface AgentChatComposerProps {
  attachments: AgentChatAttachmentItem[]
  availableProducts: AgentChatProductContextSelection[]
  availableCampaigns: AgentChatCampaignContextSelection[]
  availableSources: AgentChatSourceContextSelection[]
  canSend: boolean
  children?: ReactNode
  contextPreview: AgentChatContextPreviewState
  error: string | null
  focusVersion?: number
  heading?: ReactNode
  requestedPanel?: AgentChatComposerPanelId | null
  requestedPanelVersion?: number
  input: string
  isLoggedIn: boolean
  isLoadingPlaybooks: boolean
  isLoadingCampaigns: boolean
  isLoadingProducts: boolean
  isLoadingRuntimeOptions: boolean
  isLoadingSourceLibrary: boolean
  isStreaming: boolean
  placement?: "bottom" | "welcome"
  placeholder?: string
  onInputChange: (value: string) => void
  onRuntimeCollaborationModeChange: (value: string) => void
  onRuntimeLearningLevelChange: (value: string) => void
  onRuntimePrimaryRoleChange: (value: string) => void
  onRuntimeRoutingModeChange: (value: string) => void
  onToggleRuntimeRole: (value: string) => void
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  onSend: () => void
  onStop: () => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onClearProductContext: () => void
  onClearCampaignContext: () => void
  onClearSourceContext: () => void
  onToggleCampaignContext: (campaign: AgentChatCampaignContextSelection) => void
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  onToggleSourceContext: (source: AgentChatSourceContextSelection) => void
  onDeleteSelectedSources: () => void
  onAttachExistingSource: (fileId: string) => void
  onUploadAttachments: (files: File[]) => void
  selectedProducts: AgentChatProductContextSelection[]
  selectedCampaigns: AgentChatCampaignContextSelection[]
  selectedSources: AgentChatSourceContextSelection[]
  sourceLibrary: AgentChatSourceLibraryItem[]
  sourceLibraryError: string | null
  playbooks: AgentChatPlaybookSelection[]
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId: AgentChatStrategyId
}
