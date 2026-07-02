import { useState } from "react"
import { useNavigate } from "react-router"
import { AgentChatComposer } from "./components/agent-chat-composer"
import { AgentChatDocumentTitle } from "./components/agent-chat-document-title"
import { AgentProcessDrawer } from "./components/agent-process-drawer"
import { AgentChatStatus } from "./components/agent-chat-status"
import { AgentChatQuotaStatus } from "./components/agent-chat-quota-status"
import { AgentChatThread } from "./components/agent-chat-thread"
import { useAgentChatContext } from "./context/use-agent-chat-context"
import { useAgentChatRequestedComposerPanel } from "./view-model/use-agent-chat-requested-composer-panel"
import { useAgentChatAnalytics } from "./view-model/use-agent-chat-analytics"
import { useAgentProcessPanel } from "./view-model/use-agent-process-panel"

export default function AgentChatPage() {
  const chat = useAgentChatContext()
  const navigate = useNavigate()
  const [composerFocusVersion, setComposerFocusVersion] = useState(0)
  const requestedComposerPanel = useAgentChatRequestedComposerPanel()
  const analytics = useAgentChatAnalytics({
    attachments: chat.attachments,
    selectedCampaigns: chat.selectedCampaigns,
    selectedProducts: chat.selectedProducts,
    selectedSources: chat.selectedSources,
    strategyId: chat.strategyId,
  })
  const processPanel = useAgentProcessPanel({
    messages: chat.messages,
    onOpen: analytics.trackProcessPanelOpen,
    runtimeEvents: chat.runtimeEvents,
  })

  const sendMessage = () => {
    analytics.trackQuestionSubmit()
    processPanel.closeProcessPanel()
    void chat.sendMessage()
  }

  const uploadAttachments = (files: File[]) => {
    analytics.trackFileUpload(files)
    void chat.uploadAttachments(files)
  }

  const selectExamplePrompt = (prompt: string) => {
    chat.setInput(prompt)
    setComposerFocusVersion((current) => current + 1)
  }

  return (
    <div className="flex max-h-full min-h-0 size-full flex-1 flex-col overflow-hidden bg-background text-foreground">
      <AgentChatDocumentTitle />
      <AgentChatStatus
        error={chat.error}
        loginRequired={!chat.isLoggedIn}
        onLogin={() => navigate("/login", { state: { from: { pathname: "/commerce" } } })}
      />
      {chat.isLoggedIn ? (
        <AgentChatQuotaStatus
          isLoading={chat.isLoadingQuota}
          quota={chat.quota}
        />
      ) : null}
      <div className="relative flex min-h-0 w-full flex-1 overflow-hidden p-2 sm:p-3">
        <section className="mx-auto flex size-full min-h-0 max-w-5xl min-w-0 flex-col overflow-hidden rounded-lg bg-background">
          <AgentChatThread
            activeProcessMessageId={processPanel.activeProcessMessage?.id ?? null}
            isStreaming={chat.isStreaming}
            messages={chat.messages}
            onCloseProcessPanel={processPanel.closeProcessPanel}
            onExamplePromptSelect={selectExamplePrompt}
            onOpenProcessPanel={processPanel.openProcessPanel}
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
            contextPreview={chat.contextPreview}
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
            onUploadAttachments={uploadAttachments}
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
      {processPanel.shouldShowProcessPanel ? (
        <AgentProcessDrawer
          anchorLabel={processPanel.visibleProcessTarget?.anchorLabel}
          isStreaming={
            processPanel.activeProcessMessage?.status === "streaming" &&
            chat.isStreaming
          }
          messageId={processPanel.activeProcessMessage?.id}
          onClose={processPanel.closeProcessPanel}
          runtimeEvents={processPanel.activeRuntimeEvents}
          task={processPanel.visibleProcessTarget?.task}
        />
      ) : null}
    </div>
  )
}
