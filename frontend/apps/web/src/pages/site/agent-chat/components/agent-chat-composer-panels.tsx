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
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import type { AgentChatComposerPanelPlacement } from "../view-model/use-agent-chat-composer-panel-layout"
import type { AgentChatContextPreviewState } from "../view-model/use-agent-chat-context-preview"
import { AgentChatContextPreviewPanel } from "./agent-chat-context-preview-panel"
import { AgentChatComposerPanelShell } from "./agent-chat-composer-panel-shell"
import { AgentChatCampaignContextBar } from "./agent-chat-campaign-context-bar"
import { AgentChatProductContextBar } from "./agent-chat-product-context-bar"
import { AgentChatSettingsPanel } from "./agent-chat-settings-panel"
import { AgentChatSourceContextBar } from "./agent-chat-source-context-bar"

export function AgentChatComposerPanels({
  availableCampaigns,
  availableProducts,
  availableSources,
  contextPreview,
  contextPreviewOpen,
  isLoadingCampaigns,
  isLoadingPlaybooks,
  isLoadingProducts,
  isLoadingRuntimeOptions,
  isLoadingSourceLibrary,
  isStreaming,
  onRuntimeCollaborationModeChange,
  onRuntimeLearningLevelChange,
  onRuntimePrimaryRoleChange,
  onRuntimeRoutingModeChange,
  onSelectPlaybook,
  onStrategyChange,
  onClearCampaignContext,
  onClearProductContext,
  onClearSourceContext,
  onDeleteSelectedSources,
  onToggleCampaignContext,
  onToggleProductContext,
  onToggleSourceContext,
  onToggleRuntimeRole,
  onUploadClick,
  onAttachExistingSource,
  onClosePanels,
  panelMaxHeight,
  panelPlacement,
  playbooks,
  campaignContextOpen,
  productContextOpen,
  runtimeOptions,
  runtimeSelection,
  selectedCampaigns,
  selectedProducts,
  selectedSources,
  sourceLibrary,
  sourceLibraryError,
  settingsOpen,
  sourceContextOpen,
  strategyId,
}: {
  availableCampaigns: AgentChatCampaignContextSelection[]
  availableProducts: AgentChatProductContextSelection[]
  availableSources: AgentChatSourceContextSelection[]
  contextPreview: AgentChatContextPreviewState
  contextPreviewOpen: boolean
  isLoadingCampaigns: boolean
  isLoadingPlaybooks: boolean
  isLoadingProducts: boolean
  isLoadingRuntimeOptions: boolean
  isLoadingSourceLibrary: boolean
  isStreaming: boolean
  onFocusComposer?: () => void
  onRuntimeCollaborationModeChange: (value: string) => void
  onRuntimeLearningLevelChange: (value: string) => void
  onRuntimePrimaryRoleChange: (value: string) => void
  onRuntimeRoutingModeChange: (value: string) => void
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onClearCampaignContext: () => void
  onClearProductContext: () => void
  onClearSourceContext: () => void
  onDeleteSelectedSources: () => void
  onToggleCampaignContext: (campaign: AgentChatCampaignContextSelection) => void
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  onToggleSourceContext: (source: AgentChatSourceContextSelection) => void
  onToggleRuntimeRole: (value: string) => void
  onUploadClick: () => void
  onAttachExistingSource: (fileId: string) => void
  onClosePanels: () => void
  panelMaxHeight: number
  panelPlacement: AgentChatComposerPanelPlacement
  playbooks: AgentChatPlaybookSelection[]
  campaignContextOpen: boolean
  productContextOpen: boolean
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  selectedCampaigns: AgentChatCampaignContextSelection[]
  selectedProducts: AgentChatProductContextSelection[]
  selectedSources: AgentChatSourceContextSelection[]
  sourceLibrary: AgentChatSourceLibraryItem[]
  sourceLibraryError: string | null
  settingsOpen: boolean
  sourceContextOpen: boolean
  strategyId: AgentChatStrategyId
}) {
  return (
    <>
      <AgentChatComposerPanelShell
        maxHeight={panelMaxHeight}
        onClose={onClosePanels}
        open={contextPreviewOpen}
        placement={panelPlacement}
        size="wide"
      >
        <AgentChatContextPreviewPanel
          canPreview={contextPreview.canPreviewContext}
          error={contextPreview.previewContextError}
          isPreviewing={contextPreview.isPreviewingContext}
          onPreview={contextPreview.previewContext}
          preview={contextPreview.previewContextResult}
        />
      </AgentChatComposerPanelShell>
      <AgentChatComposerPanelShell
        maxHeight={panelMaxHeight}
        onClose={onClosePanels}
        open={productContextOpen}
        placement={panelPlacement}
        size="wide"
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
      </AgentChatComposerPanelShell>
      <AgentChatComposerPanelShell
        maxHeight={panelMaxHeight}
        onClose={onClosePanels}
        open={campaignContextOpen}
        placement={panelPlacement}
        size="wide"
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
      </AgentChatComposerPanelShell>
      <AgentChatComposerPanelShell
        maxHeight={panelMaxHeight}
        onClose={onClosePanels}
        open={sourceContextOpen}
        placement={panelPlacement}
        size="wide"
      >
        <AgentChatSourceContextBar
          availableSources={availableSources}
          disabled={isStreaming}
          isLoadingSourceLibrary={isLoadingSourceLibrary}
          sourceLibrary={sourceLibrary}
          sourceLibraryError={sourceLibraryError}
          onDeleteSelectedSources={onDeleteSelectedSources}
          onAttachExistingSource={onAttachExistingSource}
          onUploadClick={onUploadClick}
          onClearSourceContext={onClearSourceContext}
          onToggleSourceContext={onToggleSourceContext}
          selectedSources={selectedSources}
          variant="panel"
        />
      </AgentChatComposerPanelShell>
      <AgentChatComposerPanelShell
        maxHeight={panelMaxHeight}
        onClose={onClosePanels}
        open={settingsOpen}
        placement={panelPlacement}
        size="wide"
      >
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
          onToggleOpen={() => undefined}
          onToggleRole={onToggleRuntimeRole}
          playbooks={playbooks}
          runtimeOptions={runtimeOptions}
          runtimeSelection={runtimeSelection}
          strategyId={strategyId}
          variant="panel"
        />
      </AgentChatComposerPanelShell>
    </>
  )
}
