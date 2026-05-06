import { useState } from "react"
import { CheckCircle2, Loader2, MonitorCog, PanelRightOpen } from "lucide-react"
import type { AgentRuntimeEvent } from "@/api"
import { AgentChatComposer } from "./components/agent-chat-composer"
import { AgentChatStatus } from "./components/agent-chat-status"
import { AgentChatSwarmPanel } from "./components/agent-chat-swarm-panel"
import { AgentChatThread } from "./components/agent-chat-thread"
import { useAgentChatContext } from "./context/use-agent-chat-context"

export default function AgentChatPage() {
  const chat = useAgentChatContext()
  const [isProcessPanelOpen, setIsProcessPanelOpen] = useState(true)

  const layoutClassName = [
    "relative grid min-h-0 w-full flex-1 grid-cols-1 gap-3 p-2 sm:p-3",
    isProcessPanelOpen
      ? "overflow-hidden min-[1280px]:grid-cols-[minmax(31rem,0.52fr)_minmax(28rem,0.48fr)] min-[1440px]:grid-cols-[minmax(35rem,0.94fr)_minmax(38rem,1.06fr)] min-[1680px]:grid-cols-[minmax(39rem,0.92fr)_minmax(46rem,1.08fr)]"
      : "overflow-hidden",
  ].join(" ")

  const chatPanelClassName = [
    "flex min-w-0 flex-col overflow-hidden rounded-lg bg-background",
    isProcessPanelOpen
      ? "min-h-[31rem] min-[1280px]:min-h-0"
      : "mx-auto min-h-0 w-full max-w-5xl",
  ].join(" ")

  return (
    <div className="flex h-full max-h-full min-h-0 w-full flex-1 flex-col overflow-hidden bg-background text-foreground">
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
            llmConfig={chat.llmConfig}
            onInputChange={chat.setInput}
            onModelChange={chat.setSelectedModelName}
            onSend={chat.sendMessage}
            onStop={chat.stopStreaming}
            selectedModelName={chat.selectedModelName}
          />
        </section>
        {isProcessPanelOpen ? (
          <div className="absolute inset-y-2 right-2 z-20 w-[min(46rem,calc(100%_-_1rem))] shadow-2xl shadow-black/20 min-[1280px]:static min-[1280px]:h-full min-[1280px]:max-h-full min-[1280px]:min-h-0 min-[1280px]:w-auto min-[1280px]:shadow-none">
            <AgentChatSwarmPanel
              isStreaming={chat.isStreaming}
              llmConfig={chat.llmConfig}
              messages={chat.messages}
              onCollapse={() => setIsProcessPanelOpen(false)}
              runtimeEvents={chat.runtimeEvents}
              selectedModelName={chat.selectedModelName}
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

function CollapsedWorkbenchHandle({
  isStreaming,
  onExpand,
  runtimeEvents,
}: {
  isStreaming: boolean
  onExpand: () => void
  runtimeEvents: AgentRuntimeEvent[]
}) {
  const progress = getCollapsedWorkbenchProgress(runtimeEvents)
  const hasFailure = runtimeEvents.some((event) => event.type === "task_node_failed")
  const isDone = runtimeEvents.some((event) => event.type === "task_graph_finished")

  return (
    <button
      type="button"
      onClick={onExpand}
      className="absolute right-3 top-3 z-30 flex items-center gap-2 rounded-full border border-border bg-background/95 px-3 py-2 text-sm font-medium text-foreground shadow-lg shadow-black/10 backdrop-blur transition-colors hover:bg-muted"
      title="展开工作台"
      aria-label="展开工作台"
    >
      <span className="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground">
        {isStreaming ? (
          <Loader2 className="size-4 animate-spin text-emerald-600" />
        ) : isDone ? (
          <CheckCircle2 className="size-4 text-emerald-600" />
        ) : (
          <MonitorCog className={hasFailure ? "size-4 text-destructive" : "size-4"} />
        )}
      </span>
      <span className="hidden sm:inline">
        工作台 {progress}
      </span>
      <PanelRightOpen className="size-4 shrink-0 text-muted-foreground" />
    </button>
  )
}

function getCollapsedWorkbenchProgress(runtimeEvents: AgentRuntimeEvent[]) {
  const graphEvent = runtimeEvents.find(
    (event): event is Extract<AgentRuntimeEvent, { type: "task_graph_planned" }> =>
      event.type === "task_graph_planned"
  )
  const total = graphEvent ? graphEvent.graph.nodes.length + 1 : 6
  const doneNodes = new Set<string>()

  runtimeEvents.forEach((event) => {
    if (event.type === "task_node_reported" && event.report.status !== "failed") {
      doneNodes.add(event.nodeId)
      return
    }
    if (event.type === "task_node_evaluated" && event.evaluation.passed) {
      doneNodes.add(readEventId(event.evaluation.node_id))
      return
    }
    if (event.type === "task_node_passed") {
      doneNodes.add(event.nodeId)
      return
    }
    if (event.type === "task_graph_finished") {
      doneNodes.add("delivery")
    }
  })

  return `${Math.min(doneNodes.size, total)}/${total}`
}

function readEventId(value: string | [string] | { 0: string }) {
  if (typeof value === "string") return value
  if (Array.isArray(value)) return value[0]
  return value[0]
}
