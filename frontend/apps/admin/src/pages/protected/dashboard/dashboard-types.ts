export interface DashboardProductMetric {
  key: string
  label: string
  value: string
}

export interface DashboardPageRow {
  description: string
  label: string
  path: string
  pv: number
  uv: number
}

export interface DashboardSchoolTeamRow {
  activeAccounts: number
  agentQuestions: number
  label: string
  pv: number
}

export interface DashboardUsageAlert {
  account: string
  remaining: number
  schoolTeam: string
  used: number
}

export interface DashboardStatusRow {
  detail: string
  name: string
  status: string
}
