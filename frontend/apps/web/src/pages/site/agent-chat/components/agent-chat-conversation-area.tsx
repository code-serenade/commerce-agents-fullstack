import { AgentChatComposer } from "./agent-chat-composer"
import { AgentChatThread } from "./agent-chat-thread"
import { AgentChatWelcomeContent } from "./agent-chat-welcome-content"
import { useTranslation } from "react-i18next"
import type { AgentChatContextValue } from "../context/agent-chat-context-value"
import type { useAgentChatRequestedComposerPanel } from "../view-model/use-agent-chat-requested-composer-panel"
import type { useAgentProcessPanel } from "../view-model/use-agent-process-panel"

export function AgentChatConversationArea({
  chat,
  composerFocusVersion,
  onExamplePromptSelect,
  onNextActionPromptSubmit,
  onSendMessage,
  onUploadAttachments,
  processPanel,
  requestedComposerPanel,
}: {
  chat: AgentChatContextValue
  composerFocusVersion: number
  onExamplePromptSelect: (prompt: string) => void
  onNextActionPromptSubmit: (prompt: string) => void
  onSendMessage: () => void
  onUploadAttachments: (files: File[]) => void
  processPanel: ReturnType<typeof useAgentProcessPanel>
  requestedComposerPanel: ReturnType<typeof useAgentChatRequestedComposerPanel>
}) {
  const { t } = useTranslation()
  const isWelcomeMode =
    chat.messages.length === 1 && chat.messages[0]?.id === "welcome"

  return (
    <div
      className={
        isWelcomeMode
          ? "relative z-10 flex min-h-0 w-full flex-1 flex-col overflow-y-auto px-4 pt-[24vh] pb-8 sm:px-7"
          : "relative z-10 flex min-h-0 w-full flex-1 flex-col overflow-hidden"
      }
    >
      {isWelcomeMode ? null : (
        <AgentChatThread
          activeProcessMessageId={processPanel.activeProcessMessage?.id ?? null}
          isStreaming={chat.isStreaming}
          messages={chat.messages}
          onCloseProcessPanel={processPanel.closeProcessPanel}
          onExamplePromptSelect={onExamplePromptSelect}
          onNextActionPromptSubmit={onNextActionPromptSubmit}
          onOpenProcessPanel={processPanel.openProcessPanel}
          runtimeEvents={chat.runtimeEvents}
          selectedStrategy={chat.selectedStrategy}
          showProcessPanelAction
        />
      )}
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
        onSend={onSendMessage}
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
        onUploadAttachments={onUploadAttachments}
        selectedCampaigns={chat.selectedCampaigns}
        selectedProducts={chat.selectedProducts}
        selectedSources={chat.selectedSources}
        sourceLibrary={chat.sourceLibrary}
        sourceLibraryError={chat.sourceLibraryError}
        playbooks={chat.playbooks}
        runtimeOptions={chat.runtimeOptions}
        runtimeSelection={chat.runtimeSelection}
        strategyId={chat.strategyId}
        heading={
          isWelcomeMode ? t("agentChat.composer.welcomeHeading") : undefined
        }
        placement={isWelcomeMode ? "welcome" : "bottom"}
        placeholder={
          isWelcomeMode ? t("agentChat.composer.welcomePlaceholder") : undefined
        }
      >
        {isWelcomeMode ? (
          <AgentChatWelcomeContent
            onSelectExamplePrompt={onNextActionPromptSubmit}
          />
        ) : null}
      </AgentChatComposer>
    </div>
  )
}
