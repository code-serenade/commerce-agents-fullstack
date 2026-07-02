import type {
  AnalyticsAccountTableQuery,
  AnalyticsSchoolTeamTableQuery,
} from "./types"

export const analyticsAccountInitialQuery: AnalyticsAccountTableQuery = {
  keyword: "",
  schoolName: "",
  teamName: "",
}

export const analyticsSchoolTeamInitialQuery: AnalyticsSchoolTeamTableQuery = {
  schoolName: "",
  teamName: "",
}

export const analyticsPageSizeOptions = [10, 20, 50] as const
