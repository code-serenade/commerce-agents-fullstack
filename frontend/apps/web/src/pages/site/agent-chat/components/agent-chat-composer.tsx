import {
  type ChangeEvent,
  type MouseEvent,
  useEffect,
  useRef,
} from "react"
import type { AgentChatComposerProps } from "./agent-chat-composer-types"
import { AgentChatAttachmentStrip } from "./agent-chat-attachment-strip"
import { AgentChatComposerInput } from "./agent-chat-composer-input"
import { AgentChatComposerPanels } from "./agent-chat-composer-panels"
import { AgentChatComposerStatus } from "./agent-chat-composer-status"
import { AgentChatComposerToolbar } from "./agent-chat-composer-toolbar"
import { useAgentChatPanelState } from "../view-model/use-agent-chat-panel-state"

export function AgentChatComposer({
  attachments,
  availableCampaigns,
  availableProducts,
  availableSources,
  canSend,
  error,
  focusVersion,
  input,
  isLoggedIn,
  isLoadingCampaigns,
  isLoadingPlaybooks,
  isLoadingProducts,
  isLoadingRuntimeOptions,
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
  onUploadAttachments,
  selectedCampaigns,
  selectedProducts,
  selectedSources,
  playbooks,
  runtimeOptions,
  runtimeSelection,
  strategyId,
}: AgentChatComposerProps) {
  const composerRootRef = useRef<HTMLDivElement>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const panels = useAgentChatPanelState()
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

  useEffect(() => {
    if (!campaignContextOpen && !productContextOpen && !sourceContextOpen && !settingsOpen) return

    const closePanelsOnOutsideEvent = (event: Event) => {
      const target = event.target
      if (!(target instanceof Node)) return
      if (composerRootRef.current?.contains(target)) return
      closePanels()
    }

    document.addEventListener("pointerdown", closePanelsOnOutsideEvent, true)
    document.addEventListener("click", closePanelsOnOutsideEvent, true)
    return () => {
      document.removeEventListener(
        "pointerdown",
        closePanelsOnOutsideEvent,
        true
      )
      document.removeEventListener("click", closePanelsOnOutsideEvent, true)
    }
  }, [
    closePanels,
    campaignContextOpen,
    productContextOpen,
    settingsOpen,
    sourceContextOpen,
  ])

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

  const sendAndClosePanels = () => {
    closePanels()
    onSend()
  }

  const focusComposer = (event: MouseEvent<HTMLDivElement>) => {
    if (isStreaming) return
    const target = event.target
    if (!(target instanceof HTMLElement)) return
    if (target.closest("button, select, textarea, [data-composer-interactive='true']"))
      return
    textareaRef.current?.focus()
  }

  const handleAttachmentChange = (event: ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files ?? [])
    event.target.value = ""
    if (!files.length) return
    onUploadAttachments(files)
  }

  return (
    <footer className="relative z-30 shrink-0 bg-linear-to-t from-background via-background/95 to-background/70 px-3 pb-3 pt-2 sm:px-5">
      <div
        ref={composerRootRef}
        className="mx-auto flex w-full max-w-4xl flex-col gap-2 rounded-xl border border-muted-foreground/35 bg-background p-1 shadow-lg shadow-black/10 transition-colors focus-within:border-foreground/40 focus-within:ring-2 focus-within:ring-ring/15 dark:bg-card"
        onMouseDown={focusComposer}
      >
        <div className="relative">
          <AgentChatComposerPanels
            availableCampaigns={availableCampaigns}
            availableProducts={availableProducts}
            availableSources={availableSources}
            isLoadingCampaigns={isLoadingCampaigns}
            isLoadingPlaybooks={isLoadingPlaybooks}
            isLoadingProducts={isLoadingProducts}
            isLoadingRuntimeOptions={isLoadingRuntimeOptions}
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
            onToggleRuntimeRole={onToggleRuntimeRole}
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

          <div className="grid gap-1 rounded-lg bg-background p-1.5 shadow-sm">
            <AgentChatComposerStatus
              error={error}
              isLoggedIn={isLoggedIn}
              isStreaming={isStreaming}
            />

            <AgentChatComposerInput
              canSend={canSend}
              fileInputRef={fileInputRef}
              input={input}
              isLoggedIn={isLoggedIn}
              isStreaming={isStreaming}
              onAttachmentChange={handleAttachmentChange}
              onInputChange={onInputChange}
              onSend={sendAndClosePanels}
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
              onAttach={() => fileInputRef.current?.click()}
              onRuntimeCollaborationModeChange={onRuntimeCollaborationModeChange}
              onRuntimeLearningLevelChange={onRuntimeLearningLevelChange}
              onRuntimePrimaryRoleChange={onRuntimePrimaryRoleChange}
              onRuntimeRoutingModeChange={onRuntimeRoutingModeChange}
              onSelectPlaybook={(playbook) => {
                onSelectPlaybook(playbook)
                queueMicrotask(() => textareaRef.current?.focus())
              }}
              onSend={sendAndClosePanels}
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
