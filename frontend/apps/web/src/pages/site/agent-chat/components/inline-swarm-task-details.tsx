import { CheckCircle2, Loader2, MonitorCog } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import {
  getInlineAgents,
  getInlineProcessStage,
  shouldLightInlinePixel,
  type InlineAgent,
} from "../view-model/thread-inline-view-model"

export function InlineProgressSummary({
  completedCount,
  hasFailedAgent,
  hasRuntime,
  isStreaming,
  progressPercent,
  runtimeEvents,
  totalCount,
}: {
  completedCount: number
  hasFailedAgent: boolean
  hasRuntime: boolean
  isStreaming: boolean
  progressPercent: number
  runtimeEvents: AgentRuntimeEvent[]
  totalCount: number
}) {
  const { t } = useTranslation()

  return (
    <div className="rounded-lg border border-border bg-background px-3 py-2">
      <div className="flex min-w-0 items-center gap-2 text-xs">
        <CheckCircle2
          className={`size-4 shrink-0 text-emerald-600 ${
            isStreaming ? "animate-pulse" : ""
          }`}
        />
        <span className="shrink-0 font-medium text-foreground">
          {t("agentChat.thread.currentProgress", {
            completed: completedCount,
            total: totalCount,
          })}
        </span>
        <span className="hidden text-muted-foreground sm:inline">|</span>
        <span className="min-w-0 truncate text-muted-foreground">
          {getInlineProcessStage({ hasRuntime, isStreaming, runtimeEvents }, t)}
        </span>
      </div>
      <div className="mt-2 h-1 overflow-hidden rounded-full bg-muted">
        <span
          className={`block h-full rounded-full ${
            hasFailedAgent ? "bg-destructive" : "bg-emerald-500"
          }`}
          style={{ width: `${progressPercent}%` }}
        />
      </div>
    </div>
  )
}

export function InlineStepStrip({ steps }: { steps: string[][] }) {
  return (
    <div className="flex min-w-0 flex-wrap gap-1.5">
      {steps.map(([label, detail], index) => {
        return (
          <div
            key={label}
            className="flex min-w-0 max-w-full items-center gap-2 rounded-full border border-border bg-card px-2.5 py-1.5 shadow-sm"
          >
            <span className="shrink-0 font-mono text-[11px] font-semibold text-muted-foreground">
              {String(index + 1).padStart(2, "0")}
            </span>
            <p className="truncate text-xs font-medium">{label}</p>
            <span className="hidden min-w-0 truncate text-xs text-muted-foreground sm:inline">
              {detail}
            </span>
          </div>
        )
      })}
    </div>
  )
}

export function InlineRuntimeBadge({
  hasRuntime,
  isStreaming,
}: {
  hasRuntime: boolean
  isStreaming: boolean
}) {
  const { t } = useTranslation()

  return (
    <span className="rounded-full bg-secondary px-2 py-1 text-[11px] font-medium text-secondary-foreground">
      {hasRuntime
        ? isStreaming
          ? t("agentChat.thread.statuses.executing")
          : t("agentChat.thread.statuses.synced")
        : isStreaming
          ? t("agentChat.thread.statuses.planning")
          : t("agentChat.thread.statuses.waiting")}
    </span>
  )
}

export function InlineAgentGrid({
  agents,
  compact = false,
}: {
  agents: InlineAgent[]
  compact?: boolean
}) {
  const visibleAgents = compact ? agents.slice(0, 3) : agents.slice(0, 6)

  return (
    <div className="grid gap-1.5">
      {visibleAgents.map((agent, index) => (
        <div
          key={agent.nodeId}
          className={`flex min-w-0 items-center gap-2 rounded-md px-2.5 py-2 ${
            agent.state === "working" || agent.state === "reworking"
              ? "bg-background ring-1 ring-foreground/25"
              : agent.state === "failed"
                ? "bg-destructive/10"
                : "bg-background/80"
          }`}
        >
          <div className="flex size-7 shrink-0 items-center justify-center rounded-full border border-border bg-background">
            {agent.state === "working" || agent.state === "reworking" ? (
              <Loader2 className="size-4 animate-spin text-emerald-600" />
            ) : agent.state === "done" ? (
              <CheckCircle2 className="size-4 text-emerald-600" />
            ) : agent.state === "failed" ? (
              <MonitorCog className="size-4 text-destructive" />
            ) : (
              <MonitorCog className="size-4 text-muted-foreground" />
            )}
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <p className="truncate text-xs font-medium">{agent.name}</p>
              <span className="font-mono text-xs font-semibold text-muted-foreground">
                {String(index + 1).padStart(2, "0")}
              </span>
            </div>
            <p className="truncate text-[11px] text-muted-foreground">
              {agent.summary}
            </p>
          </div>
          <span className="hidden shrink-0 text-xs text-muted-foreground sm:inline">
            {agent.status}
          </span>
          <InlinePixels state={agent.state} />
        </div>
      ))}
    </div>
  )
}

function InlinePixels({
  state,
}: {
  state: ReturnType<typeof getInlineAgents>[number]["state"]
}) {
  return (
    <div className="hidden items-center gap-0.5 min-[1800px]:flex">
      {Array.from({ length: 12 }).map((_, pixelIndex) => (
        <span
          key={pixelIndex}
          className={`size-1 rounded-[1px] ${
            shouldLightInlinePixel(state, pixelIndex)
              ? state === "failed"
                ? "bg-destructive"
                : "bg-emerald-500"
              : "bg-muted-foreground/15"
          }`}
        />
      ))}
    </div>
  )
}
