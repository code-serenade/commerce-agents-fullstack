import type { TeamUsageTableQuery } from "./types"

export const teamUsageInitialQuery: TeamUsageTableQuery = {
  keyword: "",
  status: "",
}

export const teamUsagePageSizeOptions = [10, 20, 50] as const
