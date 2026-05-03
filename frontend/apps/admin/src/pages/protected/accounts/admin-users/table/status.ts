import type { AdminUserStatus as ApiAdminUserStatus } from "@/api"
import type { TFunction } from "i18next"
import type { AdminUserStatusFilter } from "../types"

export function mapApiStatusToLabel(
  status: ApiAdminUserStatus,
  t: TFunction
): string {
  return t(`admin.accounts.adminUsers.table.status.${status}`)
}

export function mapStatusLabelToApiStatus(
  status: AdminUserStatusFilter
): ApiAdminUserStatus {
  return status
}
