import type { AgentRoleActivity } from "../view-model/agent-role-cluster-view-model"
import { AgentRolePortrait } from "./agent-role-portrait"

export function InlineRoleParticipantStrip({
  activities,
  animatePortraits = false,
  label,
}: {
  activities: AgentRoleActivity[]
  animatePortraits?: boolean
  label: string
}) {
  return (
    <div
      className="mt-2 flex min-w-0 items-center gap-2"
      data-agent-role-participant-strip="true"
    >
      <span className="shrink-0 text-[11px] font-medium text-muted-foreground">
        {label}
      </span>
      <div className="flex min-w-0 flex-1 gap-1.5 overflow-x-auto pb-1 [scrollbar-gutter:stable]">
        {activities.map((activity) => (
          <div
            key={`${activity.agent.nodeId}-${activity.role}`}
            className="flex max-w-56 shrink-0 items-center gap-1.5 rounded-md border border-border bg-background/80 py-1 pl-1 pr-2 text-xs text-foreground shadow-xs"
            data-agent-role-participant={activity.role}
            title={`${activity.agent.name} - ${activity.contribution}`}
          >
            <AgentRolePortrait
              activity={activity}
              animate={animatePortraits}
              size="xs"
            />
            <span className="min-w-0 truncate">
              {activity.agent.name}
              <span className="text-muted-foreground">
                {" · "}
                {activity.contribution}
              </span>
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}
