import { useMemo, useState } from "react"
import { useNavigate } from "react-router"
import type { AgentRuntimeEvent } from "./agent-chat-data"
import { AgentChatComposer } from "./components/agent-chat-composer"
import { AgentProcessDrawer } from "./components/agent-process-drawer"
import { AgentChatStatus } from "./components/agent-chat-status"
import { AgentChatThread } from "./components/agent-chat-thread"
import { useAgentChatContext } from "./context/use-agent-chat-context"
import {
  findLatestProcessTarget,
  type AgentProcessTarget,
} from "./view-model/agent-process-visibility-view-model"
import { useAgentChatRequestedComposerPanel } from "./view-model/use-agent-chat-requested-composer-panel"

type ActiveProcessTarget = AgentProcessTarget

export default function AgentChatPage() {
  const chat = useAgentChatContext()
  const navigate = useNavigate()
  const [composerFocusVersion, setComposerFocusVersion] = useState(0)
  const requestedComposerPanel = useAgentChatRequestedComposerPanel()
  const [activeProcessTarget, setActiveProcessTarget] =
    useState<ActiveProcessTarget | null>(null)
  const latestProcessTarget = useMemo(
    () => findLatestProcessTarget(chat.messages, chat.runtimeEvents),
    [chat.messages, chat.runtimeEvents]
  )
  const visibleProcessTarget = activeProcessTarget
  const activeProcessMessage = visibleProcessTarget
    ? chat.messages.find((message) => message.id === visibleProcessTarget.messageId)
    : null
  const activeRuntimeEvents: AgentRuntimeEvent[] =
    activeProcessMessage?.runtimeEvents?.length
      ? activeProcessMessage.runtimeEvents
      : activeProcessMessage?.id === latestProcessTarget?.messageId
        ? chat.runtimeEvents
        : []
  const shouldShowProcessPanel =
    Boolean(activeProcessMessage && activeRuntimeEvents.length > 0)

  const closeProcessPanel = () => {
    setActiveProcessTarget(null)
  }

  const openProcessPanel = (target: ActiveProcessTarget) => {
    setActiveProcessTarget(target)
  }

  const sendMessage = () => {
    setActiveProcessTarget(null)
    void chat.sendMessage()
  }

  const selectExamplePrompt = (prompt: string) => {
    chat.setInput(prompt)
    setComposerFocusVersion((current) => current + 1)
  }

  const layoutClassName = "relative flex min-h-0 w-full flex-1 overflow-hidden p-2 sm:p-3"

  const chatPanelClassName = [
    "mx-auto flex h-full min-h-0 w-full max-w-5xl min-w-0 flex-col overflow-hidden rounded-lg bg-background",
  ].join(" ")

  return (
    <div className="flex max-h-full min-h-0 size-full flex-1 flex-col overflow-hidden bg-background text-foreground">
      <AgentChatStatus
        error={chat.error}
        loginRequired={!chat.isLoggedIn}
        onLogin={() => navigate("/login", { state: { from: { pathname: "/commerce" } } })}
      />
      <div className={layoutClassName}>
        <section className={chatPanelClassName}>
          <AgentChatThread
            activeProcessMessageId={activeProcessMessage?.id ?? null}
            isStreaming={chat.isStreaming}
            messages={chat.messages}
            onCloseProcessPanel={closeProcessPanel}
            onExamplePromptSelect={selectExamplePrompt}
            onOpenProcessPanel={openProcessPanel}
            runtimeEvents={chat.runtimeEvents}
            selectedStrategy={chat.selectedStrategy}
            showProcessPanelAction
          />
          <AgentChatComposer
            attachments={chat.attachments}
            availableCampaigns={chat.availableCampaigns}
            availableProducts={chat.availableProducts}
            availableSources={chat.availableSources}
            canSend={chat.canSend}
            error={chat.error}
            input={chat.input}
            focusVersion={
              composerFocusVersion + requestedComposerPanel.focusVersion
            }
            requestedPanel={requestedComposerPanel.panel}
            requestedPanelVersion={requestedComposerPanel.version}
            isLoggedIn={chat.isLoggedIn}
            isLoadingCampaigns={chat.isLoadingCampaigns}
            isLoadingPlaybooks={chat.isLoadingPlaybooks}
            isLoadingProducts={chat.isLoadingProducts}
            isLoadingRuntimeOptions={chat.isLoadingRuntimeOptions}
            isLoadingSourceLibrary={chat.isLoadingSourceLibrary}
            isStreaming={chat.isStreaming}
            onInputChange={chat.setInput}
            onRuntimeCollaborationModeChange={chat.setRuntimeCollaborationMode}
            onRuntimeLearningLevelChange={chat.setRuntimeLearningLevel}
            onRuntimePrimaryRoleChange={chat.setRuntimePrimaryRole}
            onRuntimeRoutingModeChange={chat.setRuntimeRoutingMode}
            onToggleRuntimeRole={chat.toggleRuntimeRole}
            onSelectPlaybook={chat.selectPlaybook}
            onSend={sendMessage}
            onStop={chat.stopStreaming}
            onStrategyChange={chat.setStrategyId}
            onClearCampaignContext={chat.clearSelectedCampaigns}
            onClearProductContext={chat.clearSelectedProducts}
            onClearSourceContext={chat.clearSelectedSources}
            onToggleCampaignContext={chat.toggleCampaignContext}
            onToggleProductContext={chat.toggleProductContext}
            onToggleSourceContext={chat.toggleSourceContext}
            onDeleteSelectedSources={() => void chat.deleteSelectedSources()}
            onAttachExistingSource={(fileId) =>
              void chat.attachExistingSource(fileId)
            }
            onUploadAttachments={chat.uploadAttachments}
            selectedCampaigns={chat.selectedCampaigns}
            selectedProducts={chat.selectedProducts}
            selectedSources={chat.selectedSources}
            sourceLibrary={chat.sourceLibrary}
            sourceLibraryError={chat.sourceLibraryError}
            playbooks={chat.playbooks}
            runtimeOptions={chat.runtimeOptions}
            runtimeSelection={chat.runtimeSelection}
            strategyId={chat.strategyId}
          />
        </section>
      </div>
      {shouldShowProcessPanel ? (
        <AgentProcessDrawer
          anchorLabel={visibleProcessTarget?.anchorLabel}
          isStreaming={activeProcessMessage?.status === "streaming" && chat.isStreaming}
          messageId={activeProcessMessage?.id}
          onClose={closeProcessPanel}
          runtimeEvents={activeRuntimeEvents}
          task={visibleProcessTarget?.task}
        />
      ) : null}
    </div>
  )
}
