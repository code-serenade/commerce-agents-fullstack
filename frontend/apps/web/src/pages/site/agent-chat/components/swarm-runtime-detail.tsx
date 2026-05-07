import { CheckCircle2, FileText, Loader2, MonitorCog } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "@/api"
import {
  formatAgentNumber,
  getComputerCards,
  getComputerScreenContext,
  getDockStateLabel,
  getFallbackResponse,
  type ComputerCard,
  type RuntimeNodeSnapshot,
  type SwarmAgent,
  type SwarmState,
} from "../view-model/swarm-panel-view-model"
import { StatePill } from "./swarm-state-pill"

export function ComputerRuntimeDetail({
  activeAgent,
  activeAgentIndex,
  activeNode,
  activeState,
  agents,
  lastTask,
  runtimeEvents,
}: {
  activeAgent: SwarmAgent
  activeAgentIndex: number
  activeNode?: RuntimeNodeSnapshot
  activeState: SwarmState
  agents: SwarmAgent[]
  lastTask?: string
  runtimeEvents: AgentRuntimeEvent[]
}) {
  const { t } = useTranslation()
  const cards = getComputerCards({
    activeAgent,
    activeNode,
    agents,
    lastTask,
    runtimeEvents,
  }, t)
  const screen = getComputerScreenContext({ activeAgent, activeNode, runtimeEvents }, t)
  const ActiveIcon = activeAgent.icon

  return (
    <section className="grid min-h-124 overflow-hidden rounded-[1.25rem] border border-[#dcdcd7] bg-white shadow-sm min-[1500px]:grid-cols-[12.75rem_minmax(0,1fr)] dark:border-border dark:bg-background">
      <div className="min-w-0 border-b border-border bg-[#f7f7f4] p-3 min-[1500px]:border-r min-[1500px]:border-b-0 dark:bg-card">
        <div className="mb-2 flex items-center justify-between gap-2 text-xs text-muted-foreground">
          <span>{t("agentChat.workbench.record")}</span>
          <span>
            {cards.slice(-4).length} {t("agentChat.workbench.recordCountSuffix")}
          </span>
        </div>
        <div className="grid gap-2 min-[1500px]:grid-cols-1 sm:grid-cols-2">
          {cards.slice(-4).map((card) => (
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
                {screen.mode} | {screen.filename}
              </p>
            </div>
          </div>
          <StatePill state={activeState} />
        </div>
        <ComputerScreen
          activeAgent={activeAgent}
          activeNode={activeNode}
          activeState={activeState}
          screen={screen}
        />
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

function ComputerScreen({
  activeAgent,
  activeNode,
  activeState,
  screen,
}: {
  activeAgent: SwarmAgent
  activeNode?: RuntimeNodeSnapshot
  activeState: SwarmState
  screen: ReturnType<typeof getComputerScreenContext>
}) {
  const { t } = useTranslation()

  return (
    <div className="grid min-h-112 grid-rows-[auto_minmax(0,1fr)_auto]">
      <div className="border-b border-border bg-white px-4 py-3 min-[1500px]:px-5 dark:bg-background">
        <span className="text-xs font-medium text-muted-foreground">
          {t("agentChat.workbench.businessInput")}
        </span>
        <p className="mt-1 line-clamp-2 text-sm/6 text-foreground">
          {screen.request}
        </p>
      </div>
      <PreviewDocument filename={screen.filename} lines={screen.lines} />
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
              {activeAgent.name} | {getDockStateLabel(activeState, t)}
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
  )
}

function PreviewDocument({ filename, lines }: { filename: string; lines: string[] }) {
  const { t } = useTranslation()

  return (
    <div className="min-h-0 bg-white p-3 min-[1500px]:p-4 dark:bg-background">
      <div className="flex h-full min-h-72 flex-col overflow-hidden rounded-lg border border-border bg-[#fbfbfa] dark:bg-card">
        <div className="flex items-center justify-between gap-3 border-b border-border bg-[#f7f7f4] px-4 py-2.5 dark:bg-card">
          <div className="flex min-w-0 items-center gap-2">
            <FileText className="size-4 shrink-0 text-muted-foreground" />
            <span className="truncate text-sm font-medium">{filename}</span>
          </div>
          <span className="shrink-0 rounded-full bg-secondary px-2 py-1 text-[11px] text-secondary-foreground">
            {t("agentChat.workbench.preview")}
          </span>
        </div>
        <div className="min-h-0 flex-1 overflow-hidden py-2 font-mono text-[11px]/5">
          {lines.slice(0, 14).map((line, index) => (
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
  )
}
