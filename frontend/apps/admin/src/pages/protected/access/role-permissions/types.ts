import type { PermissionKind } from "@/api"

export type PermissionSummaryType = Extract<PermissionKind, "group" | "action">

export type SelectedPermissionIdsByRole = Record<string, string[]>
