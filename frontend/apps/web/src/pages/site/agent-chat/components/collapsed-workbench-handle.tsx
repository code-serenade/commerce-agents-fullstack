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
    <button
      type="button"
      onClick={onExpand}
      className="absolute inset-y-2 right-2 z-30 flex w-12 flex-col items-center justify-between overflow-hidden rounded-[1.35rem] border border-border bg-[#f6f6f3]/95 px-1.5 py-3 text-sm font-medium text-foreground shadow-xl shadow-black/10 backdrop-blur transition-colors hover:bg-[#eeeeeb] dark:bg-card/95 dark:hover:bg-muted min-[1280px]:right-3 min-[1280px]:w-14"
      title={label}
      aria-label={label}
    >
      <span className="flex size-9 shrink-0 items-center justify-center rounded-lg border border-border bg-background text-muted-foreground shadow-sm">
        {isStreaming ? (
          <Loader2 className="size-4 animate-spin text-emerald-600" />
        ) : isDone ? (
          <CheckCircle2 className="size-4 text-emerald-600" />
        ) : (
          <MonitorCog className={hasFailure ? "size-4 text-destructive" : "size-4"} />
        )}
      </span>
      <span className="flex min-h-0 flex-1 items-center justify-center [writing-mode:vertical-rl] text-xs tracking-[0.18em]">
        {t("agentChat.workbench.title")}
      </span>
      <span className="grid gap-1">
        {Array.from({ length: 6 }).map((_, index) => (
          <span
            key={index}
            className={`h-1 w-5 rounded-full ${
              isStreaming && index < 2
                ? "bg-emerald-500"
                : isDone
                  ? "bg-emerald-500"
                  : hasFailure && index === 0
                    ? "bg-destructive"
                    : "bg-muted-foreground/20"
            }`}
          />
        ))}
      </span>
      <span className="font-mono text-[11px] text-muted-foreground">{progress}</span>
      <span className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-background text-muted-foreground shadow-sm">
        <PanelRightOpen className="size-4" />
      </span>
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
