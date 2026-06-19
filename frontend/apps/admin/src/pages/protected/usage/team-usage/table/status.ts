import type { TeamUsageStatus as ApiTeamUsageStatus } from "@/api"
import type { TFunction } from "i18next"
import type { TeamUsageStatusFilter } from "../types"

export function mapApiStatusToLabel(
  status: ApiTeamUsageStatus,
  t: TFunction
): string {
  return t(`admin.usage.teamUsage.table.status.${status}`)
}

export function mapStatusLabelToApiStatus(
  status: TeamUsageStatusFilter
): ApiTeamUsageStatus {
  return status
}
