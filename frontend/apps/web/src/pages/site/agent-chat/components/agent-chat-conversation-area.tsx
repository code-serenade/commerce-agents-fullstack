import { AgentChatComposer } from "./agent-chat-composer"
import { AgentChatQuotaStatus } from "./agent-chat-quota-status"
import { AgentChatReportActions } from "./agent-chat-report-actions"
import { AgentChatThread } from "./agent-chat-thread"
import {
  AgentChatWelcomePromptChips,
  AgentChatWelcomeRolePanel,
} from "./agent-chat-welcome-content"
import { useTranslation } from "react-i18next"
import { useLayoutEffect, useRef } from "react"
import type { AgentChatContextValue } from "../context/agent-chat-context-value"
import type { AgentChatReportActionsState } from "../view-model/use-agent-chat-report-actions"
import type { useAgentChatRequestedComposerPanel } from "../view-model/use-agent-chat-requested-composer-panel"
import type { useAgentProcessPanel } from "../view-model/use-agent-process-panel"

export function AgentChatConversationArea({
  chat,
  composerFocusVersion,
  onExamplePromptSelect,
  onNextActionPromptSubmit,
  onAnswerProbe,
  onSendMessage,
  onUploadAttachments,
  processPanel,
  reportActions,
  requestedComposerPanel,
}: {
  chat: AgentChatContextValue
  composerFocusVersion: number
  onExamplePromptSelect: (prompt: string) => void
  onNextActionPromptSubmit: (prompt: string) => void
  onAnswerProbe: (question: string) => void
  onSendMessage: () => void
  onUploadAttachments: (files: File[]) => void
  processPanel: ReturnType<typeof useAgentProcessPanel>
  reportActions: AgentChatReportActionsState
  requestedComposerPanel: ReturnType<typeof useAgentChatRequestedComposerPanel>
}) {
  const { t } = useTranslation()
  const isWelcomeMode =
    chat.messages.length === 1 && chat.messages[0]?.id === "welcome"
  const conversationAreaRef = useRef<HTMLDivElement>(null)

  useLayoutEffect(() => {
    const conversationArea = conversationAreaRef.current
    if (!isWelcomeMode && conversationArea?.scrollTop) {
      conversationArea.scrollTop = 0
    }
  })

  return (
    <div
      ref={conversationAreaRef}
      data-agent-chat-conversation-area={isWelcomeMode ? "welcome" : "thread"}
      onScroll={
        isWelcomeMode
          ? undefined
          : (event) => {
              if (event.currentTarget.scrollTop !== 0) {
                event.currentTarget.scrollTop = 0
              }
            }
      }
      className={
        isWelcomeMode
          ? "agent-chat-scrollarea relative z-10 flex min-h-0 min-w-0 w-full max-w-full flex-1 flex-col overflow-y-auto px-4 pt-[clamp(3.5rem,10svh,5rem)] pb-8 sm:px-7 lg:pt-[clamp(4rem,9svh,5.5rem)]"
          : "relative z-10 flex min-h-0 min-w-0 w-full max-w-full flex-1 flex-col overflow-clip"
      }
    >
      {isWelcomeMode ? null : (
        <>
          <AgentChatReportActions
            canUseConversation={reportActions.canUseConversation}
            isExportingTranscript={reportActions.isExportingTranscript}
            isGeneratingReport={reportActions.isGeneratingReport}
            isStreaming={chat.isStreaming}
            leading={
              chat.isLoggedIn ? (
                <AgentChatQuotaStatus
                  isLoading={chat.isLoadingQuota}
                  quota={chat.quota}
                />
              ) : undefined
            }
            onExportReport={reportActions.exportReport}
            onExportTranscript={reportActions.exportTranscript}
            onOpenReportCenter={reportActions.openReportCenter}
          />
          <AgentChatThread
            activeProcessMessageId={processPanel.activeProcessMessage?.id ?? null}
            conversationId={chat.conversationId ?? undefined}
            isStreaming={chat.isStreaming}
            messages={chat.messages}
            onCloseProcessPanel={processPanel.closeProcessPanel}
            onExamplePromptSelect={onExamplePromptSelect}
            onNextActionPromptSubmit={onNextActionPromptSubmit}
            onAnswerProbe={onAnswerProbe}
            onOpenProcessPanel={processPanel.openProcessPanel}
            runtimeEvents={chat.runtimeEvents}
            selectedStrategy={chat.selectedStrategy}
            showProcessPanelAction
          />
        </>
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
        beforeChildren={
          isWelcomeMode ? (
            <AgentChatWelcomePromptChips
              onSelectExamplePrompt={onExamplePromptSelect}
              surface="plain"
            />
          ) : undefined
        }
      >
        {isWelcomeMode ? <AgentChatWelcomeRolePanel surface="plain" /> : null}
      </AgentChatComposer>
    </div>
  )
}
