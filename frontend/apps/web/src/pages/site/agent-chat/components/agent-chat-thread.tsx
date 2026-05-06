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
  Search,
  ThumbsDown,
  ThumbsUp,
  TerminalSquare,
  UserRound,
} from "lucide-react"
import { useEffect, useRef } from "react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent, AgentTaskNode } from "@/api"
import type { AgentChatMessage } from "../types"

export interface AgentChatThreadProps {
  isStreaming: boolean
  messages: AgentChatMessage[]
  runtimeEvents: AgentRuntimeEvent[]
}

type InlineAgentState = "idle" | "queued" | "working" | "done" | "failed" | "reworking"

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

export function AgentChatThread({
  isStreaming,
  messages,
  runtimeEvents,
}: AgentChatThreadProps) {
  const { t } = useTranslation()
  const endRef = useRef<HTMLDivElement>(null)
  const latestUserTask = messages
    .filter((message) => message.role === "user")
    .at(-1)?.content

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" })
  }, [messages])

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
                className={`min-w-0 max-w-[min(43rem,88%)] text-sm/6 ${
                  isUser
                    ? "rounded-2xl bg-foreground px-4 py-3 text-background"
                    : "text-foreground"
                }`}
              >
                {isWelcome ? (
                  <WelcomeCard content={message.content} />
                ) : message.content ? (
                  <p className="whitespace-pre-wrap break-words">{message.content}</p>
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
          <InlineSwarmTaskCard
            isStreaming={
              isStreaming || messages.some((message) => message.status === "streaming")
            }
            runtimeEvents={runtimeEvents}
            task={latestUserTask}
          />
        ) : null}
        <div ref={endRef} />
      </div>
    </section>
  )
}

function WelcomeCard({ content }: { content: string }) {
  return (
    <div className="rounded-xl border border-border bg-card p-5 shadow-sm">
      <div className="flex items-start gap-3">
        <div className="flex size-10 shrink-0 items-center justify-center rounded-md bg-foreground text-background">
          <BrainCircuit className="size-5" />
        </div>
        <div className="min-w-0">
          <p className="text-base font-semibold">Commerce 运营智能体</p>
          <p className="mt-1 text-sm/6 text-muted-foreground">{content}</p>
        </div>
      </div>
      <div className="mt-4 grid gap-2 sm:grid-cols-3">
        {[
          "拆解任务图",
          "并行调度角色",
          "沉淀交付产物",
        ].map((item) => (
          <div key={item} className="flex items-center gap-2 rounded-md bg-muted px-3 py-2 text-xs text-muted-foreground">
            <ListChecks className="size-3.5" />
            {item}
          </div>
        ))}
      </div>
    </div>
  )
}

function AgentProgressCard({ content }: { content?: string }) {
  return (
    <div className="mt-4 flex items-center gap-2 rounded-lg border border-border bg-card px-3 py-2 text-xs text-muted-foreground">
      <Loader2 className="size-4 animate-spin" />
      <span>
        {content
          ? "正在接收多角色协作输出"
          : "正在等待任务图和角色执行事件"}
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
  const agents = getInlineAgents(runtimeEvents, isStreaming)
  const steps = getInlineSteps(runtimeEvents, isStreaming)
  const completedCount = agents.filter((agent) => agent.state === "done").length
  const hasRuntime = runtimeEvents.length > 0
  const activeAgent = agents.find((agent) =>
    agent.state === "working" || agent.state === "reworking" || agent.state === "failed"
  )
  const artifacts = getInlineArtifacts(agents, hasRuntime, isStreaming)
  const progressLabel = hasRuntime
    ? `${completedCount}/${agents.length} 个任务完成`
    : isStreaming
      ? "正在生成任务图"
      : "历史消息未执行"

  return (
    <article className="flex justify-start gap-3">
      <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-card text-foreground">
        <GitBranch className="size-5" />
      </div>
      <div className="min-w-0 max-w-[min(43rem,92%)] space-y-4">
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
                  <span className="absolute left-6 top-11 h-5 border-l border-dashed border-border" />
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
                智能体协作
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
                  ? "正在调度"
                  : "本轮调度已同步"
                : isStreaming
                  ? "正在规划"
                  : "等待新一轮调度"}
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
                <span>阶段资料</span>
                <span className="truncate text-xs font-normal text-muted-foreground">
                  {artifacts.filter((artifact) => artifact.state === "done").length}/
                  {artifacts.length}
                </span>
              </div>
              <span className="rounded-full bg-secondary px-2 py-1 text-[11px] text-secondary-foreground">
                {hasRuntime ? "随调度更新" : isStreaming ? "准备生成" : "待生成"}
              </span>
            </div>
            <div className="mt-3 grid gap-2">
              {artifacts.map((artifact) => (
                <InlineArtifactCard key={artifact.filename} artifact={artifact} />
              ))}
            </div>
          </div>

          <div className="mt-3 grid gap-2 sm:grid-cols-3">
            {["任务拆图", "角色并行", "方案汇总"].map((item) => (
              <div
                key={item}
                className="rounded-md border border-border bg-background px-3 py-2 text-xs text-muted-foreground"
              >
                {item}
              </div>
            ))}
          </div>
        </div>
      </div>
    </article>
  )
}

function InlineArtifactCard({ artifact }: { artifact: InlineArtifact }) {
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
        {getInlineArtifactStateLabel(artifact.state)}
      </span>
    </div>
  )
}

function FileCardIcon({ className }: { className?: string }) {
  return <ListChecks className={className} />
}

function MessageActions() {
  const actions = [
    { label: "复制", icon: Copy },
    { label: "有用", icon: ThumbsUp },
    { label: "无用", icon: ThumbsDown },
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

const INLINE_AGENT_PRESETS: InlineAgent[] = [
  {
    nodeId: "data",
    name: "数据诊断",
    summary: "先识别指标、归因和经营约束",
    status: "待命",
    state: "idle",
  },
  {
    nodeId: "accounting",
    name: "预算评估",
    summary: "核算预算、SKU 和风险边界",
    status: "待命",
    state: "idle",
  },
  {
    nodeId: "douyin_ads",
    name: "千川投放",
    summary: "拆解计划结构、人群包和 ROI 止损",
    status: "待命",
    state: "idle",
  },
  {
    nodeId: "creative",
    name: "创意产出",
    summary: "生成短视频、直播和活动内容方向",
    status: "待命",
    state: "idle",
  },
  {
    nodeId: "ops",
    name: "运营方案",
    summary: "整合上游结论形成执行节奏",
    status: "待命",
    state: "idle",
  },
  {
    nodeId: "delivery",
    name: "交付文档",
    summary: "沉淀方案、清单和后续产物",
    status: "待命",
    state: "idle",
  },
]

function getInlineAgents(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean
): InlineAgent[] {
  const graphNodes = getInlineGraphNodes(runtimeEvents)
  const baseAgents = graphNodes.length
    ? [
        ...graphNodes.map(mapGraphNodeToInlineAgent),
        INLINE_AGENT_PRESETS.find((agent) => agent.nodeId === "delivery")!,
      ]
    : INLINE_AGENT_PRESETS
  const stateMap = getInlineNodeStates(runtimeEvents)

  return baseAgents.map((agent, index) => {
    const state = stateMap.get(agent.nodeId) ?? getFallbackInlineState(index, isStreaming)
    return {
      ...agent,
      state,
      status: getInlineStateLabel(state),
    }
  })
}

function getInlineSteps(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean
) {
  const hasGraph = runtimeEvents.some((event) => event.type === "task_graph_planned")
  const hasNodeStarted = runtimeEvents.some((event) => event.type === "task_node_started")

  return [
    [
      "读取目标",
      hasGraph
        ? "角色任务图已生成"
        : isStreaming
          ? "正在读取经营目标与角色规则"
          : "经营目标与角色规则",
    ],
    [
      "调度角色",
      hasNodeStarted
        ? "角色任务正在执行"
        : isStreaming
          ? "等待执行状态同步"
          : "检查模型、预算与投放约束",
    ],
    [
      "补充资料",
      hasNodeStarted ? "按角色补充平台资料" : "补充行业与平台场景资料",
    ],
  ]
}

function getInlineArtifacts(
  agents: InlineAgent[],
  hasRuntime: boolean,
  isStreaming: boolean
): InlineArtifact[] {
  const stateByNode = new Map(agents.map((agent) => [agent.nodeId, agent.state]))
  const fallbackState = hasRuntime ? "queued" : isStreaming ? "working" : "idle"
  const specs = [
    {
      filename: "research.dimension.report.md",
      meta: "MD",
      nodeId: "data",
      status: "数据诊断、指标归因、平台场景资料",
    },
    {
      filename: "douyin.ads.launch.plan.xlsx",
      meta: "XLSX",
      nodeId: "douyin_ads",
      status: "千川计划结构、人群包、预算节奏",
    },
    {
      filename: "运营全流程执行方案.docx",
      meta: "DOCX",
      nodeId: "delivery",
      status: "最终方案、执行清单、复盘口径",
    },
  ]

  return specs.map((artifact) => ({
    ...artifact,
    state: stateByNode.get(artifact.nodeId) ?? fallbackState,
  }))
}

function getInlineArtifactStateLabel(state: InlineAgentState) {
  if (state === "done") return "已生成"
  if (state === "working" || state === "reworking") return "生成中"
  if (state === "failed") return "受阻"
  if (state === "queued") return "排队"
  return "待生成"
}

function getInlineGraphNodes(runtimeEvents: AgentRuntimeEvent[]): AgentTaskNode[] {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === "task_graph_planned") return event.graph.nodes
  }
  return []
}

function mapGraphNodeToInlineAgent(node: AgentTaskNode): InlineAgent {
  const nodeId = readInlineId(node.id)
  const preset = INLINE_AGENT_PRESETS.find((agent) => agent.nodeId === nodeId)
  return {
    nodeId,
    name: node.title || preset?.name || node.role,
    summary: preset?.summary || node.objective,
    status: "排队中",
    state: "queued",
  }
}

function getInlineNodeStates(runtimeEvents: AgentRuntimeEvent[]) {
  const states = new Map<string, InlineAgentState>()

  runtimeEvents.forEach((event) => {
    if (event.type === "task_graph_planned") {
      event.graph.nodes.forEach((node) => states.set(readInlineId(node.id), "queued"))
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
      states.set(event.nodeId, event.report.status === "failed" ? "failed" : "done")
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

function getInlineStateLabel(state: InlineAgentState) {
  if (state === "done") return "已完成"
  if (state === "working") return "处理中"
  if (state === "reworking") return "重试中"
  if (state === "failed") return "失败"
  if (state === "queued") return "排队中"
  return "待命"
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
