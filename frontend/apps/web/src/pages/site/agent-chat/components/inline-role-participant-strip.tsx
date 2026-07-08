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
      className="mt-2 flex min-w-0 items-start gap-2"
      data-agent-role-participant-strip="true"
    >
      <span className="mt-1 shrink-0 text-[11px] font-medium text-muted-foreground">
        {label}
      </span>
      <div className="flex min-w-0 flex-1 flex-wrap gap-2">
        {activities.slice(0, 4).map((activity) => (
          <div
            key={`${activity.agent.nodeId}-${activity.role}`}
            className="w-28 shrink-0 overflow-hidden rounded-xl border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) text-foreground"
            data-agent-role-participant={activity.role}
            title={`${activity.agent.name} - ${activity.contribution}`}
          >
            <AgentRolePortrait
              activity={activity}
              animate={animatePortraits}
              size="card"
            />
            <div className="px-2 py-1.5">
              <div className="truncate text-xs font-semibold text-foreground">
                {activity.agent.name}
              </div>
              <p className="mt-0.5 line-clamp-2 min-h-8 text-[11px]/4 text-muted-foreground">
                {activity.contribution}
              </p>
            </div>
          </div>
        ))}
        {activities.length > 4 ? (
          <span className="inline-flex h-7 shrink-0 items-center rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-2 text-xs font-medium text-muted-foreground">
            +{activities.length - 4}
          </span>
        ) : null}
      </div>
    </div>
  )
}
