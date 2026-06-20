import { LockKeyhole, Send, Square } from "lucide-react"
import { useTranslation } from "react-i18next"
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

export function AgentChatComposerToolbar({
  availableCampaigns,
  availableProducts,
  availableSources,
  canSend,
  isLoggedIn,
  isLoadingCampaigns,
  isLoadingPlaybooks,
  isLoadingProducts,
  isLoadingRuntimeOptions,
  isStreaming,
  onRuntimeCollaborationModeChange,
  onRuntimeLearningLevelChange,
  onRuntimePrimaryRoleChange,
  onRuntimeRoutingModeChange,
  onSelectPlaybook,
  onSend,
  onStop,
  onStrategyChange,
  onClearCampaignContext,
  onClearProductContext,
  onClearSourceContext,
  onToggleCampaignContext,
  onToggleCampaignPanel,
  onToggleProductContext,
  onToggleProductPanel,
  onToggleSourceContext,
  onToggleSourcePanel,
  onToggleRuntimeRole,
  onToggleSettingsPanel,
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
  canSend: boolean
  isLoggedIn: boolean
  isLoadingCampaigns: boolean
  isLoadingPlaybooks: boolean
  isLoadingProducts: boolean
  isLoadingRuntimeOptions: boolean
  isStreaming: boolean
  onRuntimeCollaborationModeChange: (value: string) => void
  onRuntimeLearningLevelChange: (value: string) => void
  onRuntimePrimaryRoleChange: (value: string) => void
  onRuntimeRoutingModeChange: (value: string) => void
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  onSend: () => void
  onStop: () => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onClearCampaignContext: () => void
  onClearProductContext: () => void
  onClearSourceContext: () => void
  onToggleCampaignContext: (campaign: AgentChatCampaignContextSelection) => void
  onToggleCampaignPanel: (open: boolean) => void
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  onToggleProductPanel: (open: boolean) => void
  onToggleSourceContext: (source: AgentChatSourceContextSelection) => void
  onToggleSourcePanel: (open: boolean) => void
  onToggleRuntimeRole: (value: string) => void
  onToggleSettingsPanel: () => void
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
  const shouldShowProductContext =
    productContextOpen ||
    isLoadingProducts ||
    availableProducts.length > 0 ||
    selectedProducts.length > 0
  const shouldShowCampaignContext =
    campaignContextOpen ||
    isLoadingCampaigns ||
    availableCampaigns.length > 0 ||
    selectedCampaigns.length > 0

  return (
    <div className="flex min-w-0 items-center justify-between gap-2">
      <div className="flex min-w-0 flex-1 items-center gap-1.5 overflow-x-auto overflow-y-hidden pb-1 [scrollbar-gutter:stable] sm:overflow-hidden sm:pb-0">
        <AgentChatSourceContextBar
          availableSources={availableSources}
          disabled={isStreaming || !isLoggedIn}
          isOpen={sourceContextOpen}
          onClearSourceContext={onClearSourceContext}
          onOpenChange={onToggleSourcePanel}
          onToggleSourceContext={onToggleSourceContext}
          selectedSources={selectedSources}
          variant="compact"
        />
        {shouldShowProductContext ? (
          <AgentChatProductContextBar
            availableProducts={availableProducts}
            disabled={isStreaming}
            isLoadingProducts={isLoadingProducts}
            isOpen={productContextOpen}
            onClearProductContext={onClearProductContext}
            onOpenChange={onToggleProductPanel}
            onToggleProductContext={onToggleProductContext}
            selectedProducts={selectedProducts}
            variant="compact"
          />
        ) : null}
        {shouldShowCampaignContext ? (
          <AgentChatCampaignContextBar
            availableCampaigns={availableCampaigns}
            disabled={isStreaming}
            isLoadingCampaigns={isLoadingCampaigns}
            isOpen={campaignContextOpen}
            onClearCampaignContext={onClearCampaignContext}
            onOpenChange={onToggleCampaignPanel}
            onToggleCampaignContext={onToggleCampaignContext}
            selectedCampaigns={selectedCampaigns}
            variant="compact"
          />
        ) : null}
        <AgentChatSettingsPanel
          isLoadingPlaybooks={isLoadingPlaybooks}
          isLoadingRuntimeOptions={isLoadingRuntimeOptions}
          isOpen={settingsOpen}
          isStreaming={isStreaming}
          onCollaborationModeChange={onRuntimeCollaborationModeChange}
          onLearningLevelChange={onRuntimeLearningLevelChange}
          onPrimaryRoleChange={onRuntimePrimaryRoleChange}
          onRoutingModeChange={onRuntimeRoutingModeChange}
          onSelectPlaybook={onSelectPlaybook}
          onStrategyChange={onStrategyChange}
          onToggleOpen={onToggleSettingsPanel}
          onToggleRole={onToggleRuntimeRole}
          playbooks={playbooks}
          runtimeOptions={runtimeOptions}
          runtimeSelection={runtimeSelection}
          strategyId={strategyId}
          variant="compact"
        />
      </div>
      <div className="sm:hidden">
        <AgentChatComposerSendButton
          canSend={canSend}
          isLoggedIn={isLoggedIn}
          isStreaming={isStreaming}
          onSend={onSend}
          onStop={onStop}
        />
      </div>
    </div>
  )
}

export function AgentChatComposerSendButton({
  canSend,
  isLoggedIn,
  isStreaming,
  onSend,
  onStop,
}: {
  canSend: boolean
  isLoggedIn: boolean
  isStreaming: boolean
  onSend: () => void
  onStop: () => void
}) {
  const { t } = useTranslation()

  if (isStreaming) {
    return (
      <button
        type="button"
        onClick={onStop}
        aria-label={t("agentChat.composer.stop")}
        className="flex size-8 shrink-0 items-center justify-center rounded-full border border-border bg-card text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
        title={t("agentChat.composer.stop")}
      >
        <Square className="size-4" />
      </button>
    )
  }

  return (
    <button
      type="button"
      disabled={!canSend}
      onClick={onSend}
      aria-label={t("agentChat.composer.send")}
      className="flex size-8 shrink-0 items-center justify-center rounded-full bg-foreground text-background transition-colors hover:bg-foreground/90 disabled:cursor-not-allowed disabled:opacity-45"
      title={t("agentChat.composer.send")}
    >
      {canSend ? (
        isLoggedIn ? (
          <Send className="size-4" />
        ) : (
          <LockKeyhole className="size-4" />
        )
      ) : (
        <Send className="size-4" />
      )}
    </button>
  )
}
