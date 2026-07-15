import { useEffect, useRef } from "react"
import type { AgentChatComposerProps } from "./agent-chat-composer-types"
import { AgentChatAttachmentStrip } from "./agent-chat-attachment-strip"
import { AgentChatComposerInput } from "./agent-chat-composer-input"
import { AgentChatComposerPanels } from "./agent-chat-composer-panels"
import { AgentChatComposerStatus } from "./agent-chat-composer-status"
import { AgentChatComposerToolbar } from "./agent-chat-composer-toolbar"
import { useAgentChatComposerEvents } from "../view-model/use-agent-chat-composer-events"
import { useAgentChatComposerPanelLayout } from "../view-model/use-agent-chat-composer-panel-layout"
import { useAgentChatPanelOutsideClose } from "../view-model/use-agent-chat-panel-outside-close"
import { useAgentChatPanelState } from "../view-model/use-agent-chat-panel-state"

export function AgentChatComposer({
  attachments,
  availableCampaigns,
  availableProducts,
  availableSources,
  canSend,
  children,
  contextPreview,
  error,
  focusVersion,
  heading,
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
  placement = "bottom",
  placeholder,
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
  const panelLayout = useAgentChatComposerPanelLayout(composerRootRef)
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
  const isWelcomePlacement = placement === "welcome"
  useEffect(() => {
    const textarea = textareaRef.current
    if (!textarea) return
    const minHeight = isWelcomePlacement ? 72 : 40
    textarea.style.height = "auto"
    textarea.style.height = `${Math.min(Math.max(textarea.scrollHeight, minHeight), 180)}px`
  }, [input, isWelcomePlacement])
  useEffect(() => {
    if (!focusVersion || isStreaming) return
    textareaRef.current?.focus()
  }, [focusVersion, isStreaming])
  const footerClassName = isWelcomePlacement
    ? "relative z-30 w-full shrink-0"
    : "relative z-30 shrink-0 px-4 pt-3 pb-5 sm:px-7"
  const headingClassName = isWelcomePlacement
    ? "mx-auto mb-8 w-full max-w-4xl text-center text-2xl font-semibold text-foreground sm:text-3xl 2xl:text-[32px]"
    : "mx-auto mb-4 w-full max-w-4xl text-center text-xl font-semibold text-foreground sm:text-2xl"
  return (
    <footer className={footerClassName}>
      {heading ? <div className={headingClassName}>{heading}</div> : null}
      <div
        ref={composerRootRef}
        data-agent-chat-composer-root="true"
        className="mx-auto flex w-full max-w-4xl flex-col gap-2 rounded-[26px] border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) p-2 shadow-(--agent-chat-floating-shadow) backdrop-blur-xl transition-colors focus-within:border-primary/35 focus-within:shadow-(--agent-chat-focus-shadow) dark:bg-card"
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
            panelMaxHeight={panelLayout.maxHeight}
            panelPlacement={panelLayout.placement}
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
              placeholder={placeholder}
              size={isWelcomePlacement ? "welcome" : "default"}
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
      {children ? (
        <div className="mx-auto w-full max-w-4xl">{children}</div>
      ) : null}
    </footer>
  )
}
