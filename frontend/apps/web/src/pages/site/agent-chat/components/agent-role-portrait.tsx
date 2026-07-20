import {
  getAgentPortraitAsset,
  type AgentPortraitAsset,
} from "../assets/agent-portraits/agent-portrait-assets"
import type { AgentRoleActivity } from "../view-model/agent-role-cluster-view-model"
import { AgentRoleFallbackTool } from "./agent-role-fallback-tool"
import { bodyClassNames, ringClassNames } from "./agent-role-portrait-styles"

export function AgentRolePortrait({
  activity,
  animate = false,
  size = "md",
}: {
  activity: AgentRoleActivity
  animate?: boolean
  size?: "xs" | "sm" | "md" | "card"
}) {
  const asset = getAgentPortraitAsset(activity.role)

  if (asset) {
    return (
      <AgentPortraitImage
        animate={animate}
        activity={activity}
        asset={asset}
        size={size}
      />
    )
  }

  return <AgentPortraitFallback activity={activity} size={size} />
}

function AgentPortraitImage({
  animate,
  activity,
  asset,
  size,
}: {
  animate: boolean
  activity: AgentRoleActivity
  asset: AgentPortraitAsset
  size: "xs" | "sm" | "md" | "card"
}) {
  const frameClassName =
    size === "card"
      ? "aspect-square w-full"
      : size === "xs"
        ? "size-12"
        : size === "sm"
          ? "size-16"
          : "size-20"
  const imageClassName = "size-full"
  const surfaceClassName =
    size === "card"
      ? "rounded-t-xl rounded-b-none bg-transparent"
      : `rounded-md bg-background shadow-sm ring-1 ${ringClassNames[activity.state]}`
  const isWorking =
    animate && (activity.state === "working" || activity.state === "reworking")

  return (
    <div
      className={`relative mx-auto flex items-center justify-center ${frameClassName}`}
      aria-hidden="true"
      data-agent-portrait-frame="rect"
    >
      <span
        className={`absolute inset-0 ${surfaceClassName}`}
        data-agent-portrait-frame-surface="rect"
      />
      <img
        alt=""
        className={`agent-role-portrait__sprite relative z-10 object-contain ${imageClassName} ${
          isWorking ? "agent-role-portrait__sprite--working" : ""
        }`}
        data-agent-portrait-asset={asset.key}
        data-agent-portrait-role={asset.role}
        data-agent-portrait-state={activity.state}
        data-agent-portrait-source={isWorking ? "working" : "static"}
        src={asset.src}
      />
    </div>
  )
}

function AgentPortraitFallback({
  activity,
  size,
}: {
  activity: AgentRoleActivity
  size: "xs" | "sm" | "md" | "card"
}) {
  const scaleClassName =
    size === "card" ? "scale-[1.8]" : size === "sm" ? "scale-90" : ""
  const frameClassName = size === "card" ? "aspect-square w-full" : "h-12 w-14"

  return (
    <div
      className={`relative mx-auto ${frameClassName} ${scaleClassName}`}
      aria-hidden="true"
    >
      <span className="absolute inset-x-2 bottom-0 h-1 rounded-xs bg-muted-foreground/15" />
      <span
        className={`absolute top-0 left-5 size-4 rounded-xs border border-foreground/15 ${bodyClassNames[activity.state]}`}
      />
      <span className="absolute top-4 left-4 h-4 w-6 rounded-xs border border-foreground/15 bg-foreground/80" />
      <span
        className={`absolute top-8 left-2 h-2 w-3 rounded-xs ${bodyClassNames[activity.state]}`}
      />
      <span
        className={`absolute top-8 right-2 h-2 w-3 rounded-xs ${bodyClassNames[activity.state]}`}
      />
      <AgentRoleFallbackTool role={activity.role} state={activity.state} />
    </div>
  )
}
