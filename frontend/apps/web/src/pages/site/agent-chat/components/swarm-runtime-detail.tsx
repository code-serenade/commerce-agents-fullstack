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
    <section className="grid min-h-0 overflow-hidden rounded-lg border border-border bg-background">
      <div className="min-w-0 bg-background">
        <div className="flex items-center justify-between gap-3 border-b border-border bg-background px-4 py-3 min-[1500px]:px-5">
          <div className="flex min-w-0 items-center gap-2">
            <div className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-card">
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
          cards={cards}
          screen={screen}
        />
      </div>
    </section>
  )
}

function ComputerScreen({
  activeAgent,
  activeNode,
  activeState,
  cards,
  screen,
}: {
  activeAgent: SwarmAgent
  activeNode?: RuntimeNodeSnapshot
  activeState: SwarmState
  cards: ComputerCard[]
  screen: ReturnType<typeof getComputerScreenContext>
}) {
  const { t } = useTranslation()

  return (
    <div className="grid min-h-0">
      <div className="border-b border-border bg-muted/30 px-3 py-2.5 min-[1500px]:px-4">
        <span className="text-xs font-semibold text-foreground">
          {t("agentChat.workbench.businessInput")}
        </span>
        <p className="mt-1 line-clamp-2 text-sm/6 text-foreground">
          {screen.request}
        </p>
      </div>
      <PreviewDocument filename={screen.filename} lines={screen.lines} />
      <div className="grid gap-2 border-t border-border bg-background p-2 min-[1500px]:grid-cols-[minmax(0,1fr)_8rem]">
        <div className="min-w-0 rounded-md border border-border bg-card px-3 py-2">
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            {activeState === "working" || activeState === "reworking" ? (
              <Loader2 className="size-4 animate-spin text-emerald-600" />
            ) : activeState === "done" ? (
              <CheckCircle2 className="size-4 text-emerald-600" />
            ) : activeState === "boundary" ? (
              <MonitorCog className="size-4 text-amber-600" />
            ) : (
              <MonitorCog className="size-4" />
            )}
            <span className="truncate">
              {activeAgent.name} | {getDockStateLabel(activeState, t)}
            </span>
          </div>
          <p className="mt-1 line-clamp-1 text-[11px] text-muted-foreground">
            {activeNode?.report
              ? getFallbackResponse(activeAgent, activeState, t)
              : activeAgent.objective}
          </p>
        </div>
        <div className="rounded-md border border-border bg-card px-3 py-2">
          <p className="text-xs text-muted-foreground">
            {t("agentChat.workbench.turns")}
          </p>
          <p className="mt-1 font-mono text-sm font-semibold">
            {activeNode?.attempt ?? 1}
          </p>
        </div>
        {cards.slice(-2).length ? (
          <div className="min-w-0 min-[1500px]:col-span-2">
            <div className="grid min-w-0 gap-1.5 min-[1500px]:grid-cols-2">
              {cards.slice(-2).map((card) => (
                <ComputerRuntimeChip key={card.id} card={card} />
              ))}
            </div>
          </div>
        ) : null}
      </div>
    </div>
  )
}

function ComputerRuntimeChip({ card }: { card: ComputerCard }) {
  const Icon = card.icon

  return (
    <article
      className="flex min-w-0 items-center gap-2 rounded-md border border-border bg-card px-2.5 py-1.5 text-xs"
      title={`${card.title} ${card.description}`}
    >
      <Icon className="size-3.5 shrink-0 text-muted-foreground" />
      <span className="min-w-0 flex-1 truncate font-medium">
        {card.title}
      </span>
      {card.meta ? (
        <span className="shrink-0 text-[11px] text-muted-foreground">
          {card.meta}
        </span>
      ) : null}
      <StatePill state={card.state} />
    </article>
  )
}

function PreviewDocument({ filename, lines }: { filename: string; lines: string[] }) {
  const { t } = useTranslation()

  return (
    <div className="min-h-0 bg-background p-2.5 min-[1500px]:p-3">
      <div className="flex h-full min-h-36 flex-col overflow-hidden rounded-md border border-border bg-card">
        <div className="flex items-center justify-between gap-3 border-b border-border bg-background px-4 py-2.5">
          <div className="flex min-w-0 items-center gap-2">
            <FileText className="size-4 shrink-0 text-muted-foreground" />
            <span className="truncate text-sm font-medium">{filename}</span>
          </div>
          <span className="shrink-0 rounded-full bg-secondary px-2 py-1 text-[11px] text-secondary-foreground">
            {t("agentChat.workbench.preview")}
          </span>
        </div>
        <div className="min-h-0 flex-1 overflow-x-hidden overflow-y-auto py-2 font-mono text-[11px]/5 [scrollbar-gutter:stable]">
          {lines.map((line, index) => (
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
