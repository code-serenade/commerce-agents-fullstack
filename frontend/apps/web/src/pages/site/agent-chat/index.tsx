import { useState } from "react"
import { useNavigate } from "react-router"
import type { AgentRuntimeEvent } from "./agent-chat-data"
import { AgentChatComposer } from "./components/agent-chat-composer"
import { AgentProcessDrawer } from "./components/agent-process-drawer"
import { AgentChatStatus } from "./components/agent-chat-status"
import { AgentChatThread } from "./components/agent-chat-thread"
import { useAgentChatContext } from "./context/use-agent-chat-context"
import { isChatOnlyRuntime } from "./view-model/thread-inline-view-model"

interface ActiveProcessTarget {
  anchorLabel: string
  messageId: string
  task: string
}

export default function AgentChatPage() {
  const chat = useAgentChatContext()
  const navigate = useNavigate()
  const [activeProcessTarget, setActiveProcessTarget] =
    useState<ActiveProcessTarget | null>(null)
  const activeProcessMessage = activeProcessTarget
    ? chat.messages.find((message) => message.id === activeProcessTarget.messageId)
    : null
  const activeRuntimeEvents: AgentRuntimeEvent[] =
    activeProcessMessage?.runtimeEvents ?? []
  const shouldShowProcessPanel =
    Boolean(activeProcessMessage && !isChatOnlyRuntime(activeRuntimeEvents))

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
            onCloseProcessPanel={() => setActiveProcessTarget(null)}
            onOpenProcessPanel={setActiveProcessTarget}
            runtimeEvents={chat.runtimeEvents}
            selectedStrategy={chat.selectedStrategy}
            showProcessPanelAction
          />
          <AgentChatComposer
            attachments={chat.attachments}
            availableProducts={chat.availableProducts}
            availableSources={chat.availableSources}
            canSend={chat.canSend}
            error={chat.error}
            input={chat.input}
            isLoggedIn={chat.isLoggedIn}
            isLoadingPlaybooks={chat.isLoadingPlaybooks}
            isLoadingProducts={chat.isLoadingProducts}
            isLoadingRuntimeOptions={chat.isLoadingRuntimeOptions}
            isStreaming={chat.isStreaming}
            onInputChange={chat.setInput}
            onRuntimeCollaborationModeChange={chat.setRuntimeCollaborationMode}
            onRuntimeLearningLevelChange={chat.setRuntimeLearningLevel}
            onRuntimeModelChange={chat.setRuntimeModelName}
            onRuntimePrimaryRoleChange={chat.setRuntimePrimaryRole}
            onToggleRuntimeRole={chat.toggleRuntimeRole}
            onSelectPlaybook={chat.selectPlaybook}
            onSend={chat.sendMessage}
            onStop={chat.stopStreaming}
            onStrategyChange={chat.setStrategyId}
            onClearProductContext={chat.clearSelectedProducts}
            onClearSourceContext={chat.clearSelectedSources}
            onToggleProductContext={chat.toggleProductContext}
            onToggleSourceContext={chat.toggleSourceContext}
            onUploadAttachments={chat.uploadAttachments}
            selectedProducts={chat.selectedProducts}
            selectedSources={chat.selectedSources}
            playbooks={chat.playbooks}
            runtimeOptions={chat.runtimeOptions}
            runtimeSelection={chat.runtimeSelection}
            strategyId={chat.strategyId}
          />
        </section>
      </div>
      {shouldShowProcessPanel ? (
        <AgentProcessDrawer
          anchorLabel={activeProcessTarget?.anchorLabel}
          isStreaming={activeProcessMessage?.status === "streaming" && chat.isStreaming}
          messageId={activeProcessMessage?.id}
          onClose={() => setActiveProcessTarget(null)}
          runtimeEvents={activeRuntimeEvents}
          task={activeProcessTarget?.task}
        />
      ) : null}
    </div>
  )
}
