import { useState } from "react"
import { AgentChatComposer } from "./components/agent-chat-composer"
import { AgentChatStatus } from "./components/agent-chat-status"
import { AgentChatSwarmPanel } from "./components/agent-chat-swarm-panel"
import { AgentChatThread } from "./components/agent-chat-thread"
import { useAgentChatContext } from "./context/use-agent-chat-context"

export default function AgentChatPage() {
  const chat = useAgentChatContext()
  const [isProcessPanelOpen, setIsProcessPanelOpen] = useState(true)

  const layoutClassName = [
    "relative grid min-h-0 flex-1 grid-cols-1 gap-3 p-2 sm:p-3",
    isProcessPanelOpen
      ? "overflow-hidden min-[1180px]:grid-cols-[minmax(26rem,0.9fr)_minmax(34rem,1.1fr)] min-[1500px]:grid-cols-[minmax(30rem,0.9fr)_minmax(42rem,1.1fr)]"
      : "overflow-hidden",
  ].join(" ")

  const chatPanelClassName = [
    "flex min-w-0 flex-col overflow-hidden rounded-lg bg-background",
    isProcessPanelOpen
      ? "min-h-[31rem] min-[1180px]:min-h-0"
      : "mx-auto min-h-0 w-full max-w-5xl",
  ].join(" ")

  return (
    <div className="flex h-full max-h-full min-h-0 flex-col overflow-hidden bg-background text-foreground">
      <AgentChatStatus error={chat.error} />
      <div className={layoutClassName}>
        <section className={chatPanelClassName}>
          <AgentChatThread
            isStreaming={chat.isStreaming}
            messages={chat.messages}
            onExpandProcessPanel={() => setIsProcessPanelOpen(true)}
            processPanelOpen={isProcessPanelOpen}
            runtimeEvents={chat.runtimeEvents}
          />
          <AgentChatComposer
            canSend={chat.canSend}
            disabled={!chat.isLoggedIn}
            input={chat.input}
            isStreaming={chat.isStreaming}
            llmConfig={chat.llmConfig}
            onInputChange={chat.setInput}
            onModelChange={chat.setSelectedModelName}
            onSend={chat.sendMessage}
            onStop={chat.stopStreaming}
            selectedModelName={chat.selectedModelName}
          />
        </section>
        {isProcessPanelOpen ? (
          <div className="absolute inset-y-2 right-2 z-20 w-[min(48rem,calc(100%_-_1rem))] shadow-2xl shadow-black/20 min-[1180px]:static min-[1180px]:h-full min-[1180px]:max-h-full min-[1180px]:min-h-0 min-[1180px]:w-auto min-[1180px]:shadow-none">
            <AgentChatSwarmPanel
              isStreaming={chat.isStreaming}
              llmConfig={chat.llmConfig}
              messages={chat.messages}
              onCollapse={() => setIsProcessPanelOpen(false)}
              runtimeEvents={chat.runtimeEvents}
              selectedModelName={chat.selectedModelName}
            />
          </div>
        ) : null}
      </div>
    </div>
  )
}
