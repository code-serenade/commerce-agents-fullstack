import { CheckCircle2, Loader2, MonitorCog, Plus } from "lucide-react"
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
    <button
      type="button"
      onClick={onExpand}
      className="absolute right-3 top-3 z-30 flex size-12 items-center justify-center rounded-full border border-border bg-[#f6f6f3]/95 text-foreground shadow-xl shadow-black/10 backdrop-blur transition-colors hover:bg-[#eeeeeb] dark:bg-card/95 dark:hover:bg-muted min-[1280px]:right-4 min-[1280px]:top-4 min-[1500px]:size-14"
      title={label}
      aria-label={label}
    >
      <span className="absolute -bottom-0.5 -left-0.5 flex size-5 items-center justify-center rounded-full border border-background bg-background shadow-sm min-[1500px]:size-6">
        {isStreaming ? (
          <Loader2 className="size-3.5 animate-spin text-emerald-600" />
        ) : isDone ? (
          <CheckCircle2 className="size-3.5 text-emerald-600" />
        ) : (
          <MonitorCog className={hasFailure ? "size-3.5 text-destructive" : "size-3.5 text-muted-foreground"} />
        )}
      </span>
      <span className="absolute -bottom-1 -right-1 rounded-full border border-background bg-background px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground shadow-sm">
        {progress}
      </span>
      <Plus className="size-5 text-muted-foreground min-[1500px]:size-6" />
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
