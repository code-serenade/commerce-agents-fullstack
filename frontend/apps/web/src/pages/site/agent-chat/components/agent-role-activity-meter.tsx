import type { InlineAgentState } from "../view-model/thread-inline-view-model"

export function AgentRoleActivityMeter({ state }: { state: InlineAgentState }) {
  const activeCount = getAgentRoleActivityMeterCount(state)

  return (
    <div className="grid grid-cols-10 gap-0.5" aria-hidden="true">
      {Array.from({ length: 10 }, (_, index) => (
        <span
          key={index}
          className={`h-1 rounded-full ${
            index < activeCount ? "bg-primary/70" : "bg-muted-foreground/15"
          }`}
        />
      ))}
    </div>
  )
}

function getAgentRoleActivityMeterCount(state: InlineAgentState) {
  if (state === "done") return 10
  if (state === "boundary") return 7
  if (state === "working" || state === "reworking") return 6
  if (state === "failed") return 4
  if (state === "queued") return 3
  return 1
}
