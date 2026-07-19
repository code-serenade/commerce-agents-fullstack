import type { AgentRoleKey } from "../view-model/agent-role-cluster-view-model"
import type { InlineAgentState } from "../view-model/thread-inline-view-model"

export type AgentRoleClusterVariant = "compact" | "showcase" | "timeline"

export const panelClassNames: Record<InlineAgentState, string> = {
  boundary:
    "border-amber-200/80 bg-(--agent-chat-elevated-strong) shadow-sm dark:border-amber-900/60",
  done: "border-emerald-200/80 bg-(--agent-chat-elevated-strong) shadow-sm dark:border-emerald-900/60",
  failed: "border-destructive/25 bg-(--agent-chat-elevated-strong) shadow-sm",
  idle: "border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) shadow-sm",
  queued:
    "border-sky-200/80 bg-(--agent-chat-elevated-strong) shadow-sm dark:border-sky-900/60",
  reworking:
    "border-violet-200/80 bg-(--agent-chat-elevated-strong) shadow-sm dark:border-violet-900/60",
  working: "border-primary/35 bg-(--agent-chat-elevated-strong) shadow-sm",
}

export const roleAccentClassNames: Record<AgentRoleKey, string> = {
  accounting: "border-t-emerald-500",
  box: "border-t-slate-500",
  creative: "border-t-violet-500",
  data: "border-t-teal-600",
  delivery: "border-t-blue-500",
  design: "border-t-fuchsia-500",
  engineering: "border-t-slate-700",
  ops: "border-t-blue-600",
  search: "border-t-cyan-500",
  service: "border-t-sky-500",
  web: "border-t-indigo-500",
}

export const roleDotClassNames: Record<AgentRoleKey, string> = {
  accounting: "bg-emerald-500",
  box: "bg-slate-500",
  creative: "bg-violet-500",
  data: "bg-teal-600",
  delivery: "bg-blue-500",
  design: "bg-fuchsia-500",
  engineering: "bg-slate-700",
  ops: "bg-blue-600",
  search: "bg-cyan-500",
  service: "bg-sky-500",
  web: "bg-indigo-500",
}

export const roleScrimAccentColors: Record<AgentRoleKey, string> = {
  accounting: "16 185 129",
  box: "100 116 139",
  creative: "139 92 246",
  data: "13 148 136",
  delivery: "59 130 246",
  design: "217 70 239",
  engineering: "51 65 85",
  ops: "37 99 235",
  search: "6 182 212",
  service: "14 165 233",
  web: "99 102 241",
}
