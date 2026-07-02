import type { TFunction } from "i18next"
import type {
  AnalyticsDailyTraffic,
  AnalyticsPageMetric,
  AnalyticsSchoolTeamRowResponse,
  TeamUsageResponse,
} from "@/api"
import {
  formatAnalyticsDateLabel,
  formatInteger,
} from "../analytics/access-analytics/format"
import { resolveRouteDisplayInfo } from "../analytics/route-labels"
import type {
  DashboardPageRow,
  DashboardSchoolTeamRow,
  DashboardStatusRow,
  DashboardUsageAlert,
} from "./dashboard-types"

export function buildTrendRows(rows: AnalyticsDailyTraffic[]) {
  return rows.map((row) => ({
    date: formatAnalyticsDateLabel(row.date),
    pv: row.pv,
    questions: row.agentQuestions,
    uv: row.uv,
  }))
}

export function mapTopPages(
  rows: AnalyticsPageMetric[],
  t: TFunction
): DashboardPageRow[] {
  return rows.slice(0, 8).map((row) => {
    const route = resolveRouteDisplayInfo(row.routePath, t)
    return {
      description: route.description,
      label: route.label,
      path: route.rawPath,
      pv: row.pv,
      uv: row.uv,
    }
  })
}

export function mapSchoolTeams(
  rows: AnalyticsSchoolTeamRowResponse[],
  t: TFunction
): DashboardSchoolTeamRow[] {
  return rows.slice(0, 5).map((row) => ({
    activeAccounts: row.activeAccounts,
    agentQuestions: row.agentQuestions,
    label: row.schoolName && row.teamName
      ? `${row.schoolName} / ${row.teamName}`
      : t("admin.dashboard.empty.schoolTeam"),
    pv: row.pv,
  }))
}

export function mapUsageAlerts(
  rows: TeamUsageResponse[],
  t: TFunction
): DashboardUsageAlert[] {
  return rows.slice(0, 5).map((row) => ({
    account: `${row.displayName} (${row.displayId})`,
    remaining: row.questionRemaining,
    schoolTeam:
      row.schoolName && row.teamName
        ? `${row.schoolName} / ${row.teamName}`
        : t("admin.dashboard.empty.schoolTeam"),
    used: row.questionUsed,
  }))
}

export function buildStatusRows({
  appUsersLoaded,
  overviewLoaded,
  pagesLoaded,
  schoolTeamsLoaded,
  teamUsageLoaded,
  t,
}: {
  appUsersLoaded: boolean
  overviewLoaded: boolean
  pagesLoaded: boolean
  schoolTeamsLoaded: boolean
  teamUsageLoaded: boolean
  t: TFunction
}): DashboardStatusRow[] {
  return [
    {
      detail: t("admin.dashboard.statusRows.analytics.detail"),
      name: t("admin.dashboard.statusRows.analytics.name"),
      status: overviewLoaded && pagesLoaded
        ? t("admin.dashboard.statusRows.ready")
        : t("admin.dashboard.statusRows.loading"),
    },
    {
      detail: t("admin.dashboard.statusRows.accounts.detail"),
      name: t("admin.dashboard.statusRows.accounts.name"),
      status: appUsersLoaded
        ? t("admin.dashboard.statusRows.ready")
        : t("admin.dashboard.statusRows.loading"),
    },
    {
      detail: t("admin.dashboard.statusRows.usage.detail"),
      name: t("admin.dashboard.statusRows.usage.name"),
      status: teamUsageLoaded && schoolTeamsLoaded
        ? t("admin.dashboard.statusRows.ready")
        : t("admin.dashboard.statusRows.loading"),
    },
  ]
}

export { formatInteger }
