import {
  Activity,
  ChevronDown,
  GitBranch,
  PanelRight,
  PanelRightClose,
} from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import {
  getAgentProcessBadge,
  getAgentProcessProgress,
  getAgentProcessSummary,
  getAgentProcessTitle,
  getDispatchFacts,
} from "../view-model/agent-process-kind-view-model"
import { getWorkbenchAgents } from "../view-model/agent-dispatch-workbench-agents-view-model"
import { getAgentRoleActivities } from "../view-model/agent-role-cluster-view-model"
import { hasFinished } from "../view-model/agent-process-runtime-view-model"
import { getInlineArtifactsFromRuntime } from "../view-model/thread-artifacts-view-model"
import {
  getCommerceHandoffItems,
  getInlineSteps,
} from "../view-model/thread-inline-view-model"
import { AgentRoleClusterPanel } from "./agent-role-cluster-panel"
import { InlineArtifactsPanel } from "./inline-artifacts-panel"
import { InlineRoleParticipantStrip } from "./inline-role-participant-strip"
import {
  InlineAgentGrid,
  InlineDispatchFactGrid,
  InlineProgressSummary,
  InlineRuntimeBadge,
  InlineStepStrip,
} from "./inline-swarm-task-details"

export function InlineSwarmTaskCard({
  isStreaming,
  messageId,
  onCloseProcessPanel,
  onOpenProcessPanel,
  processPanelOpen,
  runtimeEvents,
  showProcessPanelAction,
  task,
}: {
  isStreaming: boolean
  messageId: string
  onCloseProcessPanel: () => void
  onOpenProcessPanel: () => void
  processPanelOpen: boolean
  runtimeEvents: AgentRuntimeEvent[]
  showProcessPanelAction: boolean
  task: string
}) {
  const { t } = useTranslation()
  const isComplete = !isStreaming && hasFinished(runtimeEvents)
  const [manualOpen, setManualOpen] = useState<boolean | null>(null)
  const open = manualOpen ?? isStreaming
  const agents = getWorkbenchAgents(runtimeEvents, isStreaming, t)
  const steps = getInlineSteps(runtimeEvents, isStreaming, t)
  const completedCount = agents.filter(
    (agent) => agent.state === "done" || agent.state === "boundary"
  ).length
  const hasRuntime = runtimeEvents.length > 0
  const hasFailedAgent = agents.some((agent) => agent.state === "failed")
  const artifacts = getInlineArtifactsFromRuntime(runtimeEvents, t)
  const handoffItems = getCommerceHandoffItems(t)
  const processProgress = getAgentProcessProgress(
    {
      completedCount,
      isStreaming,
      runtimeEvents,
      totalCount: agents.length,
    },
    t
  )
  const processSummary = getAgentProcessSummary(runtimeEvents, task, t)
  const processTitle = hasRuntime
    ? getAgentProcessTitle(runtimeEvents, t)
    : t("agentChat.thread.processKinds.waiting.title")
  const processBadge = hasRuntime
    ? getAgentProcessBadge(runtimeEvents, t)
    : t("agentChat.thread.processKinds.waiting.badge")
  const processLabel = hasRuntime
    ? processProgress.label
    : t("agentChat.thread.processKinds.waiting.progress")
  const processDetail = hasRuntime
    ? processSummary
    : t("agentChat.thread.processKinds.waiting.summary")
  const showAgentGrid = agents.length > 0
  const dispatchFacts = getDispatchFacts(runtimeEvents, t)
  const roleActivities = showAgentGrid ? getAgentRoleActivities(agents, t) : []

  return (
    <article
      className="flex justify-start gap-3"
      data-process-message-id={messageId}
    >
      <div className="flex size-9 shrink-0 items-center justify-center rounded-xl border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) text-primary">
        <GitBranch className="size-5" />
      </div>
      <div className="max-w-[min(43rem,92%)] min-w-0 space-y-3">
        <div
          className={`rounded-2xl border bg-(--agent-chat-elevated-surface) bg-[image:var(--agent-chat-card-background)] p-3 shadow-[var(--agent-chat-card-shadow)] backdrop-blur-xl ${
            processPanelOpen
              ? "border-primary/60 ring-1 ring-primary/25"
              : "border-(--agent-chat-soft-border)"
          }`}
        >
          <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-start">
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-2 text-sm font-semibold">
                <Activity className="size-4 text-primary" />
                {processTitle}
                <span className="rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium text-secondary-foreground">
                  {processBadge}
                </span>
                <span className="text-xs font-normal text-muted-foreground">
                  {processLabel}
                </span>
                {processPanelOpen ? (
                  <span className="rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary">
                    {t("agentChat.thread.processPanelActive")}
                  </span>
                ) : null}
              </div>
              <p className="mt-1.5 text-xs/5 text-muted-foreground">
                {processDetail}
              </p>
            </div>
            <div className="flex shrink-0 flex-wrap items-center justify-start gap-2 sm:justify-end">
              <InlineRuntimeBadge
                hasRuntime={hasRuntime}
                isStreaming={isStreaming}
              />
              {showProcessPanelAction ? (
                <button
                  type="button"
                  onClick={
                    processPanelOpen ? onCloseProcessPanel : onOpenProcessPanel
                  }
                  className={`inline-flex h-8 shrink-0 items-center gap-1.5 rounded-md border px-2.5 text-xs font-medium shadow-sm transition-colors ${
                    processPanelOpen
                      ? "border-primary/60 bg-primary/10 text-primary"
                      : "border-primary/30 bg-(--agent-chat-elevated-strong) text-foreground hover:border-primary/60 hover:bg-(--agent-chat-accent-soft) hover:text-primary"
                  }`}
                  aria-pressed={processPanelOpen}
                  aria-label={
                    processPanelOpen
                      ? t("agentChat.thread.closeWorkbench")
                      : t("agentChat.thread.openWorkbench")
                  }
                  title={t("agentChat.thread.processLinkedToAnswer")}
                >
                  {processPanelOpen ? (
                    <PanelRightClose className="size-3.5" />
                  ) : (
                    <PanelRight className="size-3.5" />
                  )}
                  <span>
                    {processPanelOpen
                      ? t("agentChat.thread.closeWorkbench")
                      : t("agentChat.thread.openWorkbenchForTurn")}
                  </span>
                </button>
              ) : null}
              <button
                type="button"
                onClick={() =>
                  setManualOpen((current) => !(current ?? isStreaming))
                }
                className="flex size-8 items-center justify-center rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) text-muted-foreground transition-colors hover:bg-(--agent-chat-accent-soft) hover:text-foreground"
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
            {roleActivities.length > 0 ? (
              <div className="min-w-0 sm:col-span-2">
                <InlineRoleParticipantStrip
                  animatePortraits={isStreaming}
                  activities={roleActivities}
                  label={t("agentChat.thread.roleClusterActivity.participants")}
                />
              </div>
            ) : null}
          </div>
          {open ? (
            <div className="mt-3 grid gap-2">
              <div className="grid gap-2">
                {showAgentGrid ? (
                  <AgentRoleClusterPanel
                    agents={agents}
                    animatePortraits={isStreaming}
                  />
                ) : null}
                <InlineProgressSummary
                  completedCount={processProgress.completed}
                  hasFailedAgent={hasFailedAgent}
                  hasRuntime={hasRuntime}
                  isStreaming={isStreaming}
                  progressPercent={processProgress.percent}
                  runtimeEvents={runtimeEvents}
                  totalCount={processProgress.total}
                />
                {showAgentGrid ? (
                  <>
                    <InlineAgentGrid agents={agents} compact={isComplete} />
                    <InlineDispatchFactGrid facts={dispatchFacts} />
                  </>
                ) : (
                  <InlineDispatchFactGrid facts={dispatchFacts} />
                )}
              </div>
              <div className="mt-2">
                <InlineStepStrip steps={steps} />
              </div>
              <div className="mt-2 grid gap-2 min-[1500px]:grid-cols-2">
                <InlineArtifactsPanel
                  artifacts={artifacts}
                  hasRuntime={hasRuntime}
                  isStreaming={isStreaming}
                />
                {handoffItems.map((item, index) => {
                  const Icon = item.icon
                  return (
                    <div
                      key={`${index}-${item.label}`}
                      className="flex min-w-0 items-center gap-2 rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-subtle-surface) px-3 py-2 text-xs text-muted-foreground"
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
            </div>
          ) : null}
        </div>
      </div>
    </article>
  )
}
