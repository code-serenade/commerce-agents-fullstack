import type { AgentRoleActivity } from "../view-model/agent-role-cluster-view-model"
import { AgentRoleActivityMeter } from "./agent-role-activity-meter"
import { AgentRolePortrait } from "./agent-role-portrait"
import { AgentRoleSceneItem } from "./agent-role-scene-item"
import { getAgentRoleStatusTagClassName } from "./agent-role-status-tag"
import {
  panelClassNames,
  roleAccentClassNames,
  roleDotClassNames,
  roleScrimAccentColors,
  type AgentRoleClusterVariant,
} from "./agent-role-cluster-styles"
import type { CSSProperties } from "react"

export function AgentRoleActivityItem({
  animatePortraits,
  activity,
  index,
  itemDetail,
  showStatus = true,
  variant,
}: {
  animatePortraits?: boolean
  activity: AgentRoleActivity
  index: number
  itemDetail: "action" | "summary"
  showStatus?: boolean
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
        showStatus={showStatus}
      />
    )
  }

  if (variant === "compact") {
    return (
      <AgentRoleCompactCard
        activity={activity}
        animatePortraits={animatePortraits}
        detail={detail}
        index={index}
      />
    )
  }

  return (
    <AgentRoleTimelineItem
      activity={activity}
      animatePortraits={animatePortraits}
      detail={detail}
      index={index}
    />
  )
}
function AgentRoleCompactCard({
  animatePortraits,
  activity,
  detail,
  index,
}: {
  animatePortraits?: boolean
  activity: AgentRoleActivity
  detail: string
  index: number
}) {
  const cardStyle = {
    "--agent-role-card-scrim-accent": roleScrimAccentColors[activity.role],
  } as CSSProperties

  return (
    <div
      className={`group relative grid w-[120px] shrink-0 grid-rows-[auto_minmax(0,1fr)] overflow-hidden rounded-lg border bg-(--agent-chat-elevated-strong) bg-[radial-gradient(circle_at_100%_0%,rgb(var(--agent-role-card-scrim-accent)/0.34)_0%,rgb(var(--agent-role-card-scrim-accent)/0.18)_35%,transparent_68%),linear-gradient(145deg,rgb(var(--agent-role-card-scrim-accent)/0.16)_0%,var(--agent-chat-elevated-strong)_48%,var(--agent-chat-subtle-surface)_100%)] text-center shadow-sm transition hover:border-primary/40 ${panelClassNames[activity.state]}`}
      data-agent-role-card={activity.role}
      style={cardStyle}
      title={detail}
    >
      <span className="absolute top-0 left-0 z-20 font-mono text-2xl font-bold text-[rgb(var(--agent-role-card-scrim-accent)/0.92)] italic [-webkit-text-stroke:2px_white] [paint-order:stroke_fill] dark:[-webkit-text-stroke:2px_#191d20]">
        {String(index + 1).padStart(2, "0")}
      </span>
      <span
        className={`absolute top-0 right-0 z-20 rounded-bl-md px-2 py-0.5 text-[10px] font-medium backdrop-blur ${getAgentRoleStatusTagClassName(
          activity.state
        )}`}
      >
        {activity.agent.status}
      </span>
      <div className="relative aspect-square w-full overflow-hidden bg-transparent">
        <AgentRolePortrait
          activity={activity}
          animate={animatePortraits}
          size="card"
        />
      </div>
      <div className="flex min-h-0 flex-col items-center px-2.5 pt-2.5 pb-3">
        <div className="w-full min-w-0 text-left">
          <p className="truncate text-xs font-semibold text-foreground">
            {activity.agent.name}
          </p>
        </div>
        <p
          className="mt-1 line-clamp-2 min-h-8 w-full min-w-0 text-left text-[11px]/4 text-muted-foreground"
          title={detail}
        >
          {detail}
        </p>
        <div className="mt-2 w-full">
          <AgentRoleActivityMeter state={activity.state} />
        </div>
      </div>
    </div>
  )
}

function AgentRoleShowcaseItem({
  animatePortraits,
  activity,
  detail,
  showStatus,
}: {
  animatePortraits?: boolean
  activity: AgentRoleActivity
  detail: string
  showStatus: boolean
}) {
  if (!showStatus) {
    return <AgentRoleSceneItem activity={activity} detail={detail} />
  }

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
}: {
  animatePortraits?: boolean
  activity: AgentRoleActivity
  detail: string
  index: number
}) {
  return (
    <div
      className={`relative flex min-h-16 min-w-0 items-center gap-2.5 rounded-lg border px-3 py-2 pr-26 text-left ${panelClassNames[activity.state]}`}
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
