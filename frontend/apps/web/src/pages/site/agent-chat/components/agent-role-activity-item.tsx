import type { AgentRoleActivity } from "../view-model/agent-role-cluster-view-model"
import type { InlineAgentState } from "../view-model/thread-inline-view-model"
import { AgentRolePortrait } from "./agent-role-portrait"
import {
  panelClassNames,
  roleAccentClassNames,
  roleDotClassNames,
  type AgentRoleClusterVariant,
} from "./agent-role-cluster-styles"

export function AgentRoleActivityItem({
  animatePortraits,
  activity,
  index,
  itemDetail,
  variant,
}: {
  animatePortraits?: boolean
  activity: AgentRoleActivity
  index: number
  itemDetail: "action" | "summary"
  variant: AgentRoleClusterVariant
}) {
  const detail =
    itemDetail === "summary" ? activity.agent.summary : activity.action

  if (variant === "showcase") {
    return (
      <AgentRoleShowcaseItem
        animatePortraits={animatePortraits}
        activity={activity}
        detail={detail}
      />
    )
  }

  return (
    <AgentRoleTimelineItem
      activity={activity}
      animatePortraits={animatePortraits}
      detail={detail}
      index={index}
      isCompact={variant === "compact"}
    />
  )
}

function AgentRoleShowcaseItem({
  animatePortraits,
  activity,
  detail,
}: {
  animatePortraits?: boolean
  activity: AgentRoleActivity
  detail: string
}) {
  return (
    <div
      className={`group relative grid min-h-40 grid-rows-[auto_minmax(0,1fr)] overflow-hidden rounded-md border border-t-2 bg-background/90 p-3 text-center shadow-sm transition hover:-translate-y-0.5 hover:border-primary/40 hover:shadow-md ${panelClassNames[activity.state]} ${roleAccentClassNames[activity.role]}`}
      data-agent-role-card={activity.role}
    >
      <span
        className={`absolute inset-x-5 top-2 h-1 rounded-full ${roleDotClassNames[activity.role]}`}
        aria-hidden="true"
      />
      <div className="pt-1">
        <AgentRolePortrait
          activity={activity}
          animate={animatePortraits}
          size="md"
        />
      </div>
      <div className="mt-1 min-w-0">
        <div className="flex min-w-0 items-center justify-center gap-1.5">
          <span
            className={`size-1.5 shrink-0 rounded-full ${roleDotClassNames[activity.role]}`}
            aria-hidden="true"
          />
          <p className="truncate text-xs font-semibold text-foreground">
            {activity.agent.name}
          </p>
        </div>
        <p className="mt-1 line-clamp-2 min-h-8 text-[11px]/4 text-muted-foreground">
          {detail}
        </p>
        <p className="mt-1 line-clamp-2 text-[10px]/4 text-muted-foreground">
          {activity.contribution}
        </p>
        <div className="mt-2 grid gap-1 text-[10px] font-medium text-muted-foreground">
          <span className="truncate">{activity.agent.status}</span>
          <AgentRoleActivityMeter state={activity.state} />
        </div>
      </div>
    </div>
  )
}

function AgentRoleTimelineItem({
  animatePortraits,
  activity,
  detail,
  index,
  isCompact,
}: {
  animatePortraits?: boolean
  activity: AgentRoleActivity
  detail: string
  index: number
  isCompact: boolean
}) {
  return (
    <div
      className={`relative flex min-w-0 items-center gap-2.5 rounded-lg border px-3 py-2 pr-26 text-left ${panelClassNames[activity.state]} ${
        isCompact ? "min-h-14" : "min-h-16"
      }`}
      data-agent-role-card={activity.role}
    >
      <span className="absolute top-2 right-3 text-[11px] font-medium text-muted-foreground">
        {activity.agent.status}
      </span>
      <AgentRolePortrait
        activity={activity}
        animate={animatePortraits}
        size="xs"
      />
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 items-center gap-1.5">
          <span className="shrink-0 font-mono text-[11px] font-semibold text-muted-foreground">
            {String(index + 1).padStart(2, "0")}
          </span>
          <span
            className={`size-1.5 shrink-0 rounded-full ${roleDotClassNames[activity.role]}`}
            aria-hidden="true"
          />
          <p className="truncate text-xs font-semibold text-foreground">
            {activity.agent.name}
          </p>
        </div>
        <p className="mt-0.5 text-[11px]/4 wrap-break-word text-muted-foreground">
          {detail}
        </p>
        <p className="mt-0.5 text-[10px]/4 wrap-break-word text-muted-foreground">
          {activity.contribution}
        </p>
      </div>
      <div className="hidden w-20 shrink-0 sm:block">
        <AgentRoleActivityMeter state={activity.state} />
      </div>
    </div>
  )
}

function AgentRoleActivityMeter({ state }: { state: InlineAgentState }) {
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
