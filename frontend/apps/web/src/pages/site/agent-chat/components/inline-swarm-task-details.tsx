import {
  CheckCircle2,
  CircleAlert,
  Info,
  Loader2,
  MonitorCog,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentDispatchFact } from "../view-model/agent-process-kind-view-model"
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
    <div className="rounded-md border border-border bg-background px-3 py-2">
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
            key={`${index}-${label}`}
            className="flex min-w-0 max-w-full items-center gap-2 rounded-md border border-border bg-card px-2.5 py-1.5"
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

export function InlineDispatchFactGrid({
  facts,
}: {
  facts: AgentDispatchFact[]
}) {
  return (
    <div className="grid gap-1.5 sm:grid-cols-2">
      {facts.map((fact, index) => {
        const Icon = fact.tone === "warning" || fact.tone === "error"
          ? CircleAlert
          : fact.tone === "success"
            ? CheckCircle2
            : Info
        return (
          <div
            key={`${index}-${fact.label}`}
            className={`flex min-w-0 items-start gap-2 rounded-md border px-3 py-2 text-xs ${
              fact.tone === "warning"
                ? "border-amber-200 bg-amber-50/70 text-amber-950 dark:border-amber-900/60 dark:bg-amber-950/20 dark:text-amber-100"
                : fact.tone === "error"
                  ? "border-destructive/25 bg-destructive/10"
                  : "border-border bg-background"
            }`}
          >
            <Icon className="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
            <div className="min-w-0">
              <p className="truncate font-medium text-foreground">{fact.label}</p>
              <p className="mt-0.5 line-clamp-2 text-muted-foreground">
                {fact.detail}
              </p>
            </div>
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

  if (hasRuntime && !isStreaming) return null

  return (
    <span className="rounded-full bg-secondary px-2 py-1 text-[11px] font-medium text-secondary-foreground">
      {hasRuntime
        ? isStreaming
          ? t("agentChat.thread.statuses.executing")
          : null
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
  const visibleAgents = compact ? agents.slice(0, 4) : agents.slice(0, 8)

  return (
    <div className="grid gap-1.5">
      {visibleAgents.map((agent, index) => (
        <div
          key={agent.nodeId}
          className={`flex min-w-0 items-center gap-2.5 rounded-md border px-3 py-2 ${
            agent.state === "working" || agent.state === "reworking"
              ? "border-foreground/25 bg-background"
              : agent.state === "boundary"
                ? "border-amber-200 bg-amber-50/70 text-amber-950 dark:border-amber-900/60 dark:bg-amber-950/20 dark:text-amber-100"
              : agent.state === "failed"
                ? "border-destructive/25 bg-destructive/10"
                : "border-border bg-background/80"
          }`}
        >
          <div className="flex size-7 shrink-0 items-center justify-center rounded-md border border-border bg-background">
            {agent.state === "working" || agent.state === "reworking" ? (
              <Loader2 className="size-4 animate-spin text-emerald-600" />
            ) : agent.state === "done" ? (
              <CheckCircle2 className="size-4 text-emerald-600" />
            ) : agent.state === "boundary" ? (
              <MonitorCog className="size-4 text-amber-600" />
            ) : agent.state === "failed" ? (
              <MonitorCog className="size-4 text-destructive" />
            ) : (
              <MonitorCog className="size-4 text-muted-foreground" />
            )}
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <p className="truncate text-sm font-semibold">{agent.name}</p>
              <span className="font-mono text-xs font-semibold text-muted-foreground">
                {String(index + 1).padStart(2, "0")}
              </span>
            </div>
            <p className="truncate text-[11px] text-muted-foreground">
              {agent.summary}
            </p>
          </div>
          <span className="hidden shrink-0 text-xs font-medium text-muted-foreground sm:inline">
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
          className={`size-1 rounded-xs ${
            shouldLightInlinePixel(state, pixelIndex)
              ? state === "failed"
                ? "bg-destructive"
                : state === "boundary"
                  ? "bg-amber-500"
                : "bg-emerald-500"
              : "bg-muted-foreground/15"
          }`}
        />
      ))}
    </div>
  )
}
