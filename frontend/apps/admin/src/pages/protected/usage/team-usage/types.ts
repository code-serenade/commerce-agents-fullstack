import type { TeamUsageStatus as ApiTeamUsageStatus } from "@/api"

export type TeamUsageStatusFilter = ApiTeamUsageStatus

export interface TeamUsageRow {
  id: string
  school_name: string
  team_name: string
  member_count: number
  question_limit: number
  question_used: number
  question_remaining: number
  token_used: number
  status: ApiTeamUsageStatus
  remark: string | null
  lastUsedAt: string | null
  createdAt: string | null
  updatedAt: string | null
}

export interface TeamUsageTableQuery {
  keyword: string
  status: "" | TeamUsageStatusFilter
}
