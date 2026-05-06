import { useState } from "react"
import { Activity, ChevronLeft, PanelRightOpen } from "lucide-react"
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
      ? "overflow-hidden min-[1500px]:grid-cols-[minmax(30rem,0.9fr)_minmax(42rem,1.1fr)]"
      : "overflow-hidden",
  ].join(" ")

  const chatPanelClassName = [
    "flex min-w-0 flex-col overflow-hidden rounded-lg bg-background",
    isProcessPanelOpen
      ? "min-h-[31rem] min-[1500px]:min-h-0"
      : "mx-auto min-h-0 w-full max-w-5xl",
  ].join(" ")

  return (
    <div className="flex size-full min-h-0 flex-col overflow-hidden bg-background text-foreground">
      <AgentChatStatus error={chat.error} />
      <div className={layoutClassName}>
        <section className={chatPanelClassName}>
          <AgentChatThread
            isStreaming={chat.isStreaming}
            messages={chat.messages}
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
          <div className="absolute inset-y-2 right-2 z-20 w-[min(48rem,calc(100%_-_1rem))] shadow-2xl shadow-black/20 min-[1500px]:static min-[1500px]:w-auto min-[1500px]:shadow-none">
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
        {!isProcessPanelOpen ? (
          <CollapsedProcessHandle onExpand={() => setIsProcessPanelOpen(true)} />
        ) : null}
      </div>
    </div>
  )
}

function CollapsedProcessHandle({ onExpand }: { onExpand: () => void }) {
  return (
    <aside className="pointer-events-none absolute inset-y-0 right-0 z-30 flex items-center">
      <button
        type="button"
        onClick={onExpand}
        className="group pointer-events-auto flex min-h-28 w-10 flex-col items-center justify-center gap-2 rounded-l-xl border border-r-0 border-border bg-card/95 py-3 text-muted-foreground shadow-lg shadow-black/10 backdrop-blur transition-all hover:w-24 hover:bg-background hover:text-foreground"
        title="展开过程面板"
        aria-label="展开过程面板"
      >
        <PanelRightOpen className="size-4 shrink-0" />
        <span className="hidden whitespace-nowrap text-xs font-medium [writing-mode:vertical-rl] sm:block group-hover:hidden">
          过程
        </span>
        <span className="hidden whitespace-nowrap text-xs font-medium group-hover:block">
          展开过程
        </span>
        <div className="relative">
          <Activity className="size-4" />
          <span className="absolute -right-0.5 -top-0.5 size-1.5 rounded-full bg-emerald-500" />
        </div>
        <ChevronLeft className="hidden size-4 group-hover:block" />
      </button>
    </aside>
  )
}
