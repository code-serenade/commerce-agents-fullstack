import {
  Activity,
  BarChart3,
  Bot,
  CheckCircle2,
  ClipboardCheck,
  Database,
  FileText,
  Loader2,
  Megaphone,
  MonitorCog,
  PanelRightClose,
  PenLine,
  PlugZap,
  Search,
  SearchCheck,
  ShieldAlert,
  TerminalSquare,
  WalletCards,
} from "lucide-react"
import { type TFunction } from "i18next"
import type { ComponentType, SVGProps } from "react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent, AgentTaskNode, AgentWorkerReport } from "@/api"
import type { AgentChatMessage } from "../types"

type SwarmState =
  | "idle"
  | "queued"
  | "working"
  | "done"
  | "failed"
  | "reworking"
  | "skipped"

interface AgentChatSwarmPanelProps {
  isStreaming: boolean
  messages: AgentChatMessage[]
  onCollapse: () => void
  runtimeEvents: AgentRuntimeEvent[]
}

interface SwarmAgent {
  nodeId: string
  name: string
  role: string
  summary: string
  icon: ComponentType<SVGProps<SVGSVGElement>>
  tone: string
  objective: string
  dependsOn: string[]
}

interface RuntimeNodeSnapshot {
  attempt?: number
  evaluationReasons: string[]
  report?: AgentWorkerReport
  state: SwarmState
}

interface SwarmTimelineItem {
  label: string
  tone: "default" | "success" | "warning" | "error"
}

interface ComputerCard {
  id: string
  title: string
  meta?: string
  description: string
  icon: ComponentType<SVGProps<SVGSVGElement>>
  lines: string[]
  state: SwarmState
}

interface ComputerScreenContext {
  filename: string
  mode: string
  request: string
  lines: string[]
}

const WORKSPACE_LANES = [
  { labelKey: "agentChat.swarm.lanes.context.label", valueKey: "agentChat.swarm.lanes.context.value" },
  { labelKey: "agentChat.swarm.lanes.execution.label", valueKey: "agentChat.swarm.lanes.execution.value" },
  { labelKey: "agentChat.swarm.lanes.delivery.label", valueKey: "agentChat.swarm.lanes.delivery.value" },
  { labelKey: "agentChat.swarm.lanes.governance.label", valueKey: "agentChat.swarm.lanes.governance.value" },
]

const AGENTS = [
  {
    nodeId: "data",
    nameKey: "agentChat.swarm.agents.data.name",
    role: "data",
    summaryKey: "agentChat.swarm.agents.data.summary",
    icon: BarChart3,
    tone: "bg-blue-50 text-blue-700 dark:bg-blue-950/40 dark:text-blue-200",
    objectiveKey: "agentChat.swarm.agents.data.objective",
    dependsOn: [],
  },
  {
    nodeId: "accounting",
    nameKey: "agentChat.swarm.agents.accounting.name",
    role: "accounting",
    summaryKey: "agentChat.swarm.agents.accounting.summary",
    icon: WalletCards,
    tone: "bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-200",
    objectiveKey: "agentChat.swarm.agents.accounting.objective",
    dependsOn: ["data"],
  },
  {
    nodeId: "douyin_ads",
    nameKey: "agentChat.swarm.agents.douyinAds.name",
    role: "douyin_ads",
    summaryKey: "agentChat.swarm.agents.douyinAds.summary",
    icon: Megaphone,
    tone: "bg-cyan-50 text-cyan-700 dark:bg-cyan-950/40 dark:text-cyan-200",
    objectiveKey: "agentChat.swarm.agents.douyinAds.objective",
    dependsOn: ["data"],
  },
  {
    nodeId: "creative",
    nameKey: "agentChat.swarm.agents.creative.name",
    role: "creative",
    summaryKey: "agentChat.swarm.agents.creative.summary",
    icon: PenLine,
    tone: "bg-violet-50 text-violet-700 dark:bg-violet-950/40 dark:text-violet-200",
    objectiveKey: "agentChat.swarm.agents.creative.objective",
    dependsOn: ["data"],
  },
  {
    nodeId: "ops",
    nameKey: "agentChat.swarm.agents.ops.name",
    role: "ops",
    summaryKey: "agentChat.swarm.agents.ops.summary",
    icon: MonitorCog,
    tone: "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-200",
    objectiveKey: "agentChat.swarm.agents.ops.objective",
    dependsOn: ["accounting", "douyin_ads", "creative"],
  },
  {
    nodeId: "delivery",
    nameKey: "agentChat.swarm.agents.delivery.name",
    role: "delivery",
    summaryKey: "agentChat.swarm.agents.delivery.summary",
    icon: FileText,
    tone: "bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-200",
    objectiveKey: "agentChat.swarm.agents.delivery.objective",
    dependsOn: ["ops"],
  },
]

const FALLBACK_PHASES = [
  {
    nodeId: "data",
    titleKey: "agentChat.swarm.phases.data.title",
    detailKey: "agentChat.swarm.phases.data.detail",
  },
  {
    nodeId: "parallel",
    titleKey: "agentChat.swarm.phases.parallel.title",
    detailKey: "agentChat.swarm.phases.parallel.detail",
  },
  {
    nodeId: "ops",
    titleKey: "agentChat.swarm.phases.ops.title",
    detailKey: "agentChat.swarm.phases.ops.detail",
  },
  {
    nodeId: "delivery",
    titleKey: "agentChat.swarm.phases.delivery.title",
    detailKey: "agentChat.swarm.phases.delivery.detail",
  },
]

const WORKSPACE_HANDOFFS = [
  {
    titleKey: "agentChat.swarm.handoffs.context.title",
    detailKey: "agentChat.swarm.handoffs.context.detail",
    icon: Database,
  },
  {
    titleKey: "agentChat.swarm.handoffs.execution.title",
    detailKey: "agentChat.swarm.handoffs.execution.detail",
    icon: ClipboardCheck,
  },
  {
    titleKey: "agentChat.swarm.handoffs.delivery.title",
    detailKey: "agentChat.swarm.handoffs.delivery.detail",
    icon: FileText,
  },
  {
    titleKey: "agentChat.swarm.handoffs.governance.title",
    detailKey: "agentChat.swarm.handoffs.governance.detail",
    icon: ShieldAlert,
  },
  {
    titleKey: "agentChat.swarm.handoffs.review.title",
    detailKey: "agentChat.swarm.handoffs.review.detail",
    icon: Bot,
  },
]

export function AgentChatSwarmPanel({
  isStreaming,
  messages,
  onCollapse,
  runtimeEvents,
}: AgentChatSwarmPanelProps) {
  const { t } = useTranslation()
  const userMessages = messages.filter((message) => message.role === "user")
  const hasTask = userMessages.length > 0
  const lastTask = userMessages.at(-1)?.content
  const hasRuntime = runtimeEvents.length > 0
  const plannedGraph = getPlannedGraph(runtimeEvents)
  const graphNodes = plannedGraph?.nodes ?? []
  const runtimeNodeStates = getRuntimeNodeStates(runtimeEvents)
  const agents = getAgents(graphNodes, t)
  const doneAgents = agents.filter(
    (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "done"
  )
  const activeAgent =
    agents.find(
      (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "working"
    ) ??
    agents.find(
      (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "reworking"
    ) ??
    agents.find(
      (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "failed"
    ) ??
    doneAgents.at(-1) ??
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
  const completedCount = getCompletedCount(agents, runtimeNodeStates)
  const progress = `${completedCount}/${agents.length}`
  const stage = getStageLabel({
    agents,
    hasTask,
    isStreaming,
    hasRuntime,
    runtimeNodeStates,
  }, t)
  const phases = getPhases(graphNodes, t)
  const timeline = getTimeline(runtimeEvents, agents, hasTask, isStreaming, t)
  const computerCards = getComputerCards({
    activeAgent,
    activeNode,
    agents,
    lastTask,
    runtimeEvents,
  }, t)
  return (
    <aside className="h-full min-h-124 min-w-0 overflow-hidden rounded-[1.75rem] bg-[#eeeeeb] p-2 min-[1440px]:p-3 2xl:min-h-0 dark:bg-muted/30">
      <div className="flex h-full min-h-0 flex-col overflow-hidden rounded-[1.45rem] bg-[#f6f6f3] dark:bg-card">
        <header className="sticky top-0 z-10 shrink-0 border-b border-[#d8d8d3] bg-[#f6f6f3]/95 p-3 backdrop-blur min-[1440px]:px-5 dark:border-border dark:bg-card/95">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0">
              <div className="flex min-w-0 items-center gap-3">
                <div className="flex size-11 shrink-0 items-center justify-center rounded-md border border-foreground/20 bg-background text-foreground shadow-sm min-[1500px]:size-12">
                  <MonitorCog className="size-6 min-[1500px]:size-7" />
                </div>
                <div className="min-w-0">
                  <h2 className="truncate font-mono text-xl font-semibold tracking-wide min-[1500px]:text-2xl">
                    {t("agentChat.workbench.title")}
                  </h2>
                  <div className="mt-1.5 flex min-w-0 flex-wrap items-center gap-2 text-sm text-muted-foreground min-[1500px]:text-base">
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
            </div>
            <div className="flex shrink-0 items-center gap-2">
              <button
                type="button"
                onClick={onCollapse}
                className="flex size-9 shrink-0 items-center justify-center rounded-full border border-border bg-background text-muted-foreground shadow-sm transition-colors hover:bg-muted hover:text-foreground min-[1500px]:size-10"
                title={t("agentChat.workbench.collapse")}
                aria-label={t("agentChat.workbench.collapse")}
              >
                <PanelRightClose className="size-4" />
              </button>
            </div>
          </div>
        </header>

        <div className="min-h-0 flex-1 overflow-y-auto p-2 min-[1440px]:p-4">
          {hasRuntime ? (
            <ComputerRuntimeDetail
              activeAgent={activeAgent}
              activeAgentIndex={activeAgentIndex}
              activeNode={activeNode}
              activeState={activeNode?.state ?? activeState}
              cards={computerCards}
              runtimeEvents={runtimeEvents}
              t={t}
            />
          ) : (
            <SwarmIdleDetail
              hasTask={hasTask}
              isStreaming={isStreaming}
              lastTask={lastTask}
              t={t}
            />
          )}

          <section className="mt-3 grid gap-3 min-[1800px]:grid-cols-[minmax(0,1fr)_17rem]">
            <WorkspaceHandoffRail t={t} />
            <CompactInfoBlock
              icon={Activity}
              title={t("agentChat.workbench.activity")}
              items={[
                ...timeline.slice(0, 2).map((item) => item.label),
                ...phases.slice(0, 1).map((phase) => phase.title),
              ]}
            />
          </section>
        </div>

        <AgentDock
          activeNodeId={activeAgent.nodeId}
          agents={agents}
          hasTask={hasTask}
          isStreaming={isStreaming}
          runtimeNodeStates={runtimeNodeStates}
          t={t}
        />
      </div>
    </aside>
  )
}

function ComputerRuntimeDetail({
  activeAgent,
  activeAgentIndex,
  activeNode,
  activeState,
  cards,
  runtimeEvents,
  t,
}: {
  activeAgent: SwarmAgent
  activeAgentIndex: number
  activeNode?: RuntimeNodeSnapshot
  activeState: SwarmState
  cards: ComputerCard[]
  runtimeEvents: AgentRuntimeEvent[]
  t: TFunction
}) {
  const screen = getComputerScreenContext({
    activeAgent,
    activeNode,
    runtimeEvents,
  }, t)
  const ActiveIcon = activeAgent.icon
  const activityCards = cards.slice(-4)

  return (
    <section className="grid min-h-124 overflow-hidden rounded-[1.25rem] border border-[#dcdcd7] bg-white shadow-sm min-[1500px]:grid-cols-[12.75rem_minmax(0,1fr)] dark:border-border dark:bg-background">
      <div className="min-w-0 border-b border-border bg-[#f7f7f4] p-3 min-[1500px]:border-r min-[1500px]:border-b-0 dark:bg-card">
        <div className="mb-2 flex items-center justify-between gap-2 text-xs text-muted-foreground">
          <span>{t("agentChat.workbench.record")}</span>
          <span>
            {activityCards.length} {t("agentChat.workbench.recordCountSuffix")}
          </span>
        </div>
        <div className="grid gap-2 min-[1500px]:grid-cols-1 sm:grid-cols-2">
          {activityCards.map((card) => (
            <ComputerRuntimeCard key={card.id} card={card} />
          ))}
        </div>
      </div>

      <div className="min-w-0 bg-white dark:bg-background">
        <div className="flex items-center justify-between gap-3 border-b border-border bg-white px-4 py-3 min-[1500px]:px-5 dark:bg-background">
          <div className="flex min-w-0 items-center gap-2">
            <div className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-background">
              <ActiveIcon className="size-4" />
            </div>
            <div className="min-w-0">
              <p className="truncate font-mono text-sm font-semibold">
                {formatAgentNumber(activeAgentIndex)} | {activeAgent.name}
              </p>
              <p className="truncate text-xs text-muted-foreground">
                {screen.mode} · {screen.filename}
              </p>
            </div>
          </div>
          <StatePill state={activeState} />
        </div>

        <div className="grid min-h-112 grid-rows-[auto_minmax(0,1fr)_auto]">
          <div className="border-b border-border bg-white px-4 py-3 min-[1500px]:px-5 dark:bg-background">
            <div className="flex items-center justify-between gap-3">
              <span className="text-xs font-medium text-muted-foreground">
                {t("agentChat.workbench.businessInput")}
              </span>
            </div>
            <p className="mt-1 line-clamp-2 text-sm/6 text-foreground">
              {screen.request}
            </p>
          </div>

          <div className="min-h-0 bg-white p-3 min-[1500px]:p-4 dark:bg-background">
            <div className="flex h-full min-h-72 flex-col overflow-hidden rounded-lg border border-border bg-[#fbfbfa] dark:bg-card">
              <div className="flex items-center justify-between gap-3 border-b border-border bg-[#f7f7f4] px-4 py-2.5 dark:bg-card">
                <div className="flex min-w-0 items-center gap-2">
                  <FileText className="size-4 shrink-0 text-muted-foreground" />
                  <span className="truncate text-sm font-medium">
                    {screen.filename}
                  </span>
                </div>
                <span className="shrink-0 rounded-full bg-secondary px-2 py-1 text-[11px] text-secondary-foreground">
                  {t("agentChat.workbench.preview")}
                </span>
              </div>
              <div className="min-h-0 flex-1 overflow-hidden py-2 font-mono text-[11px]/5">
                {screen.lines.slice(0, 14).map((line, index) => (
                  <div
                    key={`${index}-${line}`}
                    className="grid grid-cols-[2.5rem_minmax(0,1fr)] gap-3 px-3 py-0.5"
                  >
                    <span className="text-right text-muted-foreground/70 tabular-nums select-none">
                      {String(index + 1).padStart(2, "0")}
                    </span>
                    <span className="min-w-0 wrap-break-word text-foreground/85">
                      {line}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          <div className="grid gap-2 border-t border-border bg-[#f5f5f2] p-3 min-[1500px]:grid-cols-[minmax(0,1fr)_9rem] dark:bg-card">
            <div className="rounded-md border border-border bg-background px-3 py-2">
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                {activeState === "working" || activeState === "reworking" ? (
                  <Loader2 className="size-4 animate-spin text-emerald-600" />
                ) : activeState === "done" ? (
                  <CheckCircle2 className="size-4 text-emerald-600" />
                ) : (
                  <MonitorCog className="size-4" />
                )}
                <span className="truncate">
                  {activeAgent.name} · {getDockStateLabel(activeState, t)}
                </span>
              </div>
              <p className="mt-1 line-clamp-1 text-xs text-muted-foreground">
                {activeNode?.report
                  ? getFallbackResponse(activeAgent, activeState, t)
                  : activeAgent.objective}
              </p>
            </div>
            <div className="rounded-md border border-border bg-background px-3 py-2">
              <p className="text-xs text-muted-foreground">
                {t("agentChat.workbench.turns")}
              </p>
              <p className="mt-1 font-mono text-sm font-semibold">
                {activeNode?.attempt ?? 1}
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}

function ComputerRuntimeCard({ card }: { card: ComputerCard }) {
  const Icon = card.icon

  return (
    <article className="rounded-lg border border-border/70 bg-background p-2.5 shadow-sm min-[1500px]:border-transparent">
      <div className="flex items-start justify-between gap-2">
        <div className="flex min-w-0 items-start gap-2">
          <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
            <Icon className="size-4" />
          </div>
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <p className="text-sm font-medium">{card.title}</p>
              {card.meta ? (
                <span className="max-w-28 truncate text-xs text-muted-foreground">
                  | {card.meta}
                </span>
              ) : null}
            </div>
            <p className="mt-1 line-clamp-2 text-xs/5 text-muted-foreground">
              {card.description}
            </p>
          </div>
        </div>
        <StatePill state={card.state} />
      </div>

      {card.lines.length ? (
        <div className="mt-2 space-y-1">
          {card.lines.slice(0, 2).map((line) => (
            <div
              key={line}
              className="line-clamp-1 rounded-md bg-muted/70 px-2 py-1 text-[11px] text-muted-foreground"
            >
              {line}
            </div>
          ))}
        </div>
      ) : null}
    </article>
  )
}

function AgentDock({
  activeNodeId,
  agents,
  hasTask,
  isStreaming,
  runtimeNodeStates,
  t,
}: {
  activeNodeId: string
  agents: SwarmAgent[]
  hasTask: boolean
  isStreaming: boolean
  runtimeNodeStates: Map<string, RuntimeNodeSnapshot>
  t: TFunction
}) {
  return (
    <div className="shrink-0 border-t border-[#d8d8d3] bg-[#f8f8f6] px-4 py-3 dark:border-border dark:bg-card">
      <div className="grid grid-cols-3 gap-2 min-[1500px]:grid-cols-6">
        {agents.map((agent, index) => {
          const state = getAgentState({
            agent,
            fallbackIndex: index,
            hasTask,
            isStreaming,
            runtimeNodeStates,
          })
          const Icon = agent.icon
          const isActive =
            agent.nodeId === activeNodeId && (hasTask || isStreaming)

          return (
            <article
              key={agent.nodeId}
              title={`${formatAgentNumber(index)} ${agent.name} ${getDockStateLabel(state, t)}`}
              className={`flex min-w-0 items-center gap-1.5 rounded-xl border px-3 py-2.5 transition-colors ${
                isActive
                  ? "border-foreground bg-background shadow-sm"
                  : "border-transparent bg-[#eeeeeb] text-muted-foreground"
              }`}
            >
              <div
                className={`flex size-7 shrink-0 items-center justify-center rounded-full border border-border bg-background ${
                  isActive ? "text-foreground" : "text-muted-foreground"
                }`}
              >
                <Icon className="size-3.5" />
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex min-w-0 items-center gap-1">
                  <span className="shrink-0 font-mono text-[10px] font-semibold text-muted-foreground">
                    {formatAgentNumber(index)}
                  </span>
                  <p className="truncate text-[11px] font-medium">
                    {agent.name}
                  </p>
                </div>
                <div className="mt-0.5 flex items-center gap-1 text-[10px] text-muted-foreground">
                  <DockStateIcon state={state} />
                  <span className="hidden truncate min-[1500px]:inline">
                    {getDockStateLabel(state, t)}
                  </span>
                </div>
              </div>
            </article>
          )
        })}
      </div>
    </div>
  )
}

function DockStateIcon({ state }: { state: SwarmState }) {
  if (state === "done")
    return <CheckCircle2 className="size-3 text-emerald-600" />
  if (state === "working" || state === "reworking") {
    return <Loader2 className="size-3 animate-spin text-emerald-600" />
  }
  if (state === "failed")
    return <span className="size-2 rounded-full bg-destructive" />
  return <span className="size-2 rounded-full bg-muted-foreground/35" />
}

function WorkspaceHandoffRail({ t }: { t: TFunction }) {
  return (
    <div className="rounded-xl border border-border bg-background p-3 shadow-sm">
      <div className="mb-3 flex items-center justify-between gap-3">
        <div className="flex items-center gap-2 text-sm font-medium">
          <PlugZap className="size-4 text-muted-foreground" />
          {t("agentChat.workbench.handoff")}
        </div>
        <span className="text-xs text-muted-foreground">
          {t("agentChat.workbench.futureModules")}
        </span>
      </div>
      <div className="grid gap-2 min-[1500px]:grid-cols-2 min-[1800px]:grid-cols-4">
        {WORKSPACE_HANDOFFS.map((handoff) => {
          const Icon = handoff.icon
          const title = t(handoff.titleKey)

          return (
            <div
              key={handoff.titleKey}
              className="flex min-w-0 items-start gap-2 rounded-lg border border-border bg-card px-3 py-2"
            >
              <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
                <Icon className="size-4" />
              </div>
              <div className="min-w-0">
                <p className="truncate text-sm font-medium">{title}</p>
                <p className="mt-1 line-clamp-2 text-xs/5 text-muted-foreground">
                  {t(handoff.detailKey)}
                </p>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}

function CompactInfoBlock({
  icon: Icon,
  items,
  title,
}: {
  icon: ComponentType<SVGProps<SVGSVGElement>>
  items: string[]
  title: string
}) {
  return (
    <div className="rounded-xl border border-border bg-background p-3 shadow-sm">
      <div className="mb-2 flex items-center gap-2 text-sm font-medium">
        <Icon className="size-4 text-muted-foreground" />
        {title}
      </div>
      <div className="space-y-1.5">
        {items.map((item) => (
          <div
            key={item}
            className="flex items-center gap-2 text-xs text-muted-foreground"
          >
            <span className="size-1.5 shrink-0 rounded-full bg-muted-foreground/35" />
            <span className="truncate">{item}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

function SwarmIdleDetail({
  hasTask,
  isStreaming,
  lastTask,
  t,
}: {
  hasTask: boolean
  isStreaming: boolean
  lastTask?: string
  t: TFunction
}) {
  const title = isStreaming
    ? t("agentChat.workbench.idle.planningTitle")
    : hasTask
      ? t("agentChat.workbench.idle.historyTitle")
      : t("agentChat.workbench.idle.waitingTitle")
  const description = isStreaming
    ? t("agentChat.workbench.idle.planningDescription")
    : hasTask
      ? t("agentChat.workbench.idle.historyDescription")
      : t("agentChat.workbench.idle.waitingDescription")
  const lines = t("agentChat.workbench.idle.lines", {
    returnObjects: true,
  }) as string[]

  return (
    <div className="p-3 min-[1500px]:p-4">
      <div className="min-w-0 overflow-hidden rounded-xl border border-[#d8d8d3] bg-white shadow-sm dark:border-border dark:bg-background">
        <div className="flex items-center justify-between gap-3 border-b border-border bg-white px-5 py-3 dark:bg-background">
          <div className="flex min-w-0 items-center gap-2">
            <MonitorCog className="size-4 shrink-0 text-muted-foreground" />
            <div className="min-w-0">
              <p className="truncate font-mono text-sm font-semibold">
                {t("agentChat.workbench.idle.loopTitle")}
              </p>
              <p className="truncate text-xs text-muted-foreground">
                {t("agentChat.workbench.idle.previewTitle")}
              </p>
            </div>
          </div>
          <StatePill state={isStreaming ? "working" : "idle"} />
        </div>
        <div className="grid min-h-100 grid-rows-[auto_minmax(0,1fr)_auto]">
          <div className="border-b border-border bg-white px-5 py-3 dark:bg-background">
            <div className="flex items-center justify-between gap-3">
              <span className="text-xs font-medium text-muted-foreground">
                {t("agentChat.workbench.businessInput")}
              </span>
            </div>
            <p className="mt-1 line-clamp-2 text-sm/6 text-foreground">
              {lastTask ?? description}
            </p>
          </div>
          <div className="min-h-0 bg-white p-3 min-[1500px]:p-5 dark:bg-background">
            <div className="flex h-full min-h-72 flex-col overflow-hidden rounded-lg border border-border bg-[#fbfbfa] dark:bg-card">
              <div className="flex items-center justify-between gap-2 border-b border-border bg-[#f7f7f4] px-4 py-2.5 dark:bg-card">
                <div className="flex min-w-0 items-center gap-2">
                  <FileText className="size-4 text-muted-foreground" />
                  <span className="truncate text-sm font-medium">
                    {t("agentChat.workbench.idle.previewFile")}
                  </span>
                </div>
                <span className="shrink-0 rounded-full bg-secondary px-2 py-1 text-[11px] text-secondary-foreground">
                  {t("agentChat.workbench.idle.pending")}
                </span>
              </div>
              <div className="min-h-0 flex-1 overflow-hidden py-2 font-mono text-[11px]/5">
                {[
                  `# ${title}`,
                  description,
                  ...lines,
                ].map((line, index) => (
                  <div
                    key={`${index}-${line}`}
                    className="grid grid-cols-[2.5rem_minmax(0,1fr)] gap-3 px-3 py-0.5"
                  >
                    <span className="text-right text-muted-foreground/70 tabular-nums select-none">
                      {String(index + 1).padStart(2, "0")}
                    </span>
                    <span className="min-w-0 wrap-break-word text-foreground/85">
                      {line}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>
          <div className="grid gap-2 border-t border-border bg-[#f5f5f2] p-3 min-[1500px]:grid-cols-4 dark:bg-card">
            {WORKSPACE_LANES.map((lane) => (
              <div
                key={lane.labelKey}
                className="min-w-0 rounded-md border border-border bg-background px-3 py-2"
              >
                <p className="text-xs text-muted-foreground">
                  {t(lane.labelKey)}
                </p>
                <p className="mt-1 truncate text-xs font-medium min-[1500px]:text-sm">
                  {t(lane.valueKey)}
                </p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

function StatePill({ state }: { state: SwarmState }) {
  const { t } = useTranslation()
  const label = getStatePillLabel(state, t)

  return (
    <span
      className={`shrink-0 rounded-full px-2 py-0.5 text-[11px] font-medium ${
        state === "done"
          ? "bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-200"
          : state === "working"
            ? "bg-blue-50 text-blue-700 dark:bg-blue-950/40 dark:text-blue-200"
            : state === "queued"
              ? "bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-200"
              : state === "failed"
                ? "bg-destructive/10 text-destructive"
                : state === "reworking"
                  ? "bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-200"
                  : "bg-secondary text-secondary-foreground"
      }`}
    >
      {label}
    </span>
  )
}

function getStatePillLabel(state: SwarmState, t: TFunction) {
  if (state === "done") return t("agentChat.swarm.statePill.done")
  if (state === "working") return t("agentChat.swarm.statePill.working")
  if (state === "queued") return t("agentChat.swarm.statePill.queued")
  if (state === "failed") return t("agentChat.swarm.statePill.failed")
  if (state === "reworking") return t("agentChat.swarm.statePill.reworking")
  if (state === "skipped") return t("agentChat.swarm.statePill.skipped")
  return t("agentChat.swarm.statePill.idle")
}

function getAgentPresets(t: TFunction): SwarmAgent[] {
  return AGENTS.map((agent) => ({
    dependsOn: agent.dependsOn,
    icon: agent.icon,
    name: t(agent.nameKey),
    nodeId: agent.nodeId,
    objective: t(agent.objectiveKey),
    role: agent.role,
    summary: t(agent.summaryKey),
    tone: agent.tone,
  }))
}

function getAgents(graphNodes: AgentTaskNode[], t: TFunction): SwarmAgent[] {
  const presets = getAgentPresets(t)
  if (!graphNodes.length) return presets

  const dynamicAgents = graphNodes.map((node) => {
    const nodeId = readId(node.id)
    const preset = presets.find((agent) => agent.nodeId === nodeId)
    return {
      nodeId,
      name: node.title,
      role: node.role,
      summary: preset?.summary ?? node.objective,
      icon: preset?.icon ?? Bot,
      tone:
        preset?.tone ??
        "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-200",
      objective: node.objective,
      dependsOn: node.input_refs.map(readId),
    }
  })

  return [
    ...dynamicAgents,
    presets.find((agent) => agent.nodeId === "delivery") ?? presets.at(-1)!,
  ]
}

function getPhases(graphNodes: AgentTaskNode[], t: TFunction) {
  if (!graphNodes.length) {
    return FALLBACK_PHASES.map((phase) => ({
      detail: t(phase.detailKey),
      nodeId: phase.nodeId,
      title: t(phase.titleKey),
    }))
  }

  return [
    ...graphNodes.map((node, index) => ({
      nodeId: readId(node.id),
      title: t("agentChat.swarm.phases.dynamicTitle", {
        index: index + 1,
        title: node.title,
      }),
      detail: node.objective,
    })),
    {
      nodeId: "delivery",
      title: t("agentChat.swarm.phases.dynamicDeliveryTitle", {
        index: graphNodes.length + 1,
      }),
      detail: t("agentChat.swarm.phases.dynamicDeliveryDetail"),
    },
  ]
}

function getStageLabel({
  agents,
  hasTask,
  hasRuntime,
  isStreaming,
  runtimeNodeStates,
}: {
  agents: SwarmAgent[]
  hasTask: boolean
  hasRuntime: boolean
  isStreaming: boolean
  runtimeNodeStates: Map<string, RuntimeNodeSnapshot>
}, t: TFunction) {
  if (!hasTask) return t("agentChat.swarm.stage.waitingInput")
  if (!hasRuntime) {
    return isStreaming
      ? t("agentChat.swarm.stage.planning")
      : t("agentChat.swarm.stage.historyWaiting")
  }
  const failedAgent = agents.find(
    (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "failed"
  )
  if (failedAgent) {
    return t("agentChat.swarm.stage.failedAgent", { agent: failedAgent.name })
  }
  const reworkingAgent = agents.find(
    (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "reworking"
  )
  if (reworkingAgent) {
    return t("agentChat.swarm.stage.reworkingAgent", {
      agent: reworkingAgent.name,
    })
  }
  const workingAgent = agents.find(
    (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "working"
  )
  if (workingAgent) {
    return t("agentChat.swarm.stage.workingAgent", { agent: workingAgent.name })
  }
  if (isStreaming) return t("agentChat.swarm.stage.multiTasking")
  return t("agentChat.swarm.stage.finished")
}

function getAgentState({
  agent,
  fallbackIndex,
  hasTask,
  isStreaming,
  runtimeNodeStates,
}: {
  agent: SwarmAgent
  fallbackIndex: number
  hasTask: boolean
  isStreaming: boolean
  runtimeNodeStates: Map<string, RuntimeNodeSnapshot>
}): SwarmState {
  const runtimeState = runtimeNodeStates.get(agent.nodeId)?.state
  if (runtimeState) return runtimeState
  if (hasFailedNode(runtimeNodeStates)) return "skipped"
  if (!hasTask || runtimeNodeStates.size === 0) {
    return isStreaming && fallbackIndex === 0 ? "queued" : "idle"
  }
  if (!isStreaming) return "done"
  if (fallbackIndex <= 1) return "done"
  if (fallbackIndex <= 4) return "working"
  return "queued"
}

function getPlannedGraph(runtimeEvents: AgentRuntimeEvent[]) {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === "task_graph_planned") return event.graph
  }
  return undefined
}

function getRuntimeNodeStates(runtimeEvents: AgentRuntimeEvent[]) {
  const states = new Map<string, RuntimeNodeSnapshot>()
  const ensureNode = (nodeId: string): RuntimeNodeSnapshot => {
    const current = states.get(nodeId)
    if (current) return current
    const next: RuntimeNodeSnapshot = { evaluationReasons: [], state: "queued" }
    states.set(nodeId, next)
    return next
  }

  runtimeEvents.forEach((event) => {
    if (event.type === "task_graph_planned") {
      event.graph.nodes.forEach((node) => {
        ensureNode(readId(node.id)).state = "queued"
      })
      return
    }

    if (event.type === "task_node_ready") {
      ensureNode(event.nodeId).state = "queued"
      return
    }

    if (event.type === "task_node_started") {
      const snapshot = ensureNode(event.nodeId)
      snapshot.state = "working"
      snapshot.attempt = event.attempt
      return
    }

    if (event.type === "task_node_reported") {
      const snapshot = ensureNode(event.nodeId)
      snapshot.report = event.report
      snapshot.state = event.report.status === "failed" ? "failed" : "done"
      return
    }

    if (event.type === "task_node_evaluated") {
      const nodeId = readId(event.evaluation.node_id)
      const snapshot = ensureNode(nodeId)
      snapshot.evaluationReasons = event.evaluation.reasons
      snapshot.state = event.evaluation.passed ? "done" : snapshot.state
      return
    }

    if (event.type === "task_node_rework_requested") {
      const snapshot = ensureNode(event.nodeId)
      snapshot.state = "reworking"
      snapshot.attempt = event.attempt
      snapshot.evaluationReasons = [event.instruction]
      return
    }

    if (event.type === "task_node_passed") {
      ensureNode(event.nodeId).state = "done"
      return
    }

    if (event.type === "task_node_failed") {
      const snapshot = ensureNode(event.nodeId)
      snapshot.state = "failed"
      snapshot.evaluationReasons = [event.reason]
      return
    }
  })

  return states
}

function getCompletedCount(
  agents: SwarmAgent[],
  runtimeNodeStates: Map<string, RuntimeNodeSnapshot>
) {
  if (runtimeNodeStates.size === 0) return 0
  return agents.filter(
    (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "done"
  ).length
}

function getTimeline(
  runtimeEvents: AgentRuntimeEvent[],
  agents: SwarmAgent[],
  hasTask: boolean,
  isStreaming: boolean,
  t: TFunction
): SwarmTimelineItem[] {
  if (!runtimeEvents.length) {
    if (isStreaming) {
      return [
        { label: t("agentChat.swarm.timeline.planning.0"), tone: "default" },
        {
          label: t("agentChat.swarm.timeline.planning.1"),
          tone: "default",
        },
      ]
    }
    if (hasTask) {
      return [
        { label: t("agentChat.swarm.timeline.history.0"), tone: "default" },
        { label: t("agentChat.swarm.timeline.history.1"), tone: "default" },
      ]
    }
    return [
      { label: t("agentChat.swarm.timeline.empty.0"), tone: "default" },
      { label: t("agentChat.swarm.timeline.empty.1"), tone: "default" },
      { label: t("agentChat.swarm.timeline.empty.2"), tone: "default" },
    ]
  }

  return runtimeEvents
    .map((event): SwarmTimelineItem | null => {
      if (event.type === "commander_thinking") {
        return {
          label: t("agentChat.swarm.timeline.events.commanderThinking"),
          tone: "default",
        }
      }
      if (event.type === "task_graph_planned") {
        return {
          label: t("agentChat.swarm.timeline.events.graphPlanned", {
            count: event.graph.nodes.length,
          }),
          tone: "success",
        }
      }
      if (event.type === "task_node_started") {
        return {
          label: t("agentChat.swarm.timeline.events.nodeStarted", {
            agent: getAgentName(event.nodeId, agents),
            attempt: event.attempt,
          }),
          tone: "default",
        }
      }
      if (event.type === "task_node_rework_requested") {
        return {
          label: t("agentChat.swarm.timeline.events.nodeRework", {
            agent: getAgentName(event.nodeId, agents),
            instruction: event.instruction,
          }),
          tone: "warning",
        }
      }
      if (event.type === "task_node_passed") {
        return {
          label: t("agentChat.swarm.timeline.events.nodePassed", {
            agent: getAgentName(event.nodeId, agents),
          }),
          tone: "success",
        }
      }
      if (event.type === "task_node_failed") {
        return {
          label: t("agentChat.swarm.timeline.events.nodeFailed", {
            agent: getAgentName(event.nodeId, agents),
            reason: event.reason,
          }),
          tone: "error",
        }
      }
      if (event.type === "task_graph_finished") {
        return {
          label: t("agentChat.swarm.timeline.events.graphFinished"),
          tone: "success",
        }
      }
      return null
    })
    .filter((item): item is SwarmTimelineItem => item !== null)
    .slice(-8)
}

function getComputerCards({
  activeAgent,
  activeNode,
  agents,
  lastTask,
  runtimeEvents,
}: {
  activeAgent: SwarmAgent
  activeNode?: RuntimeNodeSnapshot
  agents: SwarmAgent[]
  lastTask?: string
  runtimeEvents: AgentRuntimeEvent[]
}, t: TFunction): ComputerCard[] {
  const cards: ComputerCard[] = []
  const graphEvent = runtimeEvents.find(
    (
      event
    ): event is Extract<AgentRuntimeEvent, { type: "task_graph_planned" }> =>
      event.type === "task_graph_planned"
  )

  if (graphEvent) {
    cards.push({
      id: "graph-planned",
      title: t("agentChat.swarm.cards.readTarget.title"),
      meta: t("agentChat.swarm.cards.readTarget.planMeta"),
      description: t("agentChat.swarm.cards.readTarget.plannedDescription", {
        count: graphEvent.graph.nodes.length,
      }),
      icon: FileText,
      lines: [
        graphEvent.graph.root_task,
        ...graphEvent.graph.nodes
          .slice(0, 3)
          .map((node) => `${node.title}: ${node.objective}`),
      ],
      state: "done",
    })
  } else {
    cards.push({
      id: "task-input",
      title: t("agentChat.swarm.cards.readTarget.title"),
      meta: t("agentChat.swarm.cards.readTarget.userMeta"),
      description: t("agentChat.swarm.cards.readTarget.inputDescription"),
      icon: FileText,
      lines: [lastTask || t("agentChat.swarm.cards.readTarget.waitingLine")],
      state: "working",
    })
  }

  const latestNodeEvent = findLatestNodeEvent(runtimeEvents)
  cards.push({
    id: "runtime-terminal",
    title: t("agentChat.swarm.cards.assign.title"),
    meta: t("agentChat.swarm.cards.assign.meta"),
    description: latestNodeEvent
      ? formatRuntimeEventDescription(latestNodeEvent, agents, t)
      : t("agentChat.swarm.cards.assign.description"),
    icon: TerminalSquare,
    lines: [
      t("agentChat.swarm.cards.assign.currentTask", {
        agent: activeAgent.name,
      }),
      t("agentChat.swarm.cards.assign.stateLine", {
        state: getDockStateLabel(activeNode?.state ?? "working", t),
      }),
      latestNodeEvent
        ? t("agentChat.swarm.cards.assign.latestAction", {
            action: getRuntimeActionLabel(latestNodeEvent.type, t),
          })
        : t("agentChat.swarm.cards.assign.latestPlanning"),
    ],
    state: activeNode?.state ?? "working",
  })

  const reportLines = readReportLines(activeNode?.report, t)
  cards.push({
    id: "agent-output",
    title: activeNode?.report
      ? t("agentChat.swarm.cards.output.reportTitle")
      : t("agentChat.swarm.cards.output.materialTitle"),
    meta: activeNode?.report
      ? activeAgent.name
      : t("agentChat.swarm.cards.output.materialMeta"),
    description: activeNode?.report
      ? getFallbackResponse(activeAgent, activeNode.state, t)
      : t("agentChat.swarm.cards.output.description"),
    icon: activeNode?.report ? Bot : Search,
    lines: reportLines.length
      ? reportLines
      : (t("agentChat.swarm.cards.output.fallbackLines", {
          returnObjects: true,
        }) as string[]),
    state: activeNode?.state ?? "queued",
  })

  const reworkEvent = findLatestReworkEvent(runtimeEvents)
  if (reworkEvent) {
    cards.push({
      id: "rework",
      title: t("agentChat.swarm.cards.rework.title"),
      meta: getAgentName(reworkEvent.nodeId, agents),
      description: t("agentChat.swarm.cards.rework.description"),
      icon: SearchCheck,
      lines: [reworkEvent.instruction],
      state: "reworking",
    })
  }

  const failedEvent = findLatestFailedEvent(runtimeEvents)
  if (failedEvent) {
    cards.push({
      id: "failed",
      title: t("agentChat.swarm.cards.failed.title"),
      meta: getAgentName(failedEvent.nodeId, agents),
      description: t("agentChat.swarm.cards.failed.description"),
      icon: TerminalSquare,
      lines: [failedEvent.reason],
      state: "failed",
    })
  }

  return cards.slice(-5)
}

function getComputerScreenContext({
  activeAgent,
  activeNode,
  runtimeEvents,
}: {
  activeAgent: SwarmAgent
  activeNode?: RuntimeNodeSnapshot
  runtimeEvents: AgentRuntimeEvent[]
}, t: TFunction): ComputerScreenContext {
  const graphNode = getActiveGraphNode(runtimeEvents, activeAgent.nodeId)
  const reportLines = readReportDocumentLines(activeNode?.report, t)
  const request = graphNode?.objective ?? activeAgent.objective
  const acceptanceLines =
    graphNode?.acceptance.map(
      (item, index) =>
        t("agentChat.swarm.screen.acceptanceLine", {
          index: index + 1,
          item,
        })
    ) ?? []
  const lines = compactPreviewLines([
    `# ${activeAgent.name}`,
    t("agentChat.swarm.screen.responsibilityLine", {
      summary: activeAgent.summary,
    }),
    t("agentChat.swarm.screen.objectiveLine", { request }),
    ...acceptanceLines,
    ...reportLines,
    ...(activeNode?.evaluationReasons ?? []).map(
      (reason) => t("agentChat.swarm.screen.reviewLine", { reason })
    ),
  ])

  return {
    filename: getComputerFilename(activeAgent, t),
    lines,
    mode: activeNode?.report
      ? t("agentChat.swarm.screen.reportMode")
      : t("agentChat.swarm.screen.taskMode"),
    request,
  }
}

function getActiveGraphNode(
  runtimeEvents: AgentRuntimeEvent[],
  nodeId: string
) {
  return getPlannedGraph(runtimeEvents)?.nodes.find(
    (node) => readId(node.id) === nodeId
  )
}

function getComputerFilename(agent: SwarmAgent, t: TFunction) {
  if (agent.nodeId === "data") return "research.dimension.report.md"
  if (agent.nodeId === "accounting") return "budget.risk.assessment.xlsx"
  if (agent.nodeId === "douyin_ads") return "douyin.ads.launch.plan.xlsx"
  if (agent.nodeId === "creative") return "content.script.matrix.md"
  if (agent.nodeId === "ops") return "operation.execution.plan.md"
  if (agent.nodeId === "delivery") {
    return t("agentChat.swarm.screen.deliveryFilename")
  }
  return `${agent.role}.agent.output.md`
}

function readReportDocumentLines(report: AgentWorkerReport | undefined, t: TFunction) {
  if (!report) return []
  return compactPreviewLines([report.content, ...readReportLines(report, t)])
}

function compactPreviewLines(lines: Array<string | undefined | null>) {
  const seen = new Set<string>()

  return lines
    .flatMap((line) => String(line ?? "").split(/\r?\n/))
    .map((line) => line.trim())
    .filter((line) => {
      if (!line || seen.has(line)) return false
      seen.add(line)
      return true
    })
    .slice(0, 18)
}

function findLatestNodeEvent(runtimeEvents: AgentRuntimeEvent[]) {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (
      event.type === "task_node_started" ||
      event.type === "task_node_reported" ||
      event.type === "task_node_passed" ||
      event.type === "task_node_failed" ||
      event.type === "task_node_rework_requested"
    ) {
      return event
    }
  }
  return null
}

function findLatestReworkEvent(runtimeEvents: AgentRuntimeEvent[]) {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === "task_node_rework_requested") return event
  }
  return null
}

function findLatestFailedEvent(runtimeEvents: AgentRuntimeEvent[]) {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === "task_node_failed") return event
  }
  return null
}

function formatRuntimeEventDescription(
  event: NonNullable<ReturnType<typeof findLatestNodeEvent>>,
  agents: SwarmAgent[],
  t: TFunction
) {
  if (event.type === "task_node_started") {
    return t("agentChat.swarm.runtimeDescriptions.started", {
      agent: getAgentName(event.nodeId, agents),
      attempt: event.attempt,
    })
  }
  if (event.type === "task_node_reported") {
    return t("agentChat.swarm.runtimeDescriptions.reported", {
      agent: getAgentName(event.nodeId, agents),
    })
  }
  if (event.type === "task_node_passed") {
    return t("agentChat.swarm.runtimeDescriptions.passed", {
      agent: getAgentName(event.nodeId, agents),
    })
  }
  if (event.type === "task_node_failed") {
    return t("agentChat.swarm.runtimeDescriptions.failed", {
      agent: getAgentName(event.nodeId, agents),
    })
  }
  return t("agentChat.swarm.runtimeDescriptions.reworking", {
    agent: getAgentName(event.nodeId, agents),
  })
}

function getRuntimeActionLabel(eventType: AgentRuntimeEvent["type"], t: TFunction) {
  if (eventType === "commander_thinking") {
    return t("agentChat.swarm.runtimeActions.commanderThinking")
  }
  if (eventType === "task_graph_planned") {
    return t("agentChat.swarm.runtimeActions.graphPlanned")
  }
  if (eventType === "task_node_ready") {
    return t("agentChat.swarm.runtimeActions.nodeReady")
  }
  if (eventType === "task_node_started") {
    return t("agentChat.swarm.runtimeActions.nodeStarted")
  }
  if (eventType === "task_node_reported") {
    return t("agentChat.swarm.runtimeActions.nodeReported")
  }
  if (eventType === "task_node_evaluated") {
    return t("agentChat.swarm.runtimeActions.nodeEvaluated")
  }
  if (eventType === "task_node_rework_requested") {
    return t("agentChat.swarm.runtimeActions.nodeRework")
  }
  if (eventType === "task_node_passed") {
    return t("agentChat.swarm.runtimeActions.nodePassed")
  }
  if (eventType === "task_node_failed") {
    return t("agentChat.swarm.runtimeActions.nodeFailed")
  }
  if (eventType === "task_graph_finished") {
    return t("agentChat.swarm.runtimeActions.graphFinished")
  }
  return t("agentChat.swarm.runtimeActions.synced")
}

function readReportLines(report: AgentWorkerReport | undefined, t: TFunction) {
  if (!report) return []
  const roleOutput = report.role_output
  return [
    roleOutput?.summary,
    ...(roleOutput?.findings ?? []),
    ...(roleOutput?.recommendations ?? []),
    ...(roleOutput?.evidence ?? []),
    ...(report.risks ?? []).map((risk) =>
      t("agentChat.swarm.reportLines.risk", { risk })
    ),
    ...(report.open_questions ?? []).map((question) =>
      t("agentChat.swarm.reportLines.question", { question })
    ),
  ].filter((line): line is string => Boolean(line))
}

function getAgentName(nodeId: string, agents: SwarmAgent[]) {
  return agents.find((agent) => agent.nodeId === nodeId)?.name ?? nodeId
}

function getFallbackResponse(agent: SwarmAgent, state: SwarmState, t: TFunction) {
  if (state === "failed") {
    return t("agentChat.swarm.fallbackResponse.failed")
  }
  if (state === "reworking") {
    return t("agentChat.swarm.fallbackResponse.reworking")
  }
  if (state === "working")
    return t("agentChat.swarm.fallbackResponse.working", {
      agent: agent.name,
    })
  if (state === "done") {
    return t("agentChat.swarm.fallbackResponse.done", { agent: agent.name })
  }
  return t("agentChat.swarm.fallbackResponse.idle")
}

function getDockStateLabel(state: SwarmState, t: TFunction) {
  if (state === "done") return t("agentChat.swarm.dockState.done")
  if (state === "working") return t("agentChat.swarm.dockState.working")
  if (state === "reworking") return t("agentChat.swarm.dockState.reworking")
  if (state === "failed") return t("agentChat.swarm.dockState.failed")
  if (state === "queued") return t("agentChat.swarm.dockState.queued")
  if (state === "skipped") return t("agentChat.swarm.dockState.skipped")
  return t("agentChat.swarm.dockState.idle")
}

function hasFailedNode(runtimeNodeStates: Map<string, RuntimeNodeSnapshot>) {
  return Array.from(runtimeNodeStates.values()).some(
    (snapshot) => snapshot.state === "failed"
  )
}

function formatAgentNumber(index: number) {
  return String(Math.max(index + 1, 1)).padStart(2, "0")
}

function readId(value: string | [string] | { 0: string }) {
  if (typeof value === "string") return value
  if (Array.isArray(value)) return value[0]
  return value[0]
}
