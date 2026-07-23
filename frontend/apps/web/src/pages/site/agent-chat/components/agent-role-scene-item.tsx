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
      className="group relative aspect-[0.94/1] overflow-hidden rounded-lg border border-[#dce7f4] bg-(--agent-chat-elevated-strong) shadow-[0_8px_18px_rgb(62_91_126/0.06),inset_0_1px_0_rgb(255_255_255/0.9)] transition hover:-translate-y-0.5 hover:border-[#b7cdf0] hover:shadow-[0_12px_24px_rgb(62_91_126/0.1),inset_0_1px_0_rgb(255_255_255/0.96)] dark:border-(--agent-chat-soft-border)"
      data-agent-role-card={activity.role}
      style={cardStyle}
      title={description}
    >
      <div
        className="absolute inset-0 overflow-hidden"
        data-agent-portrait-frame="rect"
      >
        <div className="absolute inset-0 bg-[linear-gradient(180deg,#ffffff_0%,#ffffff_60%,#f0f6ff_100%)] dark:bg-[linear-gradient(180deg,var(--agent-chat-elevated-strong)_0%,var(--agent-chat-elevated-strong)_60%,var(--agent-chat-accent-soft)_100%)]" />
        {asset ? (
          <img
            alt=""
            className="agent-role-portrait__sprite absolute inset-x-0 top-2.5 mx-auto h-[55%] w-full object-contain opacity-100 transition-transform duration-300 group-hover:scale-[1.03]"
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
      <div className="absolute inset-x-0 bottom-0 z-10 flex min-h-[45%] flex-col items-center justify-end bg-[linear-gradient(180deg,transparent_0%,rgb(255_255_255/0.94)_22%,#edf5ff_100%)] px-3 pt-8 pb-4 text-center dark:bg-[linear-gradient(180deg,transparent_0%,var(--agent-chat-elevated-strong)_22%,var(--agent-chat-accent-soft)_100%)]">
        <p className="truncate text-sm/5 font-bold text-foreground">
          {activity.agent.name}
        </p>
        <p
          className="mt-2 line-clamp-2 min-h-10 text-[13px]/5 text-[#53677d] dark:text-muted-foreground"
          title={description}
        >
          {description}
        </p>
      </div>
    </div>
  )
}
