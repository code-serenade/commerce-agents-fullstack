import {
  Activity,
  CheckCircle2,
  ChevronDown,
  GitBranch,
  Loader2,
  MonitorCog,
} from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import {
  getCommerceHandoffItems,
  getInlineAgents,
  getInlineArtifacts,
  getInlineSteps,
  shouldLightInlinePixel,
} from "../view-model/thread-inline-view-model"
import { InlineArtifactsPanel } from "./inline-artifacts-panel"

export function InlineSwarmTaskCard({
  isStreaming,
  runtimeEvents,
  task,
}: {
  isStreaming: boolean
  runtimeEvents: AgentRuntimeEvent[]
  task: string
}) {
  const { t } = useTranslation()
  const isComplete = !isStreaming && runtimeEvents.some((event) =>
    event.type === "task_graph_finished" ||
    event.type === "workspace_run_completed" ||
    event.type === "finished"
  )
  const autoOpen = isStreaming && !isComplete
  const [manualOpen, setManualOpen] = useState<boolean | null>(null)
  const open = manualOpen ?? autoOpen
  const agents = getInlineAgents(runtimeEvents, isStreaming, t)
  const steps = getInlineSteps(runtimeEvents, isStreaming, t)
  const completedCount = agents.filter((agent) => agent.state === "done").length
  const hasRuntime = runtimeEvents.length > 0
  const activeAgent = agents.find(
    (agent) =>
      agent.state === "working" ||
      agent.state === "reworking" ||
      agent.state === "failed"
  )
  const artifacts = getInlineArtifacts(agents, hasRuntime, isStreaming, t)
  const handoffItems = getCommerceHandoffItems(t)
  const progressLabel = hasRuntime
    ? t("agentChat.thread.completedTasks", {
        completed: completedCount,
        total: agents.length,
      })
    : isStreaming
      ? t("agentChat.thread.planning")
      : t("agentChat.thread.historyNotExecuted")

  return (
    <article className="flex justify-start gap-3">
      <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-card text-foreground">
        <GitBranch className="size-5" />
      </div>
      <div className="max-w-[min(43rem,92%)] min-w-0 space-y-4">
        <div className="rounded-xl border border-border bg-[#f3f3f1] p-4 shadow-sm dark:bg-card">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-2 text-sm font-semibold">
                <Activity className="size-4 text-muted-foreground" />
                {t("agentChat.thread.executionTitle")}
                <span className="text-xs font-normal text-muted-foreground">
                  {progressLabel}
                </span>
              </div>
              <p className="mt-2 line-clamp-2 text-xs/5 text-muted-foreground">
                {activeAgent
                  ? `${activeAgent.name} ${activeAgent.status}: ${task}`
                : task}
              </p>
            </div>
            <div className="flex shrink-0 items-center gap-2">
              <InlineRuntimeBadge
                hasRuntime={hasRuntime}
                isStreaming={isStreaming}
              />
              <button
                type="button"
                onClick={() =>
                  setManualOpen((current) => !(current ?? autoOpen))
                }
                className="flex size-7 items-center justify-center rounded-md border border-border bg-background text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                aria-expanded={open}
                title={
                  open
                    ? t("agentChat.thread.collapseProcess")
                    : t("agentChat.thread.expandProcess")
                }
              >
                <ChevronDown
                  className={`size-4 transition-transform ${open ? "rotate-180" : ""}`}
                />
              </button>
            </div>
          </div>
          {open ? (
            <>
              <InlineStepStrip steps={steps} />
              <InlineAgentGrid agents={agents} />
              <InlineArtifactsPanel
                artifacts={artifacts}
                hasRuntime={hasRuntime}
                isStreaming={isStreaming}
              />
              <div className="mt-3 grid gap-2 sm:grid-cols-2">
                {handoffItems.map((item) => {
                  const Icon = item.icon
                  return (
                    <div
                      key={item.label}
                      className="flex min-w-0 items-center gap-2 rounded-md border border-border bg-background px-3 py-2 text-xs text-muted-foreground"
                    >
                      <Icon className="size-3.5 shrink-0" />
                      <span className="shrink-0 font-medium text-foreground">
                        {item.label}
                      </span>
                      <span className="min-w-0 truncate">{item.detail}</span>
                    </div>
                  )
                })}
              </div>
            </>
          ) : null}
        </div>
      </div>
    </article>
  )
}

function InlineStepStrip({ steps }: { steps: string[][] }) {
  return (
    <div className="mt-4 flex min-w-0 gap-2 overflow-x-auto pb-1">
      {steps.map(([label, detail], index) => {
        return (
          <div
            key={label}
            className="min-w-44 rounded-lg border border-border bg-card px-3 py-2 shadow-sm"
          >
            <div className="flex min-w-0 items-center gap-2">
              <span className="font-mono text-xs font-semibold text-muted-foreground">
                {String(index + 1).padStart(2, "0")}
              </span>
              <p className="truncate text-sm font-medium">{label}</p>
            </div>
            <p className="mt-1 truncate text-xs text-muted-foreground">{detail}</p>
          </div>
        )
      })}
    </div>
  )
}

function InlineRuntimeBadge({
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

function InlineAgentGrid({
  agents,
}: {
  agents: ReturnType<typeof getInlineAgents>
}) {
  return (
    <div className="mt-4 grid gap-2 sm:grid-cols-2">
      {agents.slice(0, 6).map((agent, index) => (
        <div
          key={agent.nodeId}
          className={`flex min-w-0 items-center gap-3 rounded-lg px-3 py-2 ${
            agent.state === "working" || agent.state === "reworking"
              ? "bg-background ring-1 ring-foreground/25"
              : agent.state === "failed"
                ? "bg-destructive/10"
                : "bg-background/80"
          }`}
        >
          <div className="flex size-8 shrink-0 items-center justify-center rounded-full border border-border bg-background">
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
              <p className="truncate text-sm font-medium">{agent.name}</p>
              <span className="font-mono text-xs font-semibold text-muted-foreground">
                {String(index + 1).padStart(2, "0")}
              </span>
            </div>
            <p className="truncate text-xs text-muted-foreground">
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
