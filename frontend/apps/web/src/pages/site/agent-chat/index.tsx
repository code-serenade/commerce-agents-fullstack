import { useState } from "react"
import { CheckCircle2, MonitorCog, PanelRightOpen } from "lucide-react"
import type { AgentRuntimeEvent } from "@/api"
import { AgentChatComposer } from "./components/agent-chat-composer"
import { AgentChatStatus } from "./components/agent-chat-status"
import { AgentChatSwarmPanel } from "./components/agent-chat-swarm-panel"
import { AgentChatThread } from "./components/agent-chat-thread"
import { useAgentChatContext } from "./context/use-agent-chat-context"
import type { AgentChatMessage } from "./types"

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
  const collapsedSummary = getCollapsedProcessSummary({
    isStreaming: chat.isStreaming,
    messages: chat.messages,
    runtimeEvents: chat.runtimeEvents,
  })

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
          {!isProcessPanelOpen ? (
            <CollapsedProcessSummary
              onExpand={() => setIsProcessPanelOpen(true)}
              summary={collapsedSummary}
            />
          ) : null}
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
      </div>
    </div>
  )
}

interface CollapsedSummary {
  completed: number
  stage: string
  status: "idle" | "running" | "done" | "blocked"
  total: number
}

function CollapsedProcessSummary({
  onExpand,
  summary,
}: {
  onExpand: () => void
  summary: CollapsedSummary
}) {
  const isDone = summary.status === "done"
  const isRunning = summary.status === "running"
  const progressPercent = summary.total
    ? Math.min(100, Math.round((summary.completed / summary.total) * 100))
    : 0
  return (
    <div className="shrink-0 bg-background px-3 pb-2 sm:px-5">
      <div className="mx-auto w-full max-w-3xl">
        <div className="flex min-w-0 items-center gap-3 rounded-full border border-border bg-[#f7f7f4]/95 px-3 py-2 shadow-sm shadow-black/5 backdrop-blur dark:bg-card">
          <span className="flex size-9 shrink-0 items-center justify-center rounded-full border border-border bg-background text-foreground">
            <MonitorCog className="size-4" />
          </span>
          <div className="min-w-0 flex-1">
            <div className="flex min-w-0 items-center gap-2 text-sm">
              <span
                className={`flex size-5 items-center justify-center rounded-full ${
                  isDone
                    ? "bg-emerald-50 text-emerald-600"
                    : isRunning
                      ? "bg-emerald-50 text-emerald-600"
                      : "bg-secondary text-secondary-foreground"
                }`}
              >
                <CheckCircle2
                  className={`size-4 ${isRunning ? "animate-pulse" : ""}`}
                />
              </span>
              <span className="shrink-0 font-medium text-foreground">
                当前进度 {summary.completed}/{summary.total}
              </span>
              <span className="hidden text-muted-foreground sm:inline">|</span>
              <span className="min-w-0 truncate text-muted-foreground">
                {summary.stage}
              </span>
            </div>
            <div className="mt-1 h-1 overflow-hidden rounded-full bg-muted">
              <span
                className={`block h-full rounded-full ${
                  summary.status === "blocked" ? "bg-destructive" : "bg-emerald-500"
                }`}
                style={{ width: `${progressPercent}%` }}
              />
            </div>
          </div>
          <div className="hidden shrink-0 items-center gap-1.5 text-[11px] text-muted-foreground min-[940px]:flex">
            <span>商品</span>
            <span className="size-1 rounded-full bg-muted-foreground/35" />
            <span>活动</span>
            <span className="size-1 rounded-full bg-muted-foreground/35" />
            <span>数据</span>
            <span className="size-1 rounded-full bg-muted-foreground/35" />
            <span>预警</span>
          </div>
          <button
            type="button"
            onClick={onExpand}
            className="flex h-10 shrink-0 items-center justify-center gap-2 rounded-full bg-foreground px-4 text-sm font-medium text-background transition-colors hover:bg-foreground/90"
            title="展开执行视窗"
            aria-label="展开执行视窗"
          >
            展开视窗
            <PanelRightOpen className="size-4" />
          </button>
        </div>
      </div>
    </div>
  )
}

function getCollapsedProcessSummary({
  isStreaming,
  messages,
  runtimeEvents,
}: {
  isStreaming: boolean
  messages: AgentChatMessage[]
  runtimeEvents: AgentRuntimeEvent[]
}): CollapsedSummary {
  const hasTask = messages.some((message) => message.role === "user")
  const graph = getLatestGraph(runtimeEvents)
  const nodeIds = graph
    ? [...graph.nodes.map((node) => readId(node.id)), "delivery"]
    : ["data", "accounting", "douyin_ads", "creative", "ops", "delivery"]
  const total = nodeIds.length
  const doneNodes = new Set<string>()
  let hasBlockedNode = false

  runtimeEvents.forEach((event) => {
    if (event.type === "task_node_passed") doneNodes.add(event.nodeId)
    if (event.type === "task_node_reported" && event.report.status !== "failed") {
      doneNodes.add(readId(event.report.node_id))
    }
    if (event.type === "task_node_failed") hasBlockedNode = true
  })

  const completed = Math.min(
    total,
    nodeIds.filter((nodeId) => doneNodes.has(nodeId)).length
  )

  if (hasBlockedNode) {
    return {
      completed,
      stage: "执行受阻，展开视窗查看待处理项",
      status: "blocked",
      total,
    }
  }
  if (runtimeEvents.some((event) => event.type === "task_graph_finished")) {
    return {
      completed: total,
      stage: "方案已汇总，可查看执行过程和业务落点",
      status: "done",
      total,
    }
  }
  if (isStreaming) {
    return {
      completed,
      stage: graph ? "协作流程执行中" : "正在规划协作流程",
      status: "running",
      total,
    }
  }
  if (hasTask) {
    return {
      completed,
      stage: "历史会话，展开视窗查看本轮过程",
      status: "idle",
      total,
    }
  }

  return {
    completed: 0,
    stage: "等待业务目标输入",
    status: "idle",
    total,
  }
}

function getLatestGraph(runtimeEvents: AgentRuntimeEvent[]) {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === "task_graph_planned") return event.graph
  }
  return null
}

function readId(value: string | [string] | { 0: string }) {
  if (typeof value === "string") return value
  if (Array.isArray(value)) return value[0]
  return value[0]
}
