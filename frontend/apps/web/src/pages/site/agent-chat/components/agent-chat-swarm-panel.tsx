import {
  Activity,
  BarChart3,
  Bot,
  ChevronRight,
  CheckCircle2,
  ClipboardCheck,
  FileText,
  Loader2,
  Megaphone,
  MonitorCog,
  PanelRightClose,
  PenLine,
  Search,
  SearchCheck,
  Sparkles,
  TerminalSquare,
  WalletCards,
} from "lucide-react"
import type { ComponentType, SVGProps } from "react"
import type {
  AgentLlmConfigResponse,
  AgentRuntimeEvent,
  AgentTaskNode,
  AgentWorkerReport,
} from "@/api"
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
  llmConfig: AgentLlmConfigResponse | null
  messages: AgentChatMessage[]
  onCollapse: () => void
  runtimeEvents: AgentRuntimeEvent[]
  selectedModelName: string | null
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

const AGENTS: SwarmAgent[] = [
  {
    nodeId: "data",
    name: "数据诊断",
    role: "data",
    summary: "分析数据、指标、归因和业务约束",
    icon: BarChart3,
    tone: "bg-blue-50 text-blue-700 dark:bg-blue-950/40 dark:text-blue-200",
    objective: "分析任务中的数据、指标、归因和约束",
    dependsOn: [],
  },
  {
    nodeId: "accounting",
    name: "预算评估",
    role: "accounting",
    summary: "评估成本、预算、SKU 和经营风险",
    icon: WalletCards,
    tone: "bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-200",
    objective: "基于数据诊断评估活动成本、引流 SKU、投流预算和风险",
    dependsOn: ["data"],
  },
  {
    nodeId: "douyin_ads",
    name: "千川投放优化",
    role: "douyin_ads",
    summary: "计划结构、人群包、预算节奏和 ROI 止损",
    icon: Megaphone,
    tone: "bg-cyan-50 text-cyan-700 dark:bg-cyan-950/40 dark:text-cyan-200",
    objective: "拆解巨量千川计划结构、人群包、出价预算、ROI 止损线和直播/短视频承接",
    dependsOn: ["data"],
  },
  {
    nodeId: "creative",
    name: "创意产出",
    role: "creative",
    summary: "产出短视频、直播或活动内容创意",
    icon: PenLine,
    tone: "bg-violet-50 text-violet-700 dark:bg-violet-950/40 dark:text-violet-200",
    objective: "基于数据诊断产出抖音短视频、直播或活动内容创意",
    dependsOn: ["data"],
  },
  {
    nodeId: "ops",
    name: "运营方案",
    role: "ops",
    summary: "整合财务、千川投放和创意结果，形成可执行方案",
    icon: MonitorCog,
    tone: "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-200",
    objective: "整合财务、千川投放和创意结果，形成可执行运营方案",
    dependsOn: ["accounting", "douyin_ads", "creative"],
  },
  {
    nodeId: "delivery",
    name: "交付文档",
    role: "delivery",
    summary: "沉淀方案结构、执行清单和可下载产物",
    icon: FileText,
    tone: "bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-200",
    objective: "将运营方案沉淀为客户可复用文档",
    dependsOn: ["ops"],
  },
]

const FALLBACK_PHASES = [
  {
    nodeId: "data",
    title: "阶段一：数据诊断",
    detail: "识别指标、归因、平台场景和已知约束",
  },
  {
    nodeId: "parallel",
    title: "阶段二：并行协作",
    detail: "财务、千川投放、创意角色并行处理",
  },
  {
    nodeId: "ops",
    title: "阶段三：交叉校验",
    detail: "合并上游角色结果，交叉校验预算、素材和投放节奏",
  },
  {
    nodeId: "delivery",
    title: "阶段四：交付汇总",
    detail: "合成最终策略、执行清单和交付文件",
  },
]

const DELIVERABLES = [
  "运营全流程执行方案.docx",
  "数据诊断与归因报告.md",
  "千川投放计划表.xlsx",
]

export function AgentChatSwarmPanel({
  isStreaming,
  llmConfig,
  messages,
  onCollapse,
  runtimeEvents,
  selectedModelName,
}: AgentChatSwarmPanelProps) {
  const userMessages = messages.filter((message) => message.role === "user")
  const hasTask = userMessages.length > 0
  const lastTask = userMessages.at(-1)?.content
  const hasRuntime = runtimeEvents.length > 0
  const plannedGraph = getPlannedGraph(runtimeEvents)
  const graphNodes = plannedGraph?.nodes ?? []
  const runtimeNodeStates = getRuntimeNodeStates(runtimeEvents)
  const agents = getAgents(graphNodes)
  const doneAgents = agents.filter(
    (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "done"
  )
  const activeAgent =
    agents.find((agent) => runtimeNodeStates.get(agent.nodeId)?.state === "working") ??
    agents.find((agent) => runtimeNodeStates.get(agent.nodeId)?.state === "reworking") ??
    agents.find((agent) => runtimeNodeStates.get(agent.nodeId)?.state === "failed") ??
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
  const ActiveAgentIcon = hasRuntime ? activeAgent.icon : MonitorCog
  const activeNode = runtimeNodeStates.get(activeAgent.nodeId)
  const completedCount = getCompletedCount(agents, runtimeNodeStates)
  const progress = `${completedCount}/${agents.length}`
  const stage = getStageLabel({
    agents,
    hasTask,
    isStreaming,
    hasRuntime,
    runtimeNodeStates,
  })
  const phases = getPhases(graphNodes)
  const timeline = getTimeline(runtimeEvents, agents, hasTask, isStreaming)
  const selectedLlm = getSelectedLlm(llmConfig, selectedModelName)
  const computerCards = getComputerCards({
    activeAgent,
    activeNode,
    agents,
    lastTask,
    runtimeEvents,
    selectedLlmLabel: selectedLlm?.model,
  })
  return (
    <aside className="min-h-[31rem] min-w-0 overflow-hidden rounded-lg bg-[#efefec] p-3 dark:bg-muted/30 2xl:min-h-0">
      <div className="flex h-full min-h-0 flex-col overflow-hidden rounded-lg bg-[#efefec] dark:bg-card">
        <header className="shrink-0 border-b border-[#d8d8d3] bg-transparent px-3 py-4 dark:border-border">
          <div className="flex items-start justify-between gap-4">
            <div className="min-w-0">
              <div className="flex min-w-0 items-center gap-3">
                <div className="flex size-14 shrink-0 items-center justify-center rounded-md border border-foreground/20 bg-background text-foreground shadow-sm">
                  <MonitorCog className="size-7" />
                </div>
                <div className="min-w-0">
                  <h2 className="truncate font-mono text-2xl font-semibold tracking-wide">
                    智能体执行台
                  </h2>
                  <div className="mt-1 flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
                    <span
                      className={`size-3 rounded-full border-2 border-white shadow-sm ${
                        isStreaming
                          ? "animate-pulse bg-emerald-500"
                          : runtimeNodeStates.size > 0
                            ? hasFailedNode(runtimeNodeStates)
                              ? "bg-destructive"
                              : "bg-emerald-500"
                            : "bg-muted-foreground/40"
                      }`}
                    />
                    <span className="text-base text-foreground">当前进度 {progress}</span>
                    <span className="hidden sm:inline">|</span>
                    <span className="text-base">{stage}</span>
                  </div>
                </div>
              </div>
            </div>
            <div className="flex shrink-0 items-center gap-2">
              <div className="hidden min-w-0 items-center gap-2 rounded-full border border-border bg-background px-3 py-2 text-xs font-medium text-muted-foreground shadow-sm sm:flex">
                <Sparkles className="size-4 shrink-0 text-blue-500" />
                <span className="max-w-40 truncate min-[1800px]:max-w-52">
                  {selectedLlm
                    ? `${selectedLlm.model} · ${selectedLlm.name}`
                    : "模型通道加载中"}
                </span>
              </div>
              <button
                type="button"
                onClick={onCollapse}
                className="flex size-10 shrink-0 items-center justify-center rounded-full border border-border bg-background text-muted-foreground shadow-sm transition-colors hover:bg-muted hover:text-foreground"
                title="收起过程面板"
              >
                <PanelRightClose className="size-4" />
              </button>
            </div>
          </div>
        </header>

        <div className="min-h-0 flex-1 overflow-y-auto px-3 py-3">
          <section className="overflow-hidden rounded-lg border border-border bg-background shadow-sm">
            <div className="flex items-center justify-between gap-3 border-b border-border bg-card px-4 py-3">
              <div className="flex min-w-0 items-center gap-3">
                <div className="flex size-10 shrink-0 items-center justify-center rounded-md border border-border bg-card">
                  <ActiveAgentIcon className="size-5" />
                </div>
                <div className="min-w-0">
                  <p className="truncate font-mono text-sm font-semibold tracking-wide">
                    {hasRuntime
                      ? `角色 ${formatAgentNumber(agents.indexOf(activeAgent))} | ${activeAgent.name}`
                      : isStreaming
                        ? "任务规划 | 角色规则"
                        : "调度待命"}
                  </p>
                  <p className="truncate text-xs text-muted-foreground">
                    {hasRuntime
                      ? activeAgent.name
                      : isStreaming
                        ? "正在拆解任务并准备分配角色"
                        : "等待新任务触发本轮调度"}
                  </p>
                </div>
              </div>
              <div className="flex shrink-0 items-center gap-3">
                <span className="hidden text-xs text-muted-foreground xl:inline">
                  活动轨迹
                </span>
                <StatePill state={activeNode?.state ?? activeState} />
              </div>
            </div>

            <div className="min-h-[18rem] bg-[#fbfbfa]">
              {hasRuntime ? (
                <ComputerRuntimeDetail
                  activeAgent={activeAgent}
                  activeAgentIndex={activeAgentIndex}
                  activeNode={activeNode}
                  activeState={activeNode?.state ?? activeState}
                  cards={computerCards}
                  runtimeEvents={runtimeEvents}
                  selectedLlmLabel={selectedLlm?.model}
                />
              ) : (
                <SwarmIdleDetail
                  hasTask={hasTask}
                  isStreaming={isStreaming}
                  lastTask={lastTask}
                  selectedLlmLabel={selectedLlm?.model}
                />
              )}
            </div>
          </section>

          <section className="mt-3 rounded-lg border border-border bg-background p-3 shadow-sm">
            <div className="mb-2 flex items-center justify-between gap-3">
              <div className="flex items-center gap-2 text-sm font-medium">
                <ClipboardCheck className="size-4 text-muted-foreground" />
                角色任务图
                <span className="text-xs font-normal text-muted-foreground">
                  {runtimeNodeStates.size > 0
                    ? `${agents.length} 个任务节点`
                    : "等待本轮任务"}
                </span>
              </div>
              <span className="text-xs text-muted-foreground">
                {hasRuntime ? "实时调度" : "历史会话不计入进度"}
              </span>
            </div>

            <div className="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
              {agents.map((agent, index) => {
                const snapshot = runtimeNodeStates.get(agent.nodeId)
                const state = getAgentState({
                  agent,
                  fallbackIndex: index,
                  hasTask,
                  isStreaming,
                  runtimeNodeStates,
                })
                return (
                  <SwarmAgentQueueRow
                    key={agent.nodeId}
                    agent={agent}
                    index={index}
                    isActive={
                      Boolean(hasRuntime || isStreaming) &&
                      agent.nodeId === activeAgent.nodeId
                    }
                    state={state}
                    attempt={snapshot?.attempt}
                  />
                )
              })}
            </div>
          </section>

          <section className="mt-3 grid gap-2 lg:grid-cols-3 2xl:grid-cols-1 min-[1800px]:grid-cols-[1fr_1fr_1.2fr]">
            <CompactInfoBlock
              icon={Activity}
              title="活动轨迹"
              items={timeline.slice(0, 2).map((item) => item.label)}
            />
            <CompactInfoBlock
              icon={SearchCheck}
              title="执行清单"
              items={phases.slice(0, 2).map((phase) => phase.title)}
            />
            <CompactInfoBlock
              icon={FileText}
              title="产物预览"
              items={DELIVERABLES.slice(0, 2)}
            />
          </section>
        </div>

        <AgentDock
          activeNodeId={activeAgent.nodeId}
          agents={agents}
          hasTask={hasTask}
          isStreaming={isStreaming}
          runtimeNodeStates={runtimeNodeStates}
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
  selectedLlmLabel,
}: {
  activeAgent: SwarmAgent
  activeAgentIndex: number
  activeNode?: RuntimeNodeSnapshot
  activeState: SwarmState
  cards: ComputerCard[]
  runtimeEvents: AgentRuntimeEvent[]
  selectedLlmLabel?: string
}) {
  const screen = getComputerScreenContext({
    activeAgent,
    activeNode,
    runtimeEvents,
    selectedLlmLabel,
  })
  const ActiveIcon = activeAgent.icon
  const activityCards = cards.slice(-4)

  return (
    <div className="grid gap-3 p-3 xl:grid-cols-[13rem_minmax(0,1fr)]">
      <div className="min-w-0 space-y-2">
        <div className="flex items-center justify-between gap-2 px-1 text-xs text-muted-foreground">
          <span>活动轨迹</span>
          <span>{activityCards.length} 条</span>
        </div>
        {activityCards.map((card) => (
          <ComputerRuntimeCard key={card.id} card={card} />
        ))}
      </div>

      <section className="min-w-0 overflow-hidden rounded-lg border border-border bg-[#f7f7f4] shadow-sm dark:bg-background">
        <div className="flex items-center justify-between gap-3 border-b border-border bg-card px-3 py-2">
          <div className="flex min-w-0 items-center gap-2">
            <div className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-background">
              <ActiveIcon className="size-4" />
            </div>
            <div className="min-w-0">
              <p className="truncate font-mono text-sm font-semibold">
                角色 {formatAgentNumber(activeAgentIndex)} | {activeAgent.name}
              </p>
              <p className="truncate text-xs text-muted-foreground">
                {screen.mode} · {screen.filename}
              </p>
            </div>
          </div>
          <StatePill state={activeState} />
        </div>

        <div className="border-b border-border bg-background px-3 py-2.5">
          <div className="flex items-center justify-between gap-3">
            <span className="text-xs font-medium text-muted-foreground">任务输入</span>
            {selectedLlmLabel ? (
              <span className="min-w-0 truncate rounded-full bg-secondary px-2 py-1 text-[11px] text-secondary-foreground">
                {selectedLlmLabel}
              </span>
            ) : null}
          </div>
          <p className="mt-1 line-clamp-2 text-sm/6 text-foreground">
            {screen.request}
          </p>
        </div>

        <div className="p-3">
          <div className="overflow-hidden rounded-md border border-border bg-background">
            <div className="flex items-center justify-between gap-3 border-b border-border bg-card px-3 py-2">
              <div className="flex min-w-0 items-center gap-2">
                <FileText className="size-4 shrink-0 text-muted-foreground" />
                <span className="truncate text-sm font-medium">{screen.filename}</span>
              </div>
              <span className="shrink-0 text-[11px] text-muted-foreground">
                预览
              </span>
            </div>
            <div className="max-h-[18rem] overflow-hidden py-2 font-mono text-[11px]/5">
              {screen.lines.slice(0, 14).map((line, index) => (
                <div
                  key={`${index}-${line}`}
                  className="grid grid-cols-[2.5rem_minmax(0,1fr)] gap-3 px-3 py-0.5"
                >
                  <span className="select-none text-right tabular-nums text-muted-foreground/70">
                    {String(index + 1).padStart(2, "0")}
                  </span>
                  <span className="min-w-0 break-words text-foreground/85">
                    {line}
                  </span>
                </div>
              ))}
            </div>
          </div>

          <div className="mt-3 grid gap-2 sm:grid-cols-[minmax(0,1fr)_9rem]">
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
                  {activeAgent.name} · {getDockStateLabel(activeState)}
                </span>
              </div>
              <p className="mt-1 line-clamp-1 text-xs text-muted-foreground">
                {activeNode?.report
                  ? getFallbackResponse(activeAgent, activeState)
                  : activeAgent.objective}
              </p>
            </div>
            <div className="rounded-md border border-border bg-background px-3 py-2">
              <p className="text-xs text-muted-foreground">执行轮次</p>
              <p className="mt-1 font-mono text-sm font-semibold">
                {activeNode?.attempt ?? 1}
              </p>
            </div>
          </div>
        </div>
      </section>
    </div>
  )
}

function ComputerRuntimeCard({ card }: { card: ComputerCard }) {
  const Icon = card.icon

  return (
    <article className="rounded-lg border border-border bg-background p-2.5 shadow-sm">
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
}: {
  activeNodeId: string
  agents: SwarmAgent[]
  hasTask: boolean
  isStreaming: boolean
  runtimeNodeStates: Map<string, RuntimeNodeSnapshot>
}) {
  return (
    <div className="shrink-0 border-t border-[#d8d8d3] bg-[#f8f8f6] px-3 py-2 dark:border-border dark:bg-card">
      <div className="grid grid-cols-3 gap-2 lg:grid-cols-6 2xl:grid-cols-3 min-[1800px]:grid-cols-6">
        {agents.map((agent, index) => {
          const state = getAgentState({
            agent,
            fallbackIndex: index,
            hasTask,
            isStreaming,
            runtimeNodeStates,
          })
          const Icon = agent.icon
          const isActive = agent.nodeId === activeNodeId && (hasTask || isStreaming)

          return (
            <article
              key={agent.nodeId}
              className={`flex min-w-0 flex-col gap-2 rounded-lg border px-3 py-2.5 transition-colors ${
                isActive
                  ? "border-foreground bg-background shadow-sm"
                  : "border-transparent bg-[#eeeeeb] text-muted-foreground"
              }`}
            >
              <div className="flex items-start justify-between gap-3">
                <div
                  className={`flex size-9 shrink-0 items-center justify-center rounded-full border border-border bg-background ${
                    isActive ? "text-foreground" : "text-muted-foreground"
                  }`}
                >
                  <Icon className="size-4" />
                </div>
                <span className="font-mono text-sm font-semibold">
                  {formatAgentNumber(index)}
                </span>
              </div>
              <div className="min-w-0">
                <p className="truncate text-sm font-medium">{agent.name}</p>
                <div className="mt-2 flex items-center gap-2 text-xs">
                  <DockStateIcon state={state} />
                  <span className="truncate">{getDockStateLabel(state)}</span>
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
  if (state === "done") return <CheckCircle2 className="size-4 text-emerald-600" />
  if (state === "working" || state === "reworking") {
    return <Loader2 className="size-4 animate-spin text-emerald-600" />
  }
  if (state === "failed") return <span className="size-2.5 rounded-full bg-destructive" />
  return <span className="size-2.5 rounded-full bg-muted-foreground/35" />
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
    <div className="rounded-lg border border-border bg-background p-3">
      <div className="mb-2 flex items-center gap-2 text-sm font-medium">
        <Icon className="size-4 text-muted-foreground" />
        {title}
      </div>
      <div className="space-y-1.5">
        {items.map((item) => (
          <div key={item} className="flex items-center gap-2 text-xs text-muted-foreground">
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
  selectedLlmLabel,
}: {
  hasTask: boolean
  isStreaming: boolean
  lastTask?: string
  selectedLlmLabel?: string
}) {
  const title = isStreaming
    ? "正在规划任务图"
    : hasTask
      ? "历史会话，等待新任务"
      : "等待业务目标"
  const description = isStreaming
    ? "调度器已接收输入，正在匹配角色、依赖和验收规则。"
    : hasTask
      ? "右侧执行台只显示本轮实时调度，历史消息不会计入当前进度。"
      : "输入运营目标后，将按数据、预算、投放、创意和运营方案拆成多角色协作。"

  return (
    <div className="grid gap-3 p-3 xl:grid-cols-[13rem_minmax(0,1fr)]">
      <div className="space-y-2">
        <ComputerPreviewCard
          icon={FileText}
          title="读取目标"
          detail="经营目标与角色规则"
          lines={["数据诊断 / 预算评估 / 投放优化", "先诊断，再并行，再汇总"]}
        />
        <ComputerPreviewCard
          icon={TerminalSquare}
          title="调度角色"
          detail={selectedLlmLabel ? `模型通道 ${selectedLlmLabel}` : "等待模型通道"}
          lines={["模型通道已连接", "执行器待命"]}
        />
        <ComputerPreviewCard
          icon={Search}
          title="补充资料"
          detail="补充行业与平台场景资料"
          lines={["咖啡转化", "千川投放复盘"]}
        />
      </div>

      <div className="min-w-0 overflow-hidden rounded-lg border border-border bg-[#f7f7f4] shadow-sm dark:bg-background">
        <div className="flex items-center justify-between gap-3 border-b border-border bg-card px-3 py-2">
          <div className="flex min-w-0 items-center gap-2">
            <MonitorCog className="size-4 shrink-0 text-muted-foreground" />
            <div className="min-w-0">
              <p className="truncate font-mono text-sm font-semibold">
                规划节点 01 | 任务图
              </p>
              <p className="truncate text-xs text-muted-foreground">
                运营任务拆解预览
              </p>
            </div>
          </div>
          <StatePill state={isStreaming ? "working" : "idle"} />
        </div>
        <div className="border-b border-border bg-background px-3 py-2.5">
          <span className="text-xs font-medium text-muted-foreground">任务输入</span>
          <p className="mt-1 line-clamp-2 text-sm/6 text-foreground">
            {lastTask ?? description}
          </p>
        </div>
        <div className="p-3">
          <div className="overflow-hidden rounded-md border border-border bg-background">
            <div className="flex items-center gap-2 border-b border-border bg-card px-3 py-2">
              <FileText className="size-4 text-muted-foreground" />
              <span className="truncate text-sm font-medium">
                运营任务拆解预览
              </span>
            </div>
            <div className="py-2 font-mono text-[11px]/5">
              {[
                `# ${title}`,
                description,
                "数据诊断先识别指标、归因和经营约束",
                "预算评估、千川投放、创意产出并行处理",
                "运营方案汇总预算、素材、投放节奏",
                "交付文档沉淀方案、执行清单和复盘口径",
              ].map((line, index) => (
                <div
                  key={`${index}-${line}`}
                  className="grid grid-cols-[2.5rem_minmax(0,1fr)] gap-3 px-3 py-0.5"
                >
                  <span className="select-none text-right tabular-nums text-muted-foreground/70">
                    {String(index + 1).padStart(2, "0")}
                  </span>
                  <span className="min-w-0 break-words text-foreground/85">
                    {line}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function ComputerPreviewCard({
  detail,
  icon: Icon,
  lines,
  title,
}: {
  detail: string
  icon: ComponentType<SVGProps<SVGSVGElement>>
  lines: string[]
  title: string
}) {
  return (
    <div className="rounded-lg border border-border bg-card p-3">
      <div className="flex items-center justify-between gap-3">
        <div className="flex min-w-0 items-center gap-2">
          <Icon className="size-4 shrink-0 text-muted-foreground" />
          <div className="min-w-0">
            <p className="text-sm font-medium">{title}</p>
            <p className="truncate text-xs text-muted-foreground">{detail}</p>
          </div>
        </div>
        <ChevronRight className="size-4 shrink-0 text-muted-foreground" />
      </div>
      <div className="mt-3 space-y-1">
        {lines.map((line) => (
          <div
            key={line}
            className="truncate rounded-md bg-muted/70 px-2 py-1 text-[11px] text-muted-foreground"
          >
            {line}
          </div>
        ))}
      </div>
    </div>
  )
}

function SwarmAgentQueueRow({
  agent,
  attempt,
  index,
  isActive,
  state,
}: {
  agent: SwarmAgent
  attempt?: number
  index: number
  isActive: boolean
  state: SwarmState
}) {
  const Icon = agent.icon

  return (
    <article
      className={`min-w-0 rounded-lg border p-2.5 transition-colors ${
        isActive
          ? "border-foreground bg-background shadow-sm"
          : "border-border bg-[#f5f5f3] hover:border-border/80"
      }`}
    >
      <div className="flex items-start justify-between gap-2">
        <div
          className={`flex size-8 shrink-0 items-center justify-center rounded-full ${agent.tone}`}
        >
          <Icon className="size-4" />
        </div>
        <span className="font-mono text-sm font-semibold">
          {formatAgentNumber(index)}
        </span>
      </div>
      <div className="mt-2 min-w-0">
        <div className="flex items-start justify-between gap-2">
          <p className="truncate text-sm font-medium">{agent.name}</p>
          <StatePill state={state} />
        </div>
        <p className="mt-1 line-clamp-1 text-xs text-muted-foreground">
          {agent.summary}
        </p>
      </div>
      <div className="mt-2 flex items-center gap-1">
        {Array.from({ length: 10 }).map((_, barIndex) => (
          <span
            key={barIndex}
            className={`h-1.5 flex-1 rounded-[1px] ${
              shouldLightPixel(state, barIndex)
                ? state === "failed"
                  ? "bg-destructive"
                  : state === "reworking"
                    ? "bg-amber-500"
                    : "bg-emerald-500"
                : "bg-background"
            }`}
          />
        ))}
      </div>
      {attempt ? (
        <p className="mt-2 text-[11px] text-muted-foreground">
          第 {attempt} 次执行
        </p>
      ) : null}
    </article>
  )
}

function StatePill({ state }: { state: SwarmState }) {
  const label =
    state === "done"
      ? "已完成"
      : state === "working"
        ? "正在处理"
        : state === "queued"
          ? "排队"
          : state === "failed"
            ? "失败"
            : state === "reworking"
              ? "重试"
              : state === "skipped"
                ? "跳过"
                : "待命"

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

function getAgents(graphNodes: AgentTaskNode[]): SwarmAgent[] {
  if (!graphNodes.length) return AGENTS

  const dynamicAgents = graphNodes.map((node) => {
    const nodeId = readId(node.id)
    const preset = AGENTS.find((agent) => agent.nodeId === nodeId)
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
    AGENTS.find((agent) => agent.nodeId === "delivery") ?? AGENTS.at(-1)!,
  ]
}

function getPhases(graphNodes: AgentTaskNode[]) {
  if (!graphNodes.length) return FALLBACK_PHASES

  return [
    ...graphNodes.map((node, index) => ({
      nodeId: readId(node.id),
      title: `阶段 ${index + 1}：${node.title}`,
      detail: node.objective,
    })),
    {
      nodeId: "delivery",
      title: `阶段 ${graphNodes.length + 1}：交付汇总`,
      detail: "将各角色结果汇总为可交付方案、清单和文档。",
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
}) {
  if (!hasTask) return "等待任务输入"
  if (!hasRuntime) {
    return isStreaming ? "正在规划任务图" : "历史会话，等待新任务"
  }
  const failedAgent = agents.find(
    (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "failed"
  )
  if (failedAgent) return `执行受阻: ${failedAgent.name}`
  const reworkingAgent = agents.find(
    (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "reworking"
  )
  if (reworkingAgent) return `正在重试: ${reworkingAgent.name}`
  const workingAgent = agents.find(
    (agent) => runtimeNodeStates.get(agent.nodeId)?.state === "working"
  )
  if (workingAgent) return `正在执行: ${workingAgent.name}`
  if (isStreaming) return "并行协作中"
  return "方案已汇总"
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
  isStreaming: boolean
): SwarmTimelineItem[] {
  if (!runtimeEvents.length) {
    if (isStreaming) {
      return [
        { label: "调度器已收到任务，正在规划任务图。", tone: "default" },
        { label: "角色、依赖和验收规则生成后会同步到执行台。", tone: "default" },
      ]
    }
    if (hasTask) {
      return [
        { label: "当前展示的是历史对话，本轮执行台尚未开始。", tone: "default" },
        { label: "发送新任务后会重新规划角色与依赖。", tone: "default" },
      ]
    }
    return [
      { label: "等待客户输入任务，随后会规划角色与依赖。", tone: "default" },
      { label: "数据诊断先行，财务/千川/创意角色并行。", tone: "default" },
      { label: "运营角色负责交叉校验并汇总最终交付。", tone: "default" },
    ]
  }

  return runtimeEvents
    .map((event): SwarmTimelineItem | null => {
      if (event.type === "commander_thinking") {
        return { label: "调度器正在分析任务并准备拆图。", tone: "default" }
      }
      if (event.type === "task_graph_planned") {
        return {
          label: `已规划 ${event.graph.nodes.length} 个角色任务。`,
          tone: "success",
        }
      }
      if (event.type === "task_node_started") {
        return {
          label: `${getAgentName(event.nodeId, agents)} 开始第 ${event.attempt} 次执行。`,
          tone: "default",
        }
      }
      if (event.type === "task_node_rework_requested") {
        return {
          label: `${getAgentName(event.nodeId, agents)} 进入重试：${event.instruction}`,
          tone: "warning",
        }
      }
      if (event.type === "task_node_passed") {
        return {
          label: `${getAgentName(event.nodeId, agents)} 已通过验收。`,
          tone: "success",
        }
      }
      if (event.type === "task_node_failed") {
        return {
          label: `${getAgentName(event.nodeId, agents)} 失败：${event.reason}`,
          tone: "error",
        }
      }
      if (event.type === "task_graph_finished") {
        return { label: "任务图执行结束，进入汇总输出。", tone: "success" }
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
  selectedLlmLabel,
}: {
  activeAgent: SwarmAgent
  activeNode?: RuntimeNodeSnapshot
  agents: SwarmAgent[]
  lastTask?: string
  runtimeEvents: AgentRuntimeEvent[]
  selectedLlmLabel?: string
}): ComputerCard[] {
  const cards: ComputerCard[] = []
  const graphEvent = runtimeEvents.find(
    (event): event is Extract<AgentRuntimeEvent, { type: "task_graph_planned" }> =>
      event.type === "task_graph_planned"
  )

  if (graphEvent) {
    cards.push({
      id: "graph-planned",
      title: "读取目标",
      meta: "角色任务图",
      description: `已将任务拆成 ${graphEvent.graph.nodes.length} 个角色节点。`,
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
      title: "读取目标",
      meta: "用户任务",
      description: "读取用户目标并准备生成任务图。",
      icon: FileText,
      lines: [lastTask || "等待客户输入业务目标。"],
      state: "working",
    })
  }

  const latestNodeEvent = findLatestNodeEvent(runtimeEvents)
  cards.push({
    id: "runtime-terminal",
    title: "调度角色",
    meta: selectedLlmLabel ? `模型 ${selectedLlmLabel}` : "调度器待命",
    description: latestNodeEvent
      ? formatRuntimeEventDescription(latestNodeEvent, agents)
      : "调度器正在准备角色、依赖和验收规则。",
    icon: TerminalSquare,
    lines: [
      `当前角色：${activeAgent.name}`,
      `执行状态：${getDockStateLabel(activeNode?.state ?? "working")}`,
      latestNodeEvent ? `最近动作：${getRuntimeActionLabel(latestNodeEvent.type)}` : "最近动作：正在拆解任务",
    ],
    state: activeNode?.state ?? "working",
  })

  const reportLines = readReportLines(activeNode?.report)
  cards.push({
    id: "agent-output",
    title: activeNode?.report ? "角色输出" : "补充资料",
    meta: activeNode?.report ? activeAgent.name : "补充行业与平台场景资料",
    description: activeNode?.report
      ? getFallbackResponse(activeAgent, activeNode.state)
      : "角色开始输出后，这里会切换为真实的报告摘要、证据和风险。",
    icon: activeNode?.report ? Bot : Search,
    lines: reportLines.length
      ? reportLines
      : ["咖啡转化", "千川投放复盘", "运营预算与投放节奏"],
    state: activeNode?.state ?? "queued",
  })

  const reworkEvent = findLatestReworkEvent(runtimeEvents)
  if (reworkEvent) {
    cards.push({
      id: "rework",
      title: "重新检查",
      meta: getAgentName(reworkEvent.nodeId, agents),
      description: "验收未通过，系统已触发返工指令。",
      icon: SearchCheck,
      lines: [reworkEvent.instruction],
      state: "reworking",
    })
  }

  const failedEvent = findLatestFailedEvent(runtimeEvents)
  if (failedEvent) {
    cards.push({
      id: "failed",
      title: "执行受阻",
      meta: getAgentName(failedEvent.nodeId, agents),
      description: "当前节点执行失败，需要检查模型服务或上游依赖。",
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
  selectedLlmLabel,
}: {
  activeAgent: SwarmAgent
  activeNode?: RuntimeNodeSnapshot
  runtimeEvents: AgentRuntimeEvent[]
  selectedLlmLabel?: string
}): ComputerScreenContext {
  const graphNode = getActiveGraphNode(runtimeEvents, activeAgent.nodeId)
  const reportLines = readReportDocumentLines(activeNode?.report)
  const request = graphNode?.objective ?? activeAgent.objective
  const acceptanceLines =
    graphNode?.acceptance.map((item, index) => `验收规则 ${index + 1}：${item}`) ??
    []
  const lines = compactPreviewLines([
    `# ${activeAgent.name}`,
    `角色职责：${activeAgent.summary}`,
    selectedLlmLabel ? `模型通道：${selectedLlmLabel}` : "模型通道：等待选择",
    `任务目标：${request}`,
    ...acceptanceLines,
    ...reportLines,
    ...((activeNode?.evaluationReasons ?? []).map((reason) => `复核意见：${reason}`)),
  ])

  return {
    filename: getComputerFilename(activeAgent),
    lines,
    mode: activeNode?.report ? "报告" : "任务",
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

function getComputerFilename(agent: SwarmAgent) {
  if (agent.nodeId === "data") return "research.dimension.report.md"
  if (agent.nodeId === "accounting") return "budget.risk.assessment.xlsx"
  if (agent.nodeId === "douyin_ads") return "douyin.ads.launch.plan.xlsx"
  if (agent.nodeId === "creative") return "content.script.matrix.md"
  if (agent.nodeId === "ops") return "operation.execution.plan.md"
  if (agent.nodeId === "delivery") return "运营全流程执行方案.docx"
  return `${agent.role}.agent.output.md`
}

function readReportDocumentLines(report?: AgentWorkerReport) {
  if (!report) return []
  return compactPreviewLines([report.content, ...readReportLines(report)])
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
  agents: SwarmAgent[]
) {
  if (event.type === "task_node_started") {
    return `${getAgentName(event.nodeId, agents)} 开始执行第 ${event.attempt} 次尝试。`
  }
  if (event.type === "task_node_reported") {
    return `${getAgentName(event.nodeId, agents)} 已提交报告，等待验收或汇总。`
  }
  if (event.type === "task_node_passed") {
    return `${getAgentName(event.nodeId, agents)} 已通过验收。`
  }
  if (event.type === "task_node_failed") {
    return `${getAgentName(event.nodeId, agents)} 执行失败。`
  }
  return `${getAgentName(event.nodeId, agents)} 正在按反馈重新组织输出。`
}

function getRuntimeActionLabel(eventType: AgentRuntimeEvent["type"]) {
  if (eventType === "commander_thinking") return "正在拆解任务"
  if (eventType === "task_graph_planned") return "任务图已生成"
  if (eventType === "task_node_ready") return "角色已排队"
  if (eventType === "task_node_started") return "角色开始执行"
  if (eventType === "task_node_reported") return "角色提交结果"
  if (eventType === "task_node_evaluated") return "结果已复核"
  if (eventType === "task_node_rework_requested") return "触发返工"
  if (eventType === "task_node_passed") return "通过验收"
  if (eventType === "task_node_failed") return "执行受阻"
  if (eventType === "task_graph_finished") return "任务图结束"
  return "状态已同步"
}

function readReportLines(report?: AgentWorkerReport) {
  if (!report) return []
  const roleOutput = report.role_output
  return [
    roleOutput?.summary,
    ...(roleOutput?.findings ?? []),
    ...(roleOutput?.recommendations ?? []),
    ...(roleOutput?.evidence ?? []),
    ...(report.risks ?? []).map((risk) => `风险: ${risk}`),
    ...(report.open_questions ?? []).map((question) => `待确认: ${question}`),
  ].filter((line): line is string => Boolean(line))
}

function getAgentName(nodeId: string, agents: SwarmAgent[]) {
  return agents.find((agent) => agent.nodeId === nodeId)?.name ?? nodeId
}

function getFallbackResponse(agent: SwarmAgent, state: SwarmState) {
  if (state === "failed") {
    return "该角色执行失败，通常是模型服务、工具或上游依赖不可用导致。"
  }
  if (state === "reworking") return "验收未通过，正在按反馈重新组织角色输出。"
  if (state === "working") return `${agent.name} 正在处理请求，结果会在这里更新。`
  if (state === "done") return `${agent.name} 已完成，等待最终汇总。`
  return "任务开始后，这里会展示当前角色的请求、推理结果和产物摘要。"
}

function getDockStateLabel(state: SwarmState) {
  if (state === "done") return "已完成"
  if (state === "working") return "正在推理"
  if (state === "reworking") return "正在重试"
  if (state === "failed") return "执行失败"
  if (state === "queued") return "等待执行"
  if (state === "skipped") return "已跳过"
  return "等待任务"
}

function getSelectedLlm(
  llmConfig: AgentLlmConfigResponse | null,
  selectedModelName: string | null
) {
  if (!llmConfig) return null
  return (
    llmConfig.items.find((item) => item.name === selectedModelName) ??
    llmConfig.items.find((item) => item.selected) ??
    llmConfig.items.find((item) => item.name === llmConfig.selectedLlm) ??
    llmConfig.items[0] ??
    null
  )
}

function hasFailedNode(runtimeNodeStates: Map<string, RuntimeNodeSnapshot>) {
  return Array.from(runtimeNodeStates.values()).some(
    (snapshot) => snapshot.state === "failed"
  )
}

function shouldLightPixel(state: SwarmState, index: number) {
  if (state === "done") return index < 10
  if (state === "working" || state === "reworking") return index < 6
  if (state === "failed") return index < 4
  if (state === "queued" || state === "skipped") return index < 2
  return false
}

function formatAgentNumber(index: number) {
  return String(Math.max(index + 1, 1)).padStart(2, "0")
}

function readId(value: string | [string] | { 0: string }) {
  if (typeof value === "string") return value
  if (Array.isArray(value)) return value[0]
  return value[0]
}
