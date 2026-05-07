import { CheckCircle2, Loader2 } from "lucide-react"
import { useTranslation } from "react-i18next"
import {
  formatAgentNumber,
  getAgentState,
  getDockStateLabel,
  type RuntimeNodeSnapshot,
  type SwarmAgent,
  type SwarmState,
} from "../view-model/swarm-panel-view-model"

export function AgentDock({
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
  const { t } = useTranslation()

  return (
    <div className="shrink-0 border-t border-[#d8d8d3] bg-[#f8f8f6] p-3 dark:border-border dark:bg-card">
      <div className="flex min-w-0 gap-3 overflow-x-auto overscroll-x-contain pb-1 [scrollbar-width:thin]">
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
              className={`flex min-w-48 shrink-0 items-center gap-3 rounded-xl border px-4 py-3 transition-colors min-[1500px]:min-w-52 ${
                isActive
                  ? "border-foreground bg-background shadow-sm"
                  : "border-transparent bg-[#eeeeeb] text-muted-foreground"
              }`}
            >
              <div
                className={`flex size-10 shrink-0 items-center justify-center rounded-full border border-border bg-background ${
                  isActive ? "text-foreground" : "text-muted-foreground"
                }`}
              >
                <Icon className="size-5" />
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex min-w-0 items-center gap-1.5">
                  <span className="shrink-0 font-mono text-sm font-semibold text-muted-foreground">
                    {formatAgentNumber(index)}
                  </span>
                  <p className="truncate text-sm font-semibold">{agent.name}</p>
                </div>
                <div className="mt-1 flex items-center gap-1.5 text-xs text-muted-foreground">
                  <DockStateIcon state={state} />
                  <span className="truncate">
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
  if (state === "done") {
    return <CheckCircle2 className="size-3 text-emerald-600" />
  }
  if (state === "working" || state === "reworking") {
    return <Loader2 className="size-3 animate-spin text-emerald-600" />
  }
  if (state === "failed") {
    return <span className="size-2 rounded-full bg-destructive" />
  }
  return <span className="size-2 rounded-full bg-muted-foreground/35" />
}
