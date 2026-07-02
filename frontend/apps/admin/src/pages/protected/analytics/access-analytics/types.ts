export interface AnalyticsPageReference {
  description: string
  label: string
  path: string
  pv: number
}

export interface AnalyticsAccountRow {
  user_id: string
  display_id: string
  display_name: string
  school_name: string
  team_name: string
  pv: number
  sessions: number
  active_days: number
  agent_questions: number
  token_used: number
  lastSeenAt: string | null
  top_pages: AnalyticsPageReference[]
}

export interface AnalyticsSchoolTeamRow {
  id: string
  school_name: string
  team_name: string
  account_count: number
  active_accounts: number
  pv: number
  uv: number
  sessions: number
  agent_questions: number
  token_used: number
  lastSeenAt: string | null
}

export interface AnalyticsAccountTableQuery {
  keyword: string
  schoolName: string
  teamName: string
}

export interface AnalyticsSchoolTeamTableQuery {
  schoolName: string
  teamName: string
}

export interface AnalyticsDateRange {
  from: string
  to: string
}
