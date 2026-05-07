import {
  Activity,
  ArrowRight,
  Bot,
  BrainCircuit,
  CheckCircle2,
  Copy,
  FileText,
  GitBranch,
  ListChecks,
  Loader2,
  MonitorCog,
  PlugZap,
  Search,
  ShieldAlert,
  ThumbsDown,
  ThumbsUp,
  TerminalSquare,
  UserRound,
} from "lucide-react"
import { useEffect, useRef } from "react"
import { type TFunction } from "i18next"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent, AgentTaskNode } from "@/api"
import type { AgentChatMessage } from "../types"

export interface AgentChatThreadProps {
  isStreaming: boolean
  messages: AgentChatMessage[]
  processPanelOpen: boolean
  runtimeEvents: AgentRuntimeEvent[]
}

type InlineAgentState =
  | "idle"
  | "queued"
  | "working"
  | "done"
  | "failed"
  | "reworking"

interface InlineAgent {
  nodeId: string
  name: string
  summary: string
  status: string
  state: InlineAgentState
}

interface InlineArtifact {
  filename: string
  meta: string
  nodeId: string
  status: string
  state: InlineAgentState
}

const ARTIFACT_META_MD = "MD"
const ARTIFACT_META_XLSX = "XLSX"
const ARTIFACT_META_DOCX = "DOCX"

export function AgentChatThread({
  isStreaming,
  messages,
  processPanelOpen,
  runtimeEvents,
}: AgentChatThreadProps) {
  const { t } = useTranslation()
  const endRef = useRef<HTMLDivElement>(null)
  const latestUserTask = messages
    .filter((message) => message.role === "user")
    .at(-1)?.content

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" })
  }, [messages, processPanelOpen])

  return (
    <section className="min-h-0 flex-1 overflow-y-auto bg-background px-4 py-5 sm:px-6">
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-6">
        {messages.map((message) => {
          const isUser = message.role === "user"
          const isWelcome = message.id === "welcome" && !isUser

          return (
            <article
              key={message.id}
              className={`flex gap-3 ${isUser ? "justify-end" : "justify-start"}`}
            >
              {!isUser ? (
                <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-card text-foreground">
                  <Bot className="size-5" />
                </div>
              ) : null}
              <div
                className={`max-w-[min(43rem,88%)] min-w-0 text-sm/6 ${
                  isUser
                    ? "rounded-2xl bg-foreground px-4 py-3 text-background"
                    : "text-foreground"
                }`}
              >
                {isWelcome ? (
                  <WelcomeCard content={message.content} />
                ) : message.content ? (
                  <p className="wrap-break-word whitespace-pre-wrap">
                    {message.content}
                  </p>
                ) : (
                  <p className="text-muted-foreground">
                    {t("agentChat.streaming")}
                  </p>
                )}
                {message.status === "streaming" ? (
                  <AgentProgressCard content={message.content} />
                ) : null}
                {message.status === "error" ? (
                  <p className="mt-2 text-xs font-medium text-destructive">
                    {t("agentChat.errorBadge")}
                  </p>
                ) : null}
                {!isUser && message.content && !isWelcome ? (
                  <MessageActions />
                ) : null}
              </div>
              {isUser ? (
                <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-card text-muted-foreground">
                  <UserRound className="size-5" />
                </div>
              ) : null}
            </article>
          )
        })}
        {latestUserTask ? (
          <>
            <InlineSwarmTaskCard
              isStreaming={
                isStreaming ||
                messages.some((message) => message.status === "streaming")
              }
              runtimeEvents={runtimeEvents}
              task={latestUserTask}
            />
            {!processPanelOpen ? (
              <InlineProcessStatusNode
                isStreaming={
                  isStreaming ||
                  messages.some((message) => message.status === "streaming")
                }
                runtimeEvents={runtimeEvents}
              />
            ) : null}
          </>
        ) : null}
        <div ref={endRef} />
      </div>
    </section>
  )
}

function WelcomeCard({ content }: { content: string }) {
  const { t } = useTranslation()
  const capabilities = t("agentChat.thread.welcomeCapabilities", {
    returnObjects: true,
  }) as string[]

  return (
    <div className="rounded-xl border border-border bg-card p-5 shadow-sm">
      <div className="flex items-start gap-3">
        <div className="flex size-10 shrink-0 items-center justify-center rounded-md bg-foreground text-background">
          <BrainCircuit className="size-5" />
        </div>
        <div className="min-w-0">
          <p className="text-base font-semibold">
            {t("agentChat.thread.agentName")}
          </p>
          <p className="mt-1 text-sm/6 text-muted-foreground">{content}</p>
        </div>
      </div>
      <div className="mt-4 grid gap-2 sm:grid-cols-3">
        {capabilities.map((item) => (
          <div
            key={item}
            className="flex items-center gap-2 rounded-md bg-muted px-3 py-2 text-xs text-muted-foreground"
          >
            <ListChecks className="size-3.5" />
            {item}
          </div>
        ))}
      </div>
    </div>
  )
}

function AgentProgressCard({ content }: { content?: string }) {
  const { t } = useTranslation()

  return (
    <div className="mt-4 flex items-center gap-2 rounded-lg border border-border bg-card px-3 py-2 text-xs text-muted-foreground">
      <Loader2 className="size-4 animate-spin" />
      <span>
        {content
          ? t("agentChat.thread.receivingOutput")
          : t("agentChat.thread.waitingEvents")}
      </span>
    </div>
  )
}

function InlineSwarmTaskCard({
  isStreaming,
  runtimeEvents,
  task,
}: {
  isStreaming: boolean
  runtimeEvents: AgentRuntimeEvent[]
  task: string
}) {
  const { t } = useTranslation()
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
        <div className="space-y-2">
          {steps.map(([label, detail], index) => {
            const Icon =
              index === 0 ? FileCardIcon : index === 1 ? TerminalSquare : Search
            return (
              <div
                key={label}
                className="relative rounded-xl border border-border bg-card px-4 py-3 shadow-sm"
              >
                {index < steps.length - 1 ? (
                  <span className="absolute top-11 left-6 h-5 border-l border-dashed border-border" />
                ) : null}
                <div className="flex items-center justify-between gap-3">
                  <div className="flex min-w-0 items-center gap-3">
                    <div className="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground">
                      <Icon className="size-4" />
                    </div>
                    <div className="min-w-0">
                      <p className="text-sm font-medium">
                        {label}
                        <span className="ml-2 block text-xs font-normal text-muted-foreground sm:inline">
                          | {detail}
                        </span>
                      </p>
                    </div>
                  </div>
                  <ArrowRight className="size-4 shrink-0 text-muted-foreground" />
                </div>
              </div>
            )
          })}
        </div>

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
            <span className="rounded-full bg-secondary px-2 py-1 text-[11px] font-medium text-secondary-foreground">
              {runtimeEvents.length
                ? isStreaming
                  ? t("agentChat.thread.statuses.executing")
                  : t("agentChat.thread.statuses.synced")
                : isStreaming
                  ? t("agentChat.thread.statuses.planning")
                  : t("agentChat.thread.statuses.waiting")}
            </span>
          </div>

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
                <div className="hidden items-center gap-0.5 min-[1800px]:flex">
                  {Array.from({ length: 12 }).map((_, pixelIndex) => (
                    <span
                      key={pixelIndex}
                      className={`size-1 rounded-[1px] ${
                        shouldLightInlinePixel(agent.state, pixelIndex)
                          ? agent.state === "failed"
                            ? "bg-destructive"
                            : "bg-emerald-500"
                          : "bg-muted-foreground/15"
                      }`}
                    />
                  ))}
                </div>
              </div>
            ))}
          </div>

          <div className="mt-3 rounded-lg border border-border bg-background/80 p-3">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex min-w-0 items-center gap-2 text-sm font-medium">
                <FileText className="size-4 shrink-0 text-muted-foreground" />
                <span>{t("agentChat.thread.artifactsTitle")}</span>
                <span className="truncate text-xs font-normal text-muted-foreground">
                  {
                    artifacts.filter((artifact) => artifact.state === "done")
                      .length
                  }
                  /{artifacts.length}
                </span>
              </div>
              <span className="rounded-full bg-secondary px-2 py-1 text-[11px] text-secondary-foreground">
                {hasRuntime
                  ? t("agentChat.thread.artifacts.updating")
                  : isStreaming
                    ? t("agentChat.thread.artifacts.preparing")
                    : t("agentChat.thread.artifacts.pending")}
              </span>
            </div>
            <div className="mt-3 grid gap-2">
              {artifacts.map((artifact) => (
                <InlineArtifactCard
                  key={artifact.filename}
                  artifact={artifact}
                />
              ))}
            </div>
          </div>

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
        </div>
      </div>
    </article>
  )
}

function InlineProcessStatusNode({
  isStreaming,
  runtimeEvents,
}: {
  isStreaming: boolean
  runtimeEvents: AgentRuntimeEvent[]
}) {
  const { t } = useTranslation()
  const agents = getInlineAgents(runtimeEvents, isStreaming, t)
  const completedCount = agents.filter((agent) => agent.state === "done").length
  const hasRuntime = runtimeEvents.length > 0
  const progressPercent = agents.length
    ? Math.min(100, Math.round((completedCount / agents.length) * 100))
    : 0
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
                    completed: completedCount,
                    total: agents.length,
                  })}
                </span>
                <span className="hidden text-muted-foreground sm:inline">
                  |
                </span>
                <span className="min-w-0 truncate text-muted-foreground">
                  {getInlineProcessStage({
                    hasRuntime,
                    isStreaming,
                    runtimeEvents,
                  }, t)}
                </span>
              </div>
              <div className="mt-1.5 h-px overflow-hidden rounded-full bg-muted">
                <span
                  className={`block h-full rounded-full ${
                    hasFailedAgent ? "bg-destructive" : "bg-emerald-500"
                  }`}
                  style={{ width: `${progressPercent}%` }}
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </article>
  )
}

function InlineArtifactCard({ artifact }: { artifact: InlineArtifact }) {
  const { t } = useTranslation()

  return (
    <div
      className={`flex min-w-0 items-center gap-3 rounded-lg border px-3 py-2 ${
        artifact.state === "done"
          ? "border-emerald-200 bg-emerald-50/70 text-emerald-950 dark:border-emerald-900/60 dark:bg-emerald-950/20 dark:text-emerald-100"
          : artifact.state === "working" || artifact.state === "reworking"
            ? "border-foreground/25 bg-background"
            : artifact.state === "failed"
              ? "border-destructive/30 bg-destructive/10"
              : "border-border bg-card"
      }`}
    >
      <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-background text-muted-foreground">
        {artifact.state === "working" || artifact.state === "reworking" ? (
          <Loader2 className="size-4 animate-spin text-emerald-600" />
        ) : artifact.state === "done" ? (
          <CheckCircle2 className="size-4 text-emerald-600" />
        ) : (
          <FileText className="size-4" />
        )}
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 items-center gap-2">
          <p className="truncate text-sm font-medium">{artifact.filename}</p>
          <span className="shrink-0 rounded bg-muted px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
            {artifact.meta}
          </span>
        </div>
        <p className="mt-0.5 truncate text-xs text-muted-foreground">
          {artifact.status}
        </p>
      </div>
      <span className="hidden shrink-0 text-xs text-muted-foreground sm:inline">
        {getInlineArtifactStateLabel(artifact.state, t)}
      </span>
    </div>
  )
}

function FileCardIcon({ className }: { className?: string }) {
  return <ListChecks className={className} />
}

function MessageActions() {
  const { t } = useTranslation()
  const actions = [
    { label: t("agentChat.thread.actions.copy"), icon: Copy },
    { label: t("agentChat.thread.actions.useful"), icon: ThumbsUp },
    { label: t("agentChat.thread.actions.useless"), icon: ThumbsDown },
  ]

  return (
    <div className="mt-3 flex items-center gap-1 text-muted-foreground">
      {actions.map((action) => {
        const Icon = action.icon
        return (
          <button
            key={action.label}
            type="button"
            className="flex size-7 items-center justify-center rounded-md transition-colors hover:bg-muted hover:text-foreground"
            title={action.label}
          >
            <Icon className="size-4" />
          </button>
        )
      })}
    </div>
  )
}

const INLINE_AGENT_PRESETS = [
  {
    nodeId: "data",
    nameKey: "agentChat.thread.agents.data.name",
    summaryKey: "agentChat.thread.agents.data.summary",
    state: "idle",
  },
  {
    nodeId: "accounting",
    nameKey: "agentChat.thread.agents.accounting.name",
    summaryKey: "agentChat.thread.agents.accounting.summary",
    state: "idle",
  },
  {
    nodeId: "douyin_ads",
    nameKey: "agentChat.thread.agents.douyinAds.name",
    summaryKey: "agentChat.thread.agents.douyinAds.summary",
    state: "idle",
  },
  {
    nodeId: "creative",
    nameKey: "agentChat.thread.agents.creative.name",
    summaryKey: "agentChat.thread.agents.creative.summary",
    state: "idle",
  },
  {
    nodeId: "ops",
    nameKey: "agentChat.thread.agents.ops.name",
    summaryKey: "agentChat.thread.agents.ops.summary",
    state: "idle",
  },
  {
    nodeId: "delivery",
    nameKey: "agentChat.thread.agents.delivery.name",
    summaryKey: "agentChat.thread.agents.delivery.summary",
    state: "idle",
  },
]

function getInlineAgents(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean,
  t: TFunction
): InlineAgent[] {
  const graphNodes = getInlineGraphNodes(runtimeEvents)
  const presets = getInlineAgentPresets(t)
  const baseAgents = graphNodes.length
    ? [
        ...graphNodes.map((node) => mapGraphNodeToInlineAgent(node, presets, t)),
        presets.find((agent) => agent.nodeId === "delivery")!,
      ]
    : presets
  const stateMap = getInlineNodeStates(runtimeEvents)

  return baseAgents.map((agent, index) => {
    const state =
      stateMap.get(agent.nodeId) ?? getFallbackInlineState(index, isStreaming)
    return {
      ...agent,
      state,
      status: getInlineStateLabel(state, t),
    }
  })
}

function getInlineAgentPresets(t: TFunction): InlineAgent[] {
  return INLINE_AGENT_PRESETS.map((agent) => ({
    nodeId: agent.nodeId,
    name: t(agent.nameKey),
    summary: t(agent.summaryKey),
    status: t("agentChat.thread.agents.standby"),
    state: agent.state as InlineAgentState,
  }))
}

function getInlineSteps(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean,
  t: TFunction
) {
  const hasGraph = runtimeEvents.some(
    (event) => event.type === "task_graph_planned"
  )
  const hasNodeStarted = runtimeEvents.some(
    (event) => event.type === "task_node_started"
  )

  return [
    [
      t("agentChat.thread.steps.readTarget"),
      hasGraph
        ? t("agentChat.thread.steps.planReady")
        : isStreaming
          ? t("agentChat.thread.steps.readingRules")
          : t("agentChat.thread.steps.targetRules"),
    ],
    [
      t("agentChat.thread.steps.assignTask"),
      hasNodeStarted
        ? t("agentChat.thread.steps.taskRunning")
        : isStreaming
          ? t("agentChat.thread.steps.waitingSync")
          : t("agentChat.thread.steps.checkingConstraints"),
    ],
    [
      t("agentChat.thread.steps.enrichMaterial"),
      hasNodeStarted
        ? t("agentChat.thread.steps.enrichByTask")
        : t("agentChat.thread.steps.enrichFallback"),
    ],
  ]
}

function getInlineArtifacts(
  agents: InlineAgent[],
  hasRuntime: boolean,
  isStreaming: boolean,
  t: TFunction
): InlineArtifact[] {
  const stateByNode = new Map(
    agents.map((agent) => [agent.nodeId, agent.state])
  )
  const fallbackState = hasRuntime ? "queued" : isStreaming ? "working" : "idle"
  const specs = [
    {
      filename: "research.dimension.report.md",
      meta: ARTIFACT_META_MD,
      nodeId: "data",
      status: t("agentChat.thread.artifactSpecs.dataStatus"),
    },
    {
      filename: "douyin.ads.launch.plan.xlsx",
      meta: ARTIFACT_META_XLSX,
      nodeId: "douyin_ads",
      status: t("agentChat.thread.artifactSpecs.adsStatus"),
    },
    {
      filename: t("agentChat.thread.artifactSpecs.deliveryFile"),
      meta: ARTIFACT_META_DOCX,
      nodeId: "delivery",
      status: t("agentChat.thread.artifactSpecs.deliveryStatus"),
    },
  ]

  return specs.map((artifact) => ({
    ...artifact,
    state: stateByNode.get(artifact.nodeId) ?? fallbackState,
  }))
}

function getCommerceHandoffItems(t: TFunction) {
  return [
    {
      icon: PlugZap,
      label: t("agentChat.thread.handoff.context.label"),
      detail: t("agentChat.thread.handoff.context.detail"),
    },
    {
      icon: ListChecks,
      label: t("agentChat.thread.handoff.execution.label"),
      detail: t("agentChat.thread.handoff.execution.detail"),
    },
    {
      icon: FileText,
      label: t("agentChat.thread.handoff.delivery.label"),
      detail: t("agentChat.thread.handoff.delivery.detail"),
    },
    {
      icon: ShieldAlert,
      label: t("agentChat.thread.handoff.governance.label"),
      detail: t("agentChat.thread.handoff.governance.detail"),
    },
  ]
}

function getInlineProcessStage({
  hasRuntime,
  isStreaming,
  runtimeEvents,
}: {
  hasRuntime: boolean
  isStreaming: boolean
  runtimeEvents: AgentRuntimeEvent[]
}, t: TFunction) {
  if (runtimeEvents.some((event) => event.type === "task_node_failed")) {
    return t("agentChat.thread.processStage.failed")
  }
  if (runtimeEvents.some((event) => event.type === "task_graph_finished")) {
    return t("agentChat.thread.processStage.finished")
  }
  if (isStreaming) {
    return hasRuntime
      ? t("agentChat.thread.processStage.running")
      : t("agentChat.thread.processStage.planning")
  }
  return hasRuntime
    ? t("agentChat.thread.processStage.synced")
    : t("agentChat.thread.processStage.history")
}

function getInlineArtifactStateLabel(state: InlineAgentState, t: TFunction) {
  if (state === "done") return t("agentChat.thread.artifactState.done")
  if (state === "working" || state === "reworking") {
    return t("agentChat.thread.artifactState.working")
  }
  if (state === "failed") return t("agentChat.thread.artifactState.failed")
  if (state === "queued") return t("agentChat.thread.artifactState.queued")
  return t("agentChat.thread.artifactState.idle")
}

function getInlineGraphNodes(
  runtimeEvents: AgentRuntimeEvent[]
): AgentTaskNode[] {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === "task_graph_planned") return event.graph.nodes
  }
  return []
}

function mapGraphNodeToInlineAgent(
  node: AgentTaskNode,
  presets: InlineAgent[],
  t: TFunction
): InlineAgent {
  const nodeId = readInlineId(node.id)
  const preset = presets.find((agent) => agent.nodeId === nodeId)
  return {
    nodeId,
    name: node.title || preset?.name || node.role,
    summary: preset?.summary || node.objective,
    status: t("agentChat.thread.inlineState.queued"),
    state: "queued",
  }
}

function getInlineNodeStates(runtimeEvents: AgentRuntimeEvent[]) {
  const states = new Map<string, InlineAgentState>()

  runtimeEvents.forEach((event) => {
    if (event.type === "task_graph_planned") {
      event.graph.nodes.forEach((node) =>
        states.set(readInlineId(node.id), "queued")
      )
      return
    }
    if (event.type === "task_node_ready") {
      states.set(event.nodeId, "queued")
      return
    }
    if (event.type === "task_node_started") {
      states.set(event.nodeId, "working")
      return
    }
    if (event.type === "task_node_reported") {
      states.set(
        event.nodeId,
        event.report.status === "failed" ? "failed" : "done"
      )
      return
    }
    if (event.type === "task_node_evaluated") {
      const nodeId = readInlineId(event.evaluation.node_id)
      if (event.evaluation.passed) states.set(nodeId, "done")
      return
    }
    if (event.type === "task_node_rework_requested") {
      states.set(event.nodeId, "reworking")
      return
    }
    if (event.type === "task_node_passed") {
      states.set(event.nodeId, "done")
      return
    }
    if (event.type === "task_node_failed") {
      states.set(event.nodeId, "failed")
    }
  })

  return states
}

function getFallbackInlineState(
  index: number,
  isStreaming: boolean
): InlineAgentState {
  if (!isStreaming) return "idle"
  if (index === 0) return "working"
  if (index <= 3) return "queued"
  return "idle"
}

function getInlineStateLabel(state: InlineAgentState, t: TFunction) {
  if (state === "done") return t("agentChat.thread.inlineState.done")
  if (state === "working") return t("agentChat.thread.inlineState.working")
  if (state === "reworking") return t("agentChat.thread.inlineState.reworking")
  if (state === "failed") return t("agentChat.thread.inlineState.failed")
  if (state === "queued") return t("agentChat.thread.inlineState.queued")
  return t("agentChat.thread.inlineState.idle")
}

function shouldLightInlinePixel(state: InlineAgentState, index: number) {
  if (state === "done") return index < 12
  if (state === "working" || state === "reworking") return index < 5
  if (state === "failed") return index < 4
  if (state === "queued") return index < 2
  return false
}

function readInlineId(value: string | [string] | { 0: string }) {
  if (typeof value === "string") return value
  if (Array.isArray(value)) return value[0]
  return value[0]
}
