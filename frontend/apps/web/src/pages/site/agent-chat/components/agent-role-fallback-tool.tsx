import type { AgentRoleKey } from "../view-model/agent-role-cluster-view-model"
import type { InlineAgentState } from "../view-model/thread-inline-view-model"
import { portraitRoleClassNames } from "./agent-role-portrait-styles"

export function AgentRoleFallbackTool({
  role,
  state,
}: {
  role: AgentRoleKey
  state: InlineAgentState
}) {
  if (role === "data") {
    return <DataFallbackTool />
  }
  if (role === "design") {
    return <DesignFallbackTool />
  }
  if (role === "engineering") {
    return <EngineeringFallbackTool />
  }
  if (role === "web") {
    return <WebFallbackTool />
  }
  if (role === "search") {
    return <SearchFallbackTool state={state} />
  }
  if (role === "accounting" || role === "creative" || role === "delivery") {
    return <SimpleFallbackTool role={role} />
  }

  return <SimpleFallbackTool role={role} />
}

function DataFallbackTool() {
  return (
    <span className="absolute right-0 top-3 flex h-6 w-5 items-end gap-0.5">
      {[2, 4, 6].map((height) => (
        <span
          key={height}
          className={`${portraitRoleClassNames.data} w-1 rounded-xs`}
          style={{ height }}
        />
      ))}
    </span>
  )
}

function DesignFallbackTool() {
  return (
    <span
      className={`absolute right-0 top-3 grid size-5 grid-cols-2 gap-px rounded-xs border border-foreground/20 p-0.5 ${portraitRoleClassNames.design}`}
    >
      <span className="rounded-xs bg-background/70" />
      <span className="rounded-xs bg-amber-300" />
      <span className="rounded-xs bg-sky-300" />
      <span className="rounded-xs bg-rose-300" />
    </span>
  )
}

function EngineeringFallbackTool() {
  return (
    <span
      className={`absolute right-0 top-4 h-4 w-5 rounded-xs border border-foreground/20 ${portraitRoleClassNames.engineering}`}
    >
      <span className="absolute left-1 top-1 h-0.5 w-3 rounded-xs bg-cyan-300" />
      <span className="absolute left-1 top-2.5 h-0.5 w-2 rounded-xs bg-emerald-300" />
    </span>
  )
}

function WebFallbackTool() {
  return (
    <span
      className={`absolute right-0 top-4 h-4 w-5 rounded-xs border border-indigo-300 ${portraitRoleClassNames.web}`}
    >
      <span className="absolute left-1 top-1 h-1 w-3 rounded-xs bg-background/70" />
      <span className="absolute bottom-0 right-0 size-2 rounded-full border border-background/80" />
    </span>
  )
}

function SearchFallbackTool({ state }: { state: InlineAgentState }) {
  return (
    <span
      className={`absolute right-0 top-4 size-4 rounded-full border-2 ${
        state === "failed" ? "border-destructive" : "border-cyan-500"
      }`}
    >
      <span className="absolute -bottom-1 -right-1 h-2 w-1 rotate-45 rounded-xs bg-cyan-500" />
    </span>
  )
}

function SimpleFallbackTool({ role }: { role: AgentRoleKey }) {
  const className = portraitRoleClassNames[role]
  const shapeClassName =
    role === "accounting"
      ? "size-4 rounded-full"
      : role === "creative"
        ? "h-6 w-1.5 rotate-12 rounded-xs"
        : "size-4 rounded-xs"

  return (
    <span
      className={`absolute right-0 top-4 border border-foreground/20 ${shapeClassName} ${className}`}
    />
  )
}
