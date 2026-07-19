import type { CSSProperties } from "react"
import { getAgentPortraitAsset } from "../assets/agent-portraits/agent-portrait-assets"
import type { AgentRoleActivity } from "../view-model/agent-role-cluster-view-model"
import { AgentRolePortrait } from "./agent-role-portrait"
import { roleScrimAccentColors } from "./agent-role-cluster-styles"

export function AgentRoleSceneItem({
  activity,
  detail,
}: {
  activity: AgentRoleActivity
  detail: string
}) {
  const asset = getAgentPortraitAsset(activity.role)
  const description = `${detail} ${activity.contribution}`
  const cardStyle = {
    "--agent-role-card-scrim-accent": roleScrimAccentColors[activity.role],
  } as CSSProperties

  return (
    <div
      className="group relative aspect-3/4 overflow-hidden rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) shadow-(--agent-chat-card-shadow) transition hover:border-primary/40 hover:shadow-(--agent-chat-focus-shadow)"
      data-agent-role-card={activity.role}
      style={cardStyle}
      title={description}
    >
      <div
        className="absolute inset-0 overflow-hidden"
        data-agent-portrait-frame="rect"
      >
        <div className="absolute inset-0 bg-[linear-gradient(135deg,var(--agent-chat-elevated-strong),var(--agent-chat-accent-soft)_48%,var(--agent-chat-subtle-surface))]" />
        {asset ? (
          <img
            alt=""
            className="agent-role-portrait__sprite absolute inset-0 mt-[8%] w-full object-cover opacity-95 transition-transform duration-300 group-hover:scale-[1.03]"
            data-agent-portrait-asset={asset.key}
            data-agent-portrait-state={activity.state}
            data-agent-portrait-source="static"
            src={asset.src}
          />
        ) : (
          <div className="absolute inset-x-6 top-5">
            <AgentRolePortrait activity={activity} size="card" />
          </div>
        )}
      </div>
      <div className="absolute inset-x-0 top-0 z-10 bg-[linear-gradient(175deg,rgb(var(--agent-role-card-scrim-accent)/0.56)_0%,rgb(var(--agent-role-card-scrim-accent)/0.46)_32%,rgb(var(--agent-role-card-scrim-accent)/0.14)_58%,transparent_84%)] px-3 pt-2 pr-10 pb-5">
        <p className="truncate text-sm font-semibold text-(--agent-chat-role-card-foreground)">
          {activity.agent.name}
        </p>
      </div>
      <div className="absolute inset-x-0 bottom-0 z-10 flex min-h-[42%] flex-col justify-end bg-[linear-gradient(180deg,transparent_0%,rgb(0_0_0/0.12)_12%,rgb(var(--agent-role-card-scrim-accent)/0.48)_44%,rgb(var(--agent-role-card-scrim-accent)/0.88)_100%)] px-3 pt-10 pb-3">
        <p
          className="line-clamp-2 min-h-8 text-[11px]/4 text-(--agent-chat-role-card-muted)"
          title={description}
        >
          {description}
        </p>
      </div>
    </div>
  )
}
