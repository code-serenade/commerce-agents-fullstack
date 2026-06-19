import type { AppUserStatus as ApiAppUserStatus } from "@/api"
import type {
  DataTableAuditColumnKey,
  DateRangeValue,
} from "@workspace/app-kit"

export type AppUserStatusFilter = ApiAppUserStatus
export type AppUserTimeField = DataTableAuditColumnKey

export interface AppUserRow {
  user_id: string
  display_id: string
  display_name: string
  school_name: string | null
  team_name: string | null
  remark: string | null
  status: ApiAppUserStatus
  createdAt: string | null
  updatedAt: string | null
  roles: string[]
}

export interface AppUserTableQuery {
  keyword: string
  status: "" | AppUserStatusFilter
  auditField?: AppUserTimeField
  auditRange: DateRangeValue | undefined
}
