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
      ? "overflow-hidden min-[1280px]:grid-cols-[minmax(34rem,1fr)_minmax(25rem,0.68fr)] min-[1440px]:grid-cols-[minmax(40rem,1.12fr)_minmax(32rem,0.88fr)] min-[1680px]:grid-cols-[minmax(48rem,1.18fr)_minmax(36rem,0.82fr)]"
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
          <div className="absolute inset-y-2 right-2 z-20 w-[min(42rem,calc(100%-1rem))] shadow-2xl shadow-black/20 min-[1280px]:static min-[1280px]:h-full min-[1280px]:max-h-full min-[1280px]:min-h-0 min-[1280px]:w-auto min-[1280px]:shadow-none">
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
