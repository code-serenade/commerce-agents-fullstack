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
    <div className="shrink-0 bg-background p-2">
      <div className="grid min-w-0 grid-cols-2 gap-1.5 min-[1500px]:grid-cols-3">
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
              className={`flex min-w-0 items-center gap-2 rounded-md border px-2 py-1.5 transition-colors ${
                isActive
                  ? "border-foreground bg-background shadow-sm"
                  : "border-border bg-background/70 text-muted-foreground"
              }`}
            >
              <div
                className={`flex size-7 shrink-0 items-center justify-center rounded-md border border-border bg-background ${
                  isActive ? "text-foreground" : "text-muted-foreground"
                }`}
              >
                <Icon className="size-3.5" />
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex min-w-0 items-center gap-1.5">
                  <span className="shrink-0 font-mono text-xs font-semibold text-muted-foreground">
                    {formatAgentNumber(index)}
                  </span>
                  <p className="truncate text-xs font-semibold">{agent.name}</p>
                </div>
                <div className="mt-0.5 flex items-center gap-1.5 text-[11px] text-muted-foreground">
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
  if (state === "boundary") {
    return <span className="size-2 rounded-full bg-amber-500" />
  }
  if (state === "working" || state === "reworking") {
    return <Loader2 className="size-3 animate-spin text-emerald-600" />
  }
  if (state === "failed") {
    return <span className="size-2 rounded-full bg-destructive" />
  }
  return <span className="size-2 rounded-full bg-muted-foreground/35" />
}
