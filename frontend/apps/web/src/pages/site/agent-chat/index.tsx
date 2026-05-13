import { useState } from "react"
import { useNavigate } from "react-router"
import { AgentChatComposer } from "./components/agent-chat-composer"
import { AgentChatStatus } from "./components/agent-chat-status"
import { AgentChatSwarmPanel } from "./components/agent-chat-swarm-panel"
import { AgentChatThread } from "./components/agent-chat-thread"
import { CollapsedWorkbenchHandle } from "./components/collapsed-workbench-handle"
import { useAgentChatContext } from "./context/use-agent-chat-context"
import { isChatOnlyRuntime } from "./view-model/thread-inline-view-model"

export default function AgentChatPage() {
  const chat = useAgentChatContext()
  const navigate = useNavigate()
  const [isProcessPanelOpen, setIsProcessPanelOpen] = useState(false)
  const isChatOnly = isChatOnlyRuntime(chat.runtimeEvents)
  const shouldShowProcessPanel = isProcessPanelOpen && !isChatOnly
  const canExpandProcessPanel = !isProcessPanelOpen && !isChatOnly

  const layoutClassName = [
    "relative grid min-h-0 w-full flex-1 grid-cols-1 gap-3 p-2 sm:p-3",
    shouldShowProcessPanel
      ? "overflow-hidden min-[1180px]:grid-cols-[minmax(0,1fr)_minmax(22rem,0.42fr)] min-[1440px]:grid-cols-[minmax(0,1.1fr)_minmax(25rem,0.46fr)] min-[1680px]:grid-cols-[minmax(0,1.22fr)_minmax(28rem,0.5fr)]"
      : canExpandProcessPanel
        ? "overflow-hidden min-[1180px]:grid-cols-[minmax(0,1fr)_3.25rem]"
      : "overflow-hidden",
  ].join(" ")

  const chatPanelClassName = [
    "flex min-w-0 flex-col overflow-hidden rounded-lg bg-background",
    shouldShowProcessPanel
      ? "min-h-[31rem] min-[1280px]:min-h-0"
      : "mx-auto min-h-0 w-full max-w-5xl",
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
            isStreaming={chat.isStreaming}
            messages={chat.messages}
            processPanelOpen={shouldShowProcessPanel}
            runtimeEvents={chat.runtimeEvents}
            selectedStrategy={chat.selectedStrategy}
          />
          <AgentChatComposer
            availableProducts={chat.availableProducts}
            canSend={chat.canSend}
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
            onToggleProductContext={chat.toggleProductContext}
            selectedProducts={chat.selectedProducts}
            playbooks={chat.playbooks}
            runtimeOptions={chat.runtimeOptions}
            runtimeSelection={chat.runtimeSelection}
            strategyId={chat.strategyId}
          />
        </section>
        {shouldShowProcessPanel ? (
          <div className="min-h-0 min-w-0 overflow-hidden max-[1179px]:min-h-124">
            <AgentChatSwarmPanel
              feedback={chat.feedback}
              isStreaming={chat.isStreaming}
              messages={chat.messages}
              onCollapse={() => setIsProcessPanelOpen(false)}
              runtimeEvents={chat.runtimeEvents}
            />
          </div>
        ) : canExpandProcessPanel ? (
          <CollapsedWorkbenchHandle
            isStreaming={chat.isStreaming}
            onExpand={() => setIsProcessPanelOpen(true)}
            runtimeEvents={chat.runtimeEvents}
          />
        ) : null}
      </div>
    </div>
  )
}
