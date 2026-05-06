import { useState } from "react"
import { MonitorCog, PanelRightOpen } from "lucide-react"
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
      : "overflow-hidden min-[1100px]:grid-cols-[minmax(0,1fr)_5rem]",
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
    <>
      <aside className="hidden min-h-0 border-l border-border/80 bg-[#f7f7f5]/90 pl-2 dark:bg-card/40 min-[1100px]:flex">
        <button
          type="button"
          onClick={onExpand}
          className="group mt-1 flex h-[calc(100%-0.5rem)] w-full flex-col items-center rounded-lg border border-border bg-background/95 px-2 py-3 text-center text-xs text-muted-foreground shadow-sm shadow-black/5 backdrop-blur transition-colors hover:bg-card hover:text-foreground"
          title="展开过程面板"
          aria-label="展开过程面板"
        >
          <span className="relative flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-card text-foreground">
            <MonitorCog className="size-4" />
            <span className="absolute -right-0.5 -top-0.5 size-2 rounded-full bg-emerald-500 shadow-[0_0_0_3px_rgba(16,185,129,0.14)]" />
          </span>
          <span className="mt-3 whitespace-nowrap text-[11px] font-medium text-foreground">
            展开视窗
          </span>
          <span className="mt-1 whitespace-nowrap text-[10px] text-muted-foreground">
            执行台
          </span>
          <span className="mt-3 h-px w-8 bg-border" />
          <span className="mt-3 flex size-8 items-center justify-center rounded-full border border-border bg-card transition-colors group-hover:border-foreground/30">
            <PanelRightOpen className="size-4" />
          </span>
          <span className="mt-auto hidden text-[10px] leading-4 text-muted-foreground min-[1300px]:block">
            过程视窗已收起
          </span>
        </button>
      </aside>
      <aside className="pointer-events-none absolute right-2 top-4 z-30 flex justify-end min-[1100px]:hidden">
      <button
        type="button"
        onClick={onExpand}
        className="pointer-events-auto flex items-center gap-2 rounded-full border border-border bg-background/95 px-2.5 py-2 text-xs text-muted-foreground shadow-lg shadow-black/10 backdrop-blur transition-colors hover:bg-card hover:text-foreground"
        title="展开过程面板（小屏）"
        aria-label="展开过程面板（小屏）"
      >
        <span className="flex size-7 shrink-0 items-center justify-center rounded-full border border-border bg-card">
          <MonitorCog className="size-3.5" />
        </span>
        <span className="hidden min-w-0 flex-col items-start leading-none sm:flex">
          <span className="font-medium text-foreground">展开视窗</span>
          <span className="mt-1 text-[10px] text-muted-foreground">执行台</span>
        </span>
        <span className="size-2 rounded-full bg-emerald-500 shadow-[0_0_0_3px_rgba(16,185,129,0.14)]" />
        <PanelRightOpen className="size-4 shrink-0" />
      </button>
      </aside>
    </>
  )
}
