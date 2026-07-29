import { useEffect, useRef } from "react"
import type { AgentChatComposerProps } from "./agent-chat-composer-types"
import { AgentChatAttachmentStrip } from "./agent-chat-attachment-strip"
import { AgentChatComposerInput } from "./agent-chat-composer-input"
import { AgentChatComposerPanels } from "./agent-chat-composer-panels"
import { AgentChatComposerStatus } from "./agent-chat-composer-status"
import { AgentChatComposerToolbar } from "./agent-chat-composer-toolbar"
import { useAgentChatBottomPanelReserve } from "../view-model/use-agent-chat-bottom-panel-reserve"
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
  beforeChildren,
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
  const isWelcomePlacement = placement === "welcome"
  const panelLayout = useAgentChatComposerPanelLayout(composerRootRef, "top")
  const panels = useAgentChatPanelState({ requestedPanel, requestedPanelVersion })
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
  const hasOpenComposerPanel = campaignContextOpen || productContextOpen || sourceContextOpen || settingsOpen
  useAgentChatPanelOutsideClose({
    closePanels,
    composerRootRef,
    hasOpenPanel: hasOpenComposerPanel,
  })
  const composerRootStyle = useAgentChatBottomPanelReserve({
    composerRootRef,
    hasOpenPanel: hasOpenComposerPanel,
    placement: panelLayout.placement,
  })
  useEffect(() => {
    const textarea = textareaRef.current
    if (!textarea) return
    const minHeight = isWelcomePlacement ? 56 : 40
    textarea.style.height = "auto"
    textarea.style.height = `${Math.min(Math.max(textarea.scrollHeight, minHeight), 180)}px`
  }, [input, isWelcomePlacement])
  useEffect(() => {
    if (!focusVersion || isStreaming) return
    textareaRef.current?.focus()
  }, [focusVersion, isStreaming])
  const footerClassName = isWelcomePlacement ? "relative z-30 w-full shrink-0" : "relative z-30 shrink-0 px-4 pt-3 pb-5 sm:px-7"
  const headingClassName = isWelcomePlacement ? "mx-auto mb-7 w-full max-w-6xl text-center text-[clamp(1.8rem,2.45vw,2.25rem)] leading-tight font-bold text-foreground" : "mx-auto mb-4 w-full max-w-4xl text-center text-xl font-semibold text-foreground sm:text-2xl"
  const composerRootClassName = isWelcomePlacement
    ? "mx-auto flex w-full max-w-6xl flex-col gap-2 rounded-2xl border border-primary/90 bg-(--agent-chat-elevated-strong) p-2 shadow-(--agent-chat-floating-shadow) backdrop-blur-xl transition-colors focus-within:border-primary focus-within:shadow-(--agent-chat-focus-shadow) dark:border-(--agent-chat-soft-border) dark:bg-card dark:focus-within:border-primary/35"
    : "mx-auto flex w-full max-w-4xl flex-col gap-2 rounded-[26px] border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) p-2 shadow-(--agent-chat-floating-shadow) backdrop-blur-xl transition-colors focus-within:border-primary/35 focus-within:shadow-(--agent-chat-focus-shadow) dark:bg-card"
  const inputShellClassName = isWelcomePlacement ? "grid gap-1 rounded-xl bg-(--agent-chat-elevated-strong) p-1.5" : "grid gap-1 rounded-[20px] bg-(--agent-chat-elevated-strong) p-1.5"
  return (
    <footer className={footerClassName}>
      {heading ? <div className={headingClassName}>{heading}</div> : null}
      {beforeChildren ? (
        <div className="mx-auto mb-7 w-full max-w-6xl">{beforeChildren}</div>
      ) : null}
      <div
        ref={composerRootRef}
        data-agent-chat-composer-root="true"
        className={composerRootClassName}
        style={composerRootStyle}
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
          <div className={inputShellClassName}>
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
              onRuntimeCollaborationModeChange={onRuntimeCollaborationModeChange}
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
        <div className="mx-auto w-full max-w-6xl">{children}</div>
      ) : null}
    </footer>
  )
}
