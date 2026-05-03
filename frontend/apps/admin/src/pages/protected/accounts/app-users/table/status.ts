import type { AppUserStatus as ApiAppUserStatus } from "@/api"
import type { TFunction } from "i18next"
import type { AppUserStatusFilter } from "../types"

export function mapApiStatusToLabel(
  status: ApiAppUserStatus,
  t: TFunction
): string {
  return t(`admin.accounts.appUsers.table.status.${status}`)
}

export function mapStatusLabelToApiStatus(
  status: AppUserStatusFilter
): ApiAppUserStatus {
  return status
}
