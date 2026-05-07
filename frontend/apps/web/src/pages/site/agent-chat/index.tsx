import { useState } from "react"
import { AgentChatComposer } from "./components/agent-chat-composer"
import { AgentChatStatus } from "./components/agent-chat-status"
import { AgentChatSwarmPanel } from "./components/agent-chat-swarm-panel"
import { AgentChatThread } from "./components/agent-chat-thread"
import { CollapsedWorkbenchHandle } from "./components/collapsed-workbench-handle"
import { useAgentChatContext } from "./context/use-agent-chat-context"

export default function AgentChatPage() {
  const chat = useAgentChatContext()
  const [isProcessPanelOpen, setIsProcessPanelOpen] = useState(true)

  const layoutClassName = [
    "relative grid min-h-0 w-full flex-1 grid-cols-1 gap-3 p-2 sm:p-3",
    isProcessPanelOpen
      ? "overflow-hidden min-[1280px]:grid-cols-[minmax(31rem,0.52fr)_minmax(28rem,0.48fr)] min-[1440px]:grid-cols-[minmax(35rem,0.94fr)_minmax(38rem,1.06fr)] min-[1680px]:grid-cols-[minmax(39rem,0.92fr)_minmax(46rem,1.08fr)]"
      : "overflow-hidden pr-14 min-[1280px]:pr-16",
  ].join(" ")

  const chatPanelClassName = [
    "flex min-w-0 flex-col overflow-hidden rounded-lg bg-background",
    isProcessPanelOpen
      ? "min-h-[31rem] min-[1280px]:min-h-0"
      : "mx-auto min-h-0 w-full max-w-5xl",
  ].join(" ")

  return (
    <div className="flex max-h-full min-h-0 size-full flex-1 flex-col overflow-hidden bg-background text-foreground">
      <AgentChatStatus error={chat.error} />
      <div className={layoutClassName}>
        <section className={chatPanelClassName}>
          <AgentChatThread
            isStreaming={chat.isStreaming}
            messages={chat.messages}
            processPanelOpen={isProcessPanelOpen}
            runtimeEvents={chat.runtimeEvents}
          />
          <AgentChatComposer
            canSend={chat.canSend}
            disabled={!chat.isLoggedIn}
            input={chat.input}
            isStreaming={chat.isStreaming}
            onInputChange={chat.setInput}
            onSend={chat.sendMessage}
            onStop={chat.stopStreaming}
          />
        </section>
        {isProcessPanelOpen ? (
          <div className="absolute inset-y-2 right-2 z-20 w-[min(46rem,calc(100%-1rem))] shadow-2xl shadow-black/20 min-[1280px]:static min-[1280px]:h-full min-[1280px]:max-h-full min-[1280px]:min-h-0 min-[1280px]:w-auto min-[1280px]:shadow-none">
            <AgentChatSwarmPanel
              isStreaming={chat.isStreaming}
              messages={chat.messages}
              onCollapse={() => setIsProcessPanelOpen(false)}
              runtimeEvents={chat.runtimeEvents}
            />
          </div>
        ) : (
          <CollapsedWorkbenchHandle
            isStreaming={chat.isStreaming}
            onExpand={() => setIsProcessPanelOpen(true)}
            runtimeEvents={chat.runtimeEvents}
          />
        )}
      </div>
    </div>
  )
}
