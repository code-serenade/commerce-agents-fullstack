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
    <div className="shrink-0 bg-(--agent-chat-elevated-strong)/70 px-3 py-2.5">
      <div className="grid min-w-0 grid-cols-2 gap-2 min-[1500px]:grid-cols-3">
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
              className={`relative flex min-w-0 items-center gap-2 rounded-lg px-2.5 py-2 transition-colors ${
                isActive
                  ? "bg-primary/10 text-foreground"
                  : "bg-transparent text-muted-foreground hover:bg-muted/60"
              }`}
            >
              <span
                className={`absolute inset-y-2  left-0 w-0.5 rounded-full ${
                  isActive ? "bg-primary" : "bg-transparent"
                }`}
              />
              <div
                className={`flex size-7 shrink-0 items-center justify-center rounded-md ${
                  isActive
                    ? "bg-primary/15 text-primary"
                    : "bg-muted/60 text-muted-foreground"
                }`}
              >
                <Icon className="size-3.5" />
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex min-w-0 items-center gap-1.5">
                  <span
                    className={`shrink-0 font-mono text-xs font-semibold ${
                      isActive ? "text-primary" : "text-muted-foreground"
                    }`}
                  >
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
