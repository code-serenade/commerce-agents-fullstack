import { CheckCircle2, Loader2, MonitorCog, PanelRightOpen } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"

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
  const hasFailure = runtimeEvents.some((event) => event.type === "task_node_failed")
  const isDone = runtimeEvents.some((event) => event.type === "task_graph_finished")
  const label = t("agentChat.workbench.expand")

  return (
    <div className="pointer-events-none absolute inset-y-2 right-2 z-30 flex items-start justify-end min-[1280px]:right-0">
      <button
        type="button"
        onClick={onExpand}
        className="pointer-events-auto mt-2 flex min-w-12 items-center gap-2 rounded-l-lg rounded-r-md border border-border bg-[#f6f6f3]/95 px-2.5 py-2 text-foreground shadow-xl shadow-black/10 backdrop-blur transition-colors hover:bg-[#eeeeeb] dark:bg-card/95 dark:hover:bg-muted min-[1280px]:rounded-l-lg min-[1280px]:rounded-r-none"
        title={label}
        aria-label={label}
      >
        <span className="flex size-8 shrink-0 items-center justify-center rounded-full bg-background shadow-sm">
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
        <span className="hidden font-mono text-[11px] text-muted-foreground min-[1500px]:block">
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
