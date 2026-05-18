import type {
  AgentChatCampaignContextSelection,
  AgentChatProductContextSelection,
  AgentChatSourceContextSelection,
} from "../domain/agent-chat-context-selection"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { AgentChatCampaignContextBar } from "./agent-chat-campaign-context-bar"
import { AgentChatProductContextBar } from "./agent-chat-product-context-bar"
import { AgentChatSettingsPanel } from "./agent-chat-settings-panel"
import { AgentChatSourceContextBar } from "./agent-chat-source-context-bar"

export function AgentChatComposerPanels({
  availableCampaigns,
  availableProducts,
  availableSources,
  isLoadingCampaigns,
  isLoadingPlaybooks,
  isLoadingProducts,
  isLoadingRuntimeOptions,
  isStreaming,
  onRuntimeCollaborationModeChange,
  onRuntimeLearningLevelChange,
  onRuntimeModelChange,
  onRuntimePrimaryRoleChange,
  onSelectPlaybook,
  onStrategyChange,
  onClearCampaignContext,
  onClearProductContext,
  onClearSourceContext,
  onToggleCampaignContext,
  onToggleProductContext,
  onToggleSourceContext,
  onToggleRuntimeRole,
  playbooks,
  campaignContextOpen,
  productContextOpen,
  runtimeOptions,
  runtimeSelection,
  selectedCampaigns,
  selectedProducts,
  selectedSources,
  settingsOpen,
  sourceContextOpen,
  strategyId,
}: {
  availableCampaigns: AgentChatCampaignContextSelection[]
  availableProducts: AgentChatProductContextSelection[]
  availableSources: AgentChatSourceContextSelection[]
  isLoadingCampaigns: boolean
  isLoadingPlaybooks: boolean
  isLoadingProducts: boolean
  isLoadingRuntimeOptions: boolean
  isStreaming: boolean
  onFocusComposer?: () => void
  onRuntimeCollaborationModeChange: (value: string) => void
  onRuntimeLearningLevelChange: (value: string) => void
  onRuntimeModelChange: (value: string) => void
  onRuntimePrimaryRoleChange: (value: string) => void
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onClearCampaignContext: () => void
  onClearProductContext: () => void
  onClearSourceContext: () => void
  onToggleCampaignContext: (campaign: AgentChatCampaignContextSelection) => void
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  onToggleSourceContext: (source: AgentChatSourceContextSelection) => void
  onToggleRuntimeRole: (value: string) => void
  playbooks: AgentChatPlaybookSelection[]
  campaignContextOpen: boolean
  productContextOpen: boolean
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  selectedCampaigns: AgentChatCampaignContextSelection[]
  selectedProducts: AgentChatProductContextSelection[]
  selectedSources: AgentChatSourceContextSelection[]
  settingsOpen: boolean
  sourceContextOpen: boolean
  strategyId: AgentChatStrategyId
}) {
  return (
    <>
      {productContextOpen ? (
        <div
          className="absolute inset-x-0 bottom-full z-50 mb-2 rounded-xl border border-border bg-background p-3 shadow-2xl shadow-black/15"
          data-composer-interactive="true"
        >
          <AgentChatProductContextBar
            availableProducts={availableProducts}
            disabled={isStreaming}
            isLoadingProducts={isLoadingProducts}
            onClearProductContext={onClearProductContext}
            onToggleProductContext={onToggleProductContext}
            selectedProducts={selectedProducts}
            variant="panel"
          />
        </div>
      ) : null}
      {campaignContextOpen ? (
        <div
          className="absolute inset-x-0 bottom-full z-50 mb-2 rounded-xl border border-border bg-background p-3 shadow-2xl shadow-black/15"
          data-composer-interactive="true"
        >
          <AgentChatCampaignContextBar
            availableCampaigns={availableCampaigns}
            disabled={isStreaming}
            isLoadingCampaigns={isLoadingCampaigns}
            onClearCampaignContext={onClearCampaignContext}
            onToggleCampaignContext={onToggleCampaignContext}
            selectedCampaigns={selectedCampaigns}
            variant="panel"
          />
        </div>
      ) : null}
      {sourceContextOpen ? (
        <div
          className="absolute inset-x-0 bottom-full z-50 mb-2 rounded-xl border border-border bg-background p-3 shadow-2xl shadow-black/15"
          data-composer-interactive="true"
        >
          <AgentChatSourceContextBar
            availableSources={availableSources}
            disabled={isStreaming}
            onClearSourceContext={onClearSourceContext}
            onToggleSourceContext={onToggleSourceContext}
            selectedSources={selectedSources}
            variant="panel"
          />
        </div>
      ) : null}
      {settingsOpen ? (
        <div
          className="absolute inset-x-0 bottom-full z-50 mb-2 grid max-h-[min(30rem,72vh)] gap-2 overflow-y-auto rounded-xl border border-border bg-background p-2 shadow-2xl shadow-black/15"
          data-composer-interactive="true"
        >
          <AgentChatSettingsPanel
            isLoadingPlaybooks={isLoadingPlaybooks}
            isLoadingRuntimeOptions={isLoadingRuntimeOptions}
            isOpen
            isStreaming={isStreaming}
            onCollaborationModeChange={onRuntimeCollaborationModeChange}
            onLearningLevelChange={onRuntimeLearningLevelChange}
            onModelChange={onRuntimeModelChange}
            onPrimaryRoleChange={onRuntimePrimaryRoleChange}
            onSelectPlaybook={onSelectPlaybook}
            onStrategyChange={onStrategyChange}
            onToggleOpen={() => undefined}
            onToggleRole={onToggleRuntimeRole}
            playbooks={playbooks}
            runtimeOptions={runtimeOptions}
            runtimeSelection={runtimeSelection}
            strategyId={strategyId}
            variant="panel"
          />
        </div>
      ) : null}
    </>
  )
}
