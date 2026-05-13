import { CheckCircle2, Loader2, MonitorCog, PanelRightOpen } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import { readRuntimeNodeId } from "../view-model/agent-runtime-event-mapping"

export interface CollapsedWorkbenchHandleProps {
  isStreaming: boolean
  onExpand: () => void
  runtimeEvents: AgentRuntimeEvent[]
}

export function CollapsedWorkbenchHandle({
  isStreaming,
  onExpand,
  runtimeEvents,
}: CollapsedWorkbenchHandleProps) {
  const { t } = useTranslation()
  const progress = getCollapsedWorkbenchProgress(runtimeEvents)
  const hasFailure = runtimeEvents.some((event) =>
    event.type === "task_node_failed" ||
    event.type === "workspace_run_failed" ||
    event.type === "failed"
  )
  const isDone = runtimeEvents.some((event) =>
    event.type === "task_graph_finished" ||
    event.type === "workspace_run_completed" ||
    event.type === "finished"
  )
  const title = t("agentChat.workbench.title")
  const label = `${t("agentChat.workbench.expand")} ${progress}`

  return (
    <div className="pointer-events-none absolute inset-y-0 right-2 z-20 flex items-center max-[1179px]:hidden">
      <button
        type="button"
        onClick={onExpand}
        className="pointer-events-auto grid w-11 justify-items-center gap-2 rounded-full border border-border bg-background/95 px-1.5 py-3 text-foreground shadow-lg shadow-black/10 backdrop-blur transition-colors hover:bg-muted dark:bg-card/95"
        title={label}
        aria-label={label}
      >
        <span className="flex size-8 shrink-0 items-center justify-center rounded-full border border-border bg-background shadow-sm">
          {isStreaming ? (
            <Loader2 className="size-4 animate-spin text-emerald-600" />
          ) : isDone ? (
            <CheckCircle2 className="size-4 text-emerald-600" />
          ) : (
            <MonitorCog
              className={
                hasFailure ? "size-4 text-destructive" : "size-4 text-muted-foreground"
              }
            />
          )}
        </span>
        <span className="text-[11px]/3 font-medium text-muted-foreground [writing-mode:vertical-rl]">
          {title}
        </span>
        <span className="rounded-full bg-muted px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
          {progress}
        </span>
        <PanelRightOpen className="size-4 text-muted-foreground" />
      </button>
    </div>
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
      doneNodes.add(readRuntimeNodeId(event))
      return
    }
    if (event.type === "task_node_evaluated" && event.evaluation.passed) {
      doneNodes.add(readEventId(event.evaluation.node_id))
      return
    }
    if (event.type === "task_node_passed") {
      doneNodes.add(readRuntimeNodeId(event))
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
