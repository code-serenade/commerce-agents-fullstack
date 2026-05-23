import { CheckCircle2, MonitorCog } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import { getAgentProcessProgress } from "../view-model/agent-process-kind-view-model"
import { getInlineAgents } from "../view-model/thread-inline-view-model"

export function InlineProcessStatusNode({
  isStreaming,
  runtimeEvents,
}: {
  isStreaming: boolean
  runtimeEvents: AgentRuntimeEvent[]
}) {
  const { t } = useTranslation()
  const agents = getInlineAgents(runtimeEvents, isStreaming, t)
  const completedCount = agents.filter((agent) => agent.state === "done").length
  const processProgress = getAgentProcessProgress(
    {
      completedCount,
      isStreaming,
      runtimeEvents,
      totalCount: agents.length,
    },
    t
  )
  const hasFailedAgent = agents.some((agent) => agent.state === "failed")

  return (
    <article className="flex justify-start gap-3">
      <div className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-card text-foreground">
        <MonitorCog className="size-5" />
      </div>
      <div className="max-w-[min(43rem,92%)] min-w-0 flex-1">
        <div className="border-y border-border/80 bg-background/95 py-2">
          <div className="flex min-w-0 items-center gap-3 px-1">
            <div className="min-w-0 flex-1">
              <div className="flex min-w-0 items-center gap-2 text-sm">
                <CheckCircle2
                  className={`size-4 shrink-0 text-emerald-600 ${
                    isStreaming ? "animate-pulse" : ""
                  }`}
                />
                <span className="shrink-0 font-medium text-foreground">
                  {t("agentChat.thread.currentProgress", {
                    completed: processProgress.completed,
                    total: processProgress.total,
                  })}
                </span>
                <span className="hidden text-muted-foreground sm:inline">|</span>
                <span className="min-w-0 truncate text-muted-foreground">
                  {processProgress.stage}
                </span>
              </div>
              <div className="mt-1.5 h-px overflow-hidden rounded-full bg-muted">
                <span
                  className={`block h-full rounded-full ${
                    hasFailedAgent ? "bg-destructive" : "bg-emerald-500"
                  }`}
                  style={{ width: `${processProgress.percent}%` }}
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </article>
  )
}
