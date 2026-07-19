import { useEffect, useRef, useState } from "react"
import { AgentChatConversationArea } from "./components/agent-chat-conversation-area"
import { AgentChatDocumentTitle } from "./components/agent-chat-document-title"
import { AgentChatQuotaStatus } from "./components/agent-chat-quota-status"
import { AgentProcessDrawer } from "./components/agent-process-drawer"
import { AgentChatStatus } from "./components/agent-chat-status"
import { useAgentChatContext } from "./context/use-agent-chat-context"
import { useAgentChatRequestedComposerPanel } from "./view-model/use-agent-chat-requested-composer-panel"
import { useAgentChatAnalytics } from "./view-model/use-agent-chat-analytics"
import { useAgentProcessPanel } from "./view-model/use-agent-process-panel"
import { useAuthDialog } from "@/components/auth/AuthDialogProvider"
import { useAgentChatReportActions } from "./view-model/use-agent-chat-report-actions"

export default function AgentChatPage() {
  const chat = useAgentChatContext()
  const chatRef = useRef(chat)
  const { requestLogin } = useAuthDialog()
  const [composerFocusVersion, setComposerFocusVersion] = useState(0)
  const requestedComposerPanel = useAgentChatRequestedComposerPanel()
  const isWelcomeMode =
    chat.messages.length === 1 && chat.messages[0]?.id === "welcome"
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
  const reportActions = useAgentChatReportActions({
    conversationId: chat.conversationId,
    isStreaming: chat.isStreaming,
    runtimeEvents: chat.runtimeEvents,
  })

  useEffect(() => {
    chatRef.current = chat
  }, [chat])

  const sendMessage = () => {
    if (!chatRef.current.isLoggedIn) {
      requestLogin()
      return
    }
    analytics.trackQuestionSubmit()
    processPanel.closeProcessPanel()
    void chatRef.current.sendMessage()
  }

  const uploadAttachments = (files: File[]) => {
    analytics.trackFileUpload(files)
    void chat.uploadAttachments(files)
  }

  const selectExamplePrompt = (prompt: string) => {
    chat.setInput(prompt)
    setComposerFocusVersion((current) => current + 1)
  }

  const submitNextActionPrompt = (prompt: string) => {
    chat.setInput(prompt)
    window.setTimeout(sendMessage, 0)
  }

  return (
    <div className="relative flex size-full max-h-full min-h-0 min-w-0 flex-1 flex-col overflow-hidden text-foreground">
      <AgentChatDocumentTitle />
      <AgentChatStatus
        error={chat.error}
        loginRequired={!chat.isLoggedIn}
        onLogin={requestLogin}
      />
      {chat.isLoggedIn && isWelcomeMode ? (
        <div className="pointer-events-none absolute top-3 left-3 z-30 max-w-[calc(100%-1.5rem)] sm:top-5 sm:left-7 sm:max-w-[calc(100%-3.5rem)]">
          <div className="pointer-events-auto">
            <AgentChatQuotaStatus
              isLoading={chat.isLoadingQuota}
              quota={chat.quota}
            />
          </div>
        </div>
      ) : null}
      <AgentChatConversationArea
        chat={chat}
        composerFocusVersion={composerFocusVersion}
        onExamplePromptSelect={selectExamplePrompt}
        onNextActionPromptSubmit={submitNextActionPrompt}
        onSendMessage={sendMessage}
        onUploadAttachments={uploadAttachments}
        processPanel={processPanel}
        reportActions={reportActions}
        requestedComposerPanel={requestedComposerPanel}
      />
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
