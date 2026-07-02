import type { AgentRoleKey } from "../view-model/agent-role-cluster-view-model"
import type { InlineAgentState } from "../view-model/thread-inline-view-model"

export const bodyClassNames: Record<InlineAgentState, string> = {
  boundary: "bg-amber-500",
  done: "bg-emerald-500",
  failed: "bg-destructive",
  idle: "bg-muted-foreground/35",
  queued: "bg-sky-400",
  reworking: "bg-violet-500",
  working: "bg-primary",
}

export const portraitRoleClassNames: Record<AgentRoleKey, string> = {
  accounting: "bg-emerald-500",
  box: "bg-slate-500",
  creative: "bg-violet-500",
  data: "bg-teal-600",
  delivery: "bg-blue-500",
  design: "bg-fuchsia-500",
  engineering: "bg-slate-700",
  ops: "bg-blue-600",
  search: "bg-cyan-500",
  service: "bg-cyan-600",
  web: "bg-indigo-500",
}

export const ringClassNames: Record<InlineAgentState, string> = {
  boundary: "ring-amber-300/70",
  done: "ring-emerald-300/70",
  failed: "ring-destructive/45",
  idle: "ring-border",
  queued: "ring-sky-300/70",
  reworking: "ring-violet-300/70",
  working: "ring-primary/45",
}

