import {
  Activity,
  ChevronDown,
  GitBranch,
  PanelRightOpen,
} from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import { getInlineArtifactsFromRuntime } from "../view-model/thread-artifacts-view-model"
import {
  getCommerceHandoffItems,
  getInlineAgents,
  getInlineSteps,
} from "../view-model/thread-inline-view-model"
import { InlineArtifactsPanel } from "./inline-artifacts-panel"
import {
  InlineAgentGrid,
  InlineProgressSummary,
  InlineRuntimeBadge,
  InlineStepStrip,
} from "./inline-swarm-task-details"

export function InlineSwarmTaskCard({
  isStreaming,
  onOpenProcessPanel,
  processPanelOpen,
  runtimeEvents,
  showProcessPanelAction,
  task,
}: {
  isStreaming: boolean
  onOpenProcessPanel: () => void
  processPanelOpen: boolean
  runtimeEvents: AgentRuntimeEvent[]
  showProcessPanelAction: boolean
  task: string
}) {
  const { t } = useTranslation()
  const isComplete = !isStreaming && runtimeEvents.some((event) =>
    event.type === "task_graph_finished" ||
    event.type === "workspace_run_completed" ||
    event.type === "workspace_run_failed" ||
    event.type === "failed" ||
    event.type === "finished"
  )
  const [open, setOpen] = useState(false)
  const agents = getInlineAgents(runtimeEvents, isStreaming, t)
  const steps = getInlineSteps(runtimeEvents, isStreaming, t)
  const completedCount = agents.filter((agent) => agent.state === "done").length
  const hasRuntime = runtimeEvents.length > 0
  const progressPercent = agents.length
    ? Math.min(100, Math.round((completedCount / agents.length) * 100))
    : 0
  const hasFailedAgent = agents.some((agent) => agent.state === "failed")
  const activeAgent = agents.find(
    (agent) =>
      agent.state === "working" ||
      agent.state === "reworking" ||
      agent.state === "failed"
  )
  const artifacts = getInlineArtifactsFromRuntime(runtimeEvents, t)
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
        <div className="rounded-lg border border-border bg-[#f5f5f2] p-2.5 shadow-sm dark:bg-card">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-2 text-sm font-semibold">
                <Activity className="size-4 text-muted-foreground" />
                {t("agentChat.thread.executionTitle")}
                <span className="text-xs font-normal text-muted-foreground">
                  {progressLabel}
                </span>
              </div>
              <p className="mt-1.5 line-clamp-1 text-xs/5 text-muted-foreground">
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
              {showProcessPanelAction ? (
                <button
                  type="button"
                  onClick={onOpenProcessPanel}
                  className="flex h-7 items-center gap-1.5 rounded-md border border-border bg-background px-2 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                  aria-pressed={processPanelOpen}
                  title={t("agentChat.thread.openWorkbench")}
                >
                  <PanelRightOpen className="size-3.5" />
                  <span className="hidden sm:inline">
                    {t("agentChat.thread.openWorkbench")}
                  </span>
                </button>
              ) : null}
              <button
                type="button"
                onClick={() => setOpen((current) => !current)}
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
              <div className="mt-2 grid gap-2">
                <InlineProgressSummary
                  completedCount={completedCount}
                  hasFailedAgent={hasFailedAgent}
                  hasRuntime={hasRuntime}
                  isStreaming={isStreaming}
                  progressPercent={progressPercent}
                  runtimeEvents={runtimeEvents}
                  totalCount={agents.length}
                />
                <InlineAgentGrid agents={agents} compact={isComplete} />
                <InlineStepStrip steps={steps} />
              </div>
              <div className="mt-2 grid gap-2 min-[1500px]:grid-cols-2">
                <InlineArtifactsPanel
                  artifacts={artifacts}
                  hasRuntime={hasRuntime}
                  isStreaming={isStreaming}
                />
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
