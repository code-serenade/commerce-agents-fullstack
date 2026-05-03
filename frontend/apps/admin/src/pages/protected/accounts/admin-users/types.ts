import type { AdminUserStatus as ApiAdminUserStatus } from "@/api"

export type AdminUserStatusFilter = ApiAdminUserStatus

export interface AdminUserRow {
  user_id: string
  display_id: string
  display_name: string
  remark: string | null
  status: ApiAdminUserStatus
  roles: string[]
}

export interface AdminUserTableQuery {
  keyword: string
  status: "" | AdminUserStatusFilter
}
