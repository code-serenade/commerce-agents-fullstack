import type {
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
  availableSources: AgentChatSourceContextSelection[]
  canSend: boolean
  input: string
  isLoggedIn: boolean
  isLoadingPlaybooks: boolean
  isLoadingProducts: boolean
  isLoadingRuntimeOptions: boolean
  isStreaming: boolean
  onInputChange: (value: string) => void
  onRuntimeCollaborationModeChange: (value: string) => void
  onRuntimeLearningLevelChange: (value: string) => void
  onRuntimeModelChange: (value: string) => void
  onRuntimePrimaryRoleChange: (value: string) => void
  onToggleRuntimeRole: (value: string) => void
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  onSend: () => void
  onStop: () => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onClearProductContext: () => void
  onClearSourceContext: () => void
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  onToggleSourceContext: (source: AgentChatSourceContextSelection) => void
  onUploadAttachments: (files: File[]) => void
  selectedProducts: AgentChatProductContextSelection[]
  selectedSources: AgentChatSourceContextSelection[]
  playbooks: AgentChatPlaybookSelection[]
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId: AgentChatStrategyId
}
