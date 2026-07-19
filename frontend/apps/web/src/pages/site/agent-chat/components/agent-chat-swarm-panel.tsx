import { Activity, GitBranch, PanelRightClose, MonitorCog } from "lucide-react"
import type { ReactNode } from "react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentChatMessage } from "../types"
import {
  getAgentProcessKind,
  getDispatchFacts,
} from "../view-model/agent-process-kind-view-model"
import type { AgentRunFeedbackState } from "../view-model/use-agent-run-feedback"
import {
  getAgents,
  getAgentState,
  getCompletedCount,
  getPhases,
  getPlannedGraph,
  getRuntimeNodeStates,
  getStageLabel,
  getTimeline,
  hasFailedNode,
} from "../view-model/swarm-panel-view-model"
import { AgentDock } from "./swarm-agent-dock"
import { ComputerRuntimeDetail } from "./swarm-runtime-detail"
import {
  CompactInfoBlock,
  SwarmIdleDetail,
  WorkspaceHandoffRail,
} from "./swarm-support-panels"
import { AgentRunFeedbackPanel } from "./agent-run-feedback-panel"
import { AgentDispatchProcessPanel } from "./agent-dispatch-process-panel"

interface AgentChatSwarmPanelProps {
  feedback: AgentRunFeedbackState
  headerAddon?: ReactNode
  isStreaming: boolean
  messages: AgentChatMessage[]
  onCollapse: () => void
  runtimeEvents: AgentRuntimeEvent[]
  task?: string
}

export function AgentChatSwarmPanel({
  feedback,
  headerAddon,
  isStreaming,
  messages,
  onCollapse,
  runtimeEvents,
  task,
}: AgentChatSwarmPanelProps) {
  const { t } = useTranslation()
  const userMessages = messages.filter((message) => message.role === "user")
  const lastTask = task ?? userMessages.at(-1)?.content
  const hasTask = Boolean(lastTask)
  const hasRuntime = runtimeEvents.length > 0
  const processKind = getAgentProcessKind(runtimeEvents)
  if (processKind !== "multi_agent") {
    return (
      <AgentDispatchProcessPanel
        feedback={feedback}
        isStreaming={isStreaming}
        onCollapse={onCollapse}
        runtimeEvents={runtimeEvents}
        task={lastTask}
      />
    )
  }

  const graphNodes = getPlannedGraph(runtimeEvents)?.nodes ?? []
  const runtimeNodeStates = getRuntimeNodeStates(runtimeEvents)
  const agents = getAgents(graphNodes, t)
  const doneAgents = agents.filter((agent) => {
    const state = runtimeNodeStates.get(agent.nodeId)?.state
    return state === "done" || state === "boundary"
  })
  const activeAgent =
    agents.find(
      (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "working"
    ) ??
    agents.find(
      (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "reworking"
    ) ??
    agents.find(
      (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "boundary"
    ) ??
    doneAgents.at(-1) ??
    agents.find(
      (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "failed"
    ) ??
    agents[0]
  const activeAgentIndex = agents.indexOf(activeAgent)
  const activeState = getAgentState({
    agent: activeAgent,
    fallbackIndex: activeAgentIndex,
    hasTask,
    isStreaming,
    runtimeNodeStates,
  })
  const activeNode = runtimeNodeStates.get(activeAgent.nodeId)
  const progress = `${getCompletedCount(agents, runtimeNodeStates)}/${agents.length}`
  const stage = getStageLabel(
    { agents, hasTask, isStreaming, hasRuntime, runtimeNodeStates },
    t
  )
  const phases = getPhases(graphNodes, t)
  const timeline = getTimeline(runtimeEvents, agents, hasTask, isStreaming, t)
  const dispatchFactItems = getDispatchFacts(runtimeEvents, t)
    .slice(0, 3)
    .map((fact) => `${fact.label}: ${fact.detail}`)

  return (
    <aside className="h-full min-h-0 min-w-0 overflow-hidden bg-transparent">
      <div className="flex h-full min-h-0 flex-col overflow-hidden bg-transparent">
        <header className="sticky top-0 z-10 shrink-0 border-b border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong)/90 px-4 py-3 backdrop-blur-xl">
          <div className="flex items-start justify-between gap-3">
            <div className="flex min-w-0 items-center gap-3">
              <div className="flex size-9 shrink-0 items-center justify-center rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-subtle-surface) text-primary">
                <MonitorCog className="size-4" />
              </div>
              <div className="min-w-0">
                <h2 className="truncate text-sm font-semibold">
                  {t("agentChat.workbench.title")}
                </h2>
                <div className="mt-1.5 flex min-w-0 flex-wrap items-center gap-2 text-xs text-muted-foreground">
                  <span
                    className={`size-2.5 rounded-full border-2 border-white shadow-sm ${
                      isStreaming
                        ? "animate-pulse bg-emerald-500"
                        : runtimeNodeStates.size > 0
                          ? hasFailedNode(runtimeNodeStates)
                            ? "bg-destructive"
                            : "bg-emerald-500"
                          : "bg-muted-foreground/40"
                    }`}
                  />
                  <span className="text-foreground">
                    {t("agentChat.workbench.progress")} {progress}
                  </span>
                  <span className="hidden sm:inline">|</span>
                  <span className="min-w-0 truncate">{stage}</span>
                </div>
              </div>
            </div>
            <div className="cursor-pointer rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-subtle-surface) p-1">
              <button
                type="button"
                onClick={onCollapse}
                className="flex size-9 shrink-0 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                title={t("agentChat.workbench.collapse")}
                aria-label={t("agentChat.workbench.collapse")}
              >
                <PanelRightClose className="size-4" />
              </button>
            </div>
          </div>
        </header>

        {headerAddon ? (
          <div className="shrink-0 border-b border-(--agent-chat-soft-border) bg-(--agent-chat-subtle-surface)/70 p-2.5">
            {headerAddon}
          </div>
        ) : null}

        <div className="min-h-0 flex-1 overflow-x-hidden overflow-y-auto p-3 [scrollbar-gutter:stable]">
          {hasRuntime ? (
            <ComputerRuntimeDetail
              activeAgent={activeAgent}
              activeAgentIndex={activeAgentIndex}
              activeNode={activeNode}
              activeState={activeNode?.state ?? activeState}
              agents={agents}
              lastTask={lastTask}
              runtimeEvents={runtimeEvents}
            />
          ) : (
            <SwarmIdleDetail
              hasTask={hasTask}
              isStreaming={isStreaming}
              lastTask={lastTask}
            />
          )}

          <section className="mt-3 rounded-xl bg-(--agent-chat-subtle-surface)/45 p-2.5">
            <div className="mb-2 flex items-center gap-2 px-1 text-sm font-semibold text-foreground">
              <span className="size-1.5 rounded-full bg-primary" />
              <span>{t("agentChat.workbench.summary")}</span>
            </div>
            <div className="grid gap-2 pl-4">
              <CompactInfoBlock
                icon={GitBranch}
                title={t("agentChat.workbench.adaptiveDispatch")}
                items={dispatchFactItems}
                maxItems={4}
              />
              <WorkspaceHandoffRail compact />
              <AgentRunFeedbackPanel compact feedback={feedback} />
              <CompactInfoBlock
                icon={Activity}
                title={t("agentChat.workbench.activity")}
                items={[
                  ...timeline.slice(0, 2).map((item) => item.label),
                  ...phases.slice(0, 1).map((phase) => phase.title),
                ]}
              />
            </div>
          </section>
        </div>

        <div className="max-h-40 shrink-0 overflow-x-hidden overflow-y-auto border-t border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong)/70 [scrollbar-gutter:stable]">
          <AgentDock
            activeNodeId={activeAgent.nodeId}
            agents={agents}
            hasTask={hasTask}
            isStreaming={isStreaming}
            runtimeNodeStates={runtimeNodeStates}
          />
        </div>
      </div>
    </aside>
  )
}
