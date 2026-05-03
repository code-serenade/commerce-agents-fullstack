import type { DataTableAuditColumnKey } from "@workspace/app-kit"
import type { AppUserTableQuery } from "./types"

export const appUserAuditColumns = [
  "createdAt",
  "updatedAt",
] as const satisfies readonly DataTableAuditColumnKey[]

const appUserAuditColumnList =
  appUserAuditColumns as readonly DataTableAuditColumnKey[]

export const appUserDefaultTimeField: DataTableAuditColumnKey | undefined =
  appUserAuditColumnList[0]

export const appUserMetricsQueryKey = [
  "admin",
  "app-users",
  "metrics",
] as const

export const appUserInitialQuery: AppUserTableQuery = {
  keyword: "",
  status: "",
  auditField: appUserDefaultTimeField,
  auditRange: undefined,
}

export const appUserPageSizeOptions = [10, 20, 50] as const
