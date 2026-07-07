import { useEffect, useRef } from "react"
import type { AgentChatComposerProps } from "./agent-chat-composer-types"
import { AgentChatAttachmentStrip } from "./agent-chat-attachment-strip"
import { AgentChatComposerInput } from "./agent-chat-composer-input"
import { AgentChatComposerPanels } from "./agent-chat-composer-panels"
import { AgentChatComposerStatus } from "./agent-chat-composer-status"
import { AgentChatComposerToolbar } from "./agent-chat-composer-toolbar"
import { useAgentChatComposerEvents } from "../view-model/use-agent-chat-composer-events"
import { useAgentChatComposerPanelMaxHeight } from "../view-model/use-agent-chat-composer-panel-layout"
import { useAgentChatPanelOutsideClose } from "../view-model/use-agent-chat-panel-outside-close"
import { useAgentChatPanelState } from "../view-model/use-agent-chat-panel-state"

export function AgentChatComposer({
  attachments,
  availableCampaigns,
  availableProducts,
  availableSources,
  canSend,
  contextPreview,
  error,
  focusVersion,
  requestedPanel,
  requestedPanelVersion,
  input,
  isLoggedIn,
  isLoadingCampaigns,
  isLoadingPlaybooks,
  isLoadingProducts,
  isLoadingRuntimeOptions,
  isLoadingSourceLibrary,
  isStreaming,
  onInputChange,
  onRuntimeCollaborationModeChange,
  onRuntimeLearningLevelChange,
  onRuntimePrimaryRoleChange,
  onRuntimeRoutingModeChange,
  onToggleRuntimeRole,
  onSelectPlaybook,
  onSend,
  onStop,
  onStrategyChange,
  onClearCampaignContext,
  onClearProductContext,
  onClearSourceContext,
  onToggleCampaignContext,
  onToggleProductContext,
  onToggleSourceContext,
  onDeleteSelectedSources,
  onAttachExistingSource,
  onUploadAttachments,
  selectedCampaigns,
  selectedProducts,
  selectedSources,
  sourceLibrary,
  sourceLibraryError,
  playbooks,
  runtimeOptions,
  runtimeSelection,
  strategyId,
}: AgentChatComposerProps) {
  const composerRootRef = useRef<HTMLDivElement>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const panelMaxHeight = useAgentChatComposerPanelMaxHeight(composerRootRef)
  const panels = useAgentChatPanelState({
    requestedPanel,
    requestedPanelVersion,
  })
  const {
    campaignContextOpen,
    closePanels,
    productContextOpen,
    setCampaignContextOpen,
    setProductContextOpen,
    setSourceContextOpen,
    settingsOpen,
    sourceContextOpen,
    toggleSettingsPanel,
  } = panels
  const composerEvents = useAgentChatComposerEvents({
    closePanels,
    fileInputRef,
    isStreaming,
    onSend,
    onUploadAttachments,
    textareaRef,
  })
  useAgentChatPanelOutsideClose({
    closePanels,
    composerRootRef,
    hasOpenPanel:
      campaignContextOpen ||
      productContextOpen ||
      sourceContextOpen ||
      settingsOpen,
  })

  useEffect(() => {
    const textarea = textareaRef.current
    if (!textarea) return
    textarea.style.height = "auto"
    textarea.style.height = `${Math.min(textarea.scrollHeight, 180)}px`
  }, [input])

  useEffect(() => {
    if (!focusVersion || isStreaming) return
    textareaRef.current?.focus()
  }, [focusVersion, isStreaming])

  return (
    <footer className="relative z-30 shrink-0 px-4 pt-3 pb-5 sm:px-7">
      <div
        ref={composerRootRef}
        data-agent-chat-composer-root="true"
        className="mx-auto flex w-full max-w-4xl flex-col gap-2 rounded-[26px] border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) p-2 shadow-[var(--agent-chat-floating-shadow)] backdrop-blur-xl transition-colors focus-within:border-primary/35 focus-within:shadow-[var(--agent-chat-focus-shadow)] dark:bg-card"
        onMouseDown={composerEvents.focusComposer}
      >
        <div className="relative">
          <AgentChatComposerPanels
            availableCampaigns={availableCampaigns}
            availableProducts={availableProducts}
            availableSources={availableSources}
            contextPreview={contextPreview}
            contextPreviewOpen={false}
            isLoadingCampaigns={isLoadingCampaigns}
            isLoadingPlaybooks={isLoadingPlaybooks}
            isLoadingProducts={isLoadingProducts}
            isLoadingRuntimeOptions={isLoadingRuntimeOptions}
            isLoadingSourceLibrary={isLoadingSourceLibrary}
            isStreaming={isStreaming}
            onFocusComposer={() => textareaRef.current?.focus()}
            onRuntimeCollaborationModeChange={onRuntimeCollaborationModeChange}
            onRuntimeLearningLevelChange={onRuntimeLearningLevelChange}
            onRuntimePrimaryRoleChange={onRuntimePrimaryRoleChange}
            onRuntimeRoutingModeChange={onRuntimeRoutingModeChange}
            onSelectPlaybook={onSelectPlaybook}
            onStrategyChange={onStrategyChange}
            onClearCampaignContext={onClearCampaignContext}
            onClearProductContext={onClearProductContext}
            onClearSourceContext={onClearSourceContext}
            onToggleCampaignContext={onToggleCampaignContext}
            onToggleProductContext={onToggleProductContext}
            onToggleSourceContext={onToggleSourceContext}
            onUploadClick={composerEvents.openUploadFilePicker}
            onDeleteSelectedSources={onDeleteSelectedSources}
            onAttachExistingSource={onAttachExistingSource}
            onClosePanels={closePanels}
            onManageSources={composerEvents.manageSources}
            onToggleRuntimeRole={onToggleRuntimeRole}
            panelMaxHeight={panelMaxHeight}
            playbooks={playbooks}
            campaignContextOpen={campaignContextOpen}
            productContextOpen={productContextOpen}
            runtimeOptions={runtimeOptions}
            runtimeSelection={runtimeSelection}
            selectedCampaigns={selectedCampaigns}
            selectedProducts={selectedProducts}
            selectedSources={selectedSources}
            sourceLibrary={sourceLibrary}
            sourceLibraryError={sourceLibraryError}
            settingsOpen={settingsOpen}
            sourceContextOpen={sourceContextOpen}
            strategyId={strategyId}
          />

          <div className="grid gap-1 rounded-[20px] bg-(--agent-chat-elevated-strong) p-1.5">
            <AgentChatComposerStatus
              canRetry={canSend}
              error={error}
              isLoggedIn={isLoggedIn}
              isStreaming={isStreaming}
              onRetry={composerEvents.sendAndClosePanels}
            />

            <AgentChatComposerInput
              canSend={canSend}
              fileInputRef={fileInputRef}
              input={input}
              isLoggedIn={isLoggedIn}
              isStreaming={isStreaming}
              onAttachmentChange={composerEvents.handleAttachmentChange}
              onInputChange={onInputChange}
              onSend={composerEvents.sendAndClosePanels}
              onStop={onStop}
              textareaRef={textareaRef}
            />

            <AgentChatAttachmentStrip attachments={attachments} />

            <AgentChatComposerToolbar
              availableCampaigns={availableCampaigns}
              availableProducts={availableProducts}
              availableSources={availableSources}
              canSend={canSend}
              isLoggedIn={isLoggedIn}
              isLoadingCampaigns={isLoadingCampaigns}
              isLoadingPlaybooks={isLoadingPlaybooks}
              isLoadingProducts={isLoadingProducts}
              isLoadingRuntimeOptions={isLoadingRuntimeOptions}
              isStreaming={isStreaming}
              onRuntimeCollaborationModeChange={
                onRuntimeCollaborationModeChange
              }
              onRuntimeLearningLevelChange={onRuntimeLearningLevelChange}
              onRuntimePrimaryRoleChange={onRuntimePrimaryRoleChange}
              onRuntimeRoutingModeChange={onRuntimeRoutingModeChange}
              onSelectPlaybook={(playbook) => {
                onSelectPlaybook(playbook)
                queueMicrotask(() => textareaRef.current?.focus())
              }}
              onSend={composerEvents.sendAndClosePanels}
              onStop={onStop}
              onStrategyChange={onStrategyChange}
              onClearCampaignContext={onClearCampaignContext}
              onClearProductContext={onClearProductContext}
              onClearSourceContext={onClearSourceContext}
              onToggleCampaignContext={onToggleCampaignContext}
              onToggleCampaignPanel={setCampaignContextOpen}
              onToggleProductContext={onToggleProductContext}
              onToggleProductPanel={setProductContextOpen}
              onToggleSourceContext={onToggleSourceContext}
              onToggleSourcePanel={setSourceContextOpen}
              onToggleRuntimeRole={onToggleRuntimeRole}
              onToggleSettingsPanel={toggleSettingsPanel}
              playbooks={playbooks}
              campaignContextOpen={campaignContextOpen}
              productContextOpen={productContextOpen}
              runtimeOptions={runtimeOptions}
              runtimeSelection={runtimeSelection}
              selectedCampaigns={selectedCampaigns}
              selectedProducts={selectedProducts}
              selectedSources={selectedSources}
              settingsOpen={settingsOpen}
              sourceContextOpen={sourceContextOpen}
              strategyId={strategyId}
            />
          </div>
        </div>
      </div>
    </footer>
  )
}
