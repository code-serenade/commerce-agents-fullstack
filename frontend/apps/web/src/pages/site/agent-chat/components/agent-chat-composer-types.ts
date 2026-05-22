import type {
  AgentChatCampaignContextSelection,
  AgentChatProductContextSelection,
  AgentChatSourceContextSelection,
} from "../domain/agent-chat-context-selection"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatAttachmentItem } from "../types"

export interface AgentChatComposerProps {
  attachments: AgentChatAttachmentItem[]
  availableProducts: AgentChatProductContextSelection[]
  availableCampaigns: AgentChatCampaignContextSelection[]
  availableSources: AgentChatSourceContextSelection[]
  canSend: boolean
  error: string | null
  focusVersion?: number
  input: string
  isLoggedIn: boolean
  isLoadingPlaybooks: boolean
  isLoadingCampaigns: boolean
  isLoadingProducts: boolean
  isLoadingRuntimeOptions: boolean
  isStreaming: boolean
  onInputChange: (value: string) => void
  onRuntimeCollaborationModeChange: (value: string) => void
  onRuntimeLearningLevelChange: (value: string) => void
  onRuntimePrimaryRoleChange: (value: string) => void
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
  onUploadAttachments: (files: File[]) => void
  selectedProducts: AgentChatProductContextSelection[]
  selectedCampaigns: AgentChatCampaignContextSelection[]
  selectedSources: AgentChatSourceContextSelection[]
  playbooks: AgentChatPlaybookSelection[]
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId: AgentChatStrategyId
}
