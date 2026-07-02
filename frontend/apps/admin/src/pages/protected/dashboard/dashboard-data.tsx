import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import { useQuery } from "@tanstack/react-query"
import {
  MessageSquareText,
  MousePointerClick,
  School,
  UsersRound,
} from "lucide-react"
import type { MetricCardsItem } from "@workspace/app-kit"
import {
  getAnalyticsOverviewApi,
  getAnalyticsPagesApi,
  listAnalyticsSchoolTeamsApi,
  listAppUserMetricsApi,
  listTeamUsagesApi,
} from "@/api"
import {
  defaultDateRange,
  formatDuration,
  formatInteger,
  formatPercent,
  toEndIso,
  toStartIso,
} from "../analytics/access-analytics/format"
import {
  buildStatusRows,
  buildTrendRows,
  mapSchoolTeams,
  mapTopPages,
  mapUsageAlerts,
} from "./dashboard-view-model"
import type { DashboardProductMetric } from "./dashboard-types"

export function useDashboardData() {
  const { t } = useTranslation()
  const range = useMemo(() => defaultDateRange(), [])
  const rangeParams = useMemo(
    () => ({
      from: toStartIso(range.from),
      to: toEndIso(range.to),
      granularity: "day" as const,
    }),
    [range]
  )

  const overviewQuery = useQuery({
    queryKey: ["admin", "dashboard", "analytics-overview", rangeParams],
    queryFn: () => getAnalyticsOverviewApi(rangeParams),
  })
  const pagesQuery = useQuery({
    queryKey: ["admin", "dashboard", "analytics-pages", rangeParams],
    queryFn: () => getAnalyticsPagesApi(rangeParams),
  })
  const appUserMetricsQuery = useQuery({
    queryKey: ["admin", "dashboard", "app-user-metrics"],
    queryFn: listAppUserMetricsApi,
  })
  const teamUsageQuery = useQuery({
    queryKey: ["admin", "dashboard", "team-usage"],
    queryFn: () =>
      listTeamUsagesApi({
        page: 1,
        pageSize: 5,
        sortBy: "questionRemaining",
        sortOrder: "asc",
      }),
  })
  const schoolTeamsQuery = useQuery({
    queryKey: ["admin", "dashboard", "school-teams", rangeParams],
    queryFn: () =>
      listAnalyticsSchoolTeamsApi({
        ...rangeParams,
        page: 1,
        pageSize: 5,
        sortBy: "activeAccounts",
        sortOrder: "desc",
      }),
  })

  const summary = overviewQuery.data?.summary
  const appUsers = appUserMetricsQuery.data

  const metricCards = useMemo<MetricCardsItem[]>(
    () => [
      {
        key: "pv",
        label: t("admin.dashboard.metrics.pv.label"),
        value: formatInteger(summary?.pv ?? 0),
        tail: t("admin.dashboard.metrics.pv.tail", {
          sessions: formatInteger(summary?.sessions ?? 0),
        }),
        icon: <MousePointerClick className="size-4" />,
      },
      {
        key: "questions",
        label: t("admin.dashboard.metrics.questions.label"),
        value: formatInteger(summary?.agentQuestions ?? 0),
        tail: t("admin.dashboard.metrics.questions.tail", {
          tokens: formatInteger(summary?.tokenUsed ?? 0),
        }),
        icon: <MessageSquareText className="size-4" />,
        variant: "accent",
      },
      {
        key: "accounts",
        label: t("admin.dashboard.metrics.accounts.label"),
        value: formatInteger(appUsers?.total ?? 0),
        tail: t("admin.dashboard.metrics.accounts.tail", {
          enabled: formatInteger(appUsers?.enabled ?? 0),
        }),
        icon: <UsersRound className="size-4" />,
        variant: "success",
      },
      {
        key: "teams",
        label: t("admin.dashboard.metrics.teams.label"),
        value: formatInteger(schoolTeamsQuery.data?.total ?? 0),
        tail: t("admin.dashboard.metrics.teams.tail", {
          active: formatInteger(summary?.activeAccounts ?? 0),
        }),
        icon: <School className="size-4" />,
        variant: "warning",
      },
    ],
    [appUsers, schoolTeamsQuery.data?.total, summary, t]
  )

  const trend = useMemo(
    () => buildTrendRows(overviewQuery.data?.dailyTraffic ?? []),
    [overviewQuery.data?.dailyTraffic]
  )

  const productMix = useMemo<DashboardProductMetric[]>(
    () => [
      {
        key: "activeAccounts",
        label: t("admin.dashboard.productMix.activeAccounts"),
        value: formatInteger(summary?.activeAccounts ?? 0),
      },
      {
        key: "avgSession",
        label: t("admin.dashboard.productMix.avgSession"),
        value: formatDuration(summary?.avgSessionDurationSec ?? 0),
      },
      {
        key: "bounceRate",
        label: t("admin.dashboard.productMix.bounceRate"),
        value: formatPercent(summary?.bounceRate ?? 0),
      },
      {
        key: "errors",
        label: t("admin.dashboard.productMix.errors"),
        value: formatInteger(
          (summary?.frontendErrors ?? 0) + (summary?.apiErrors ?? 0)
        ),
      },
    ],
    [summary, t]
  )

  const topPages = useMemo(
    () => mapTopPages(pagesQuery.data?.items ?? [], t),
    [pagesQuery.data?.items, t]
  )
  const schoolTeams = useMemo(
    () => mapSchoolTeams(schoolTeamsQuery.data?.items ?? [], t),
    [schoolTeamsQuery.data?.items, t]
  )
  const usageAlerts = useMemo(
    () => mapUsageAlerts(teamUsageQuery.data?.items ?? [], t),
    [teamUsageQuery.data?.items, t]
  )
  const statusRows = useMemo(
    () =>
      buildStatusRows({
        appUsersLoaded: appUserMetricsQuery.isSuccess,
        overviewLoaded: overviewQuery.isSuccess,
        pagesLoaded: pagesQuery.isSuccess,
        schoolTeamsLoaded: schoolTeamsQuery.isSuccess,
        teamUsageLoaded: teamUsageQuery.isSuccess,
        t,
      }),
    [
      appUserMetricsQuery.isSuccess,
      overviewQuery.isSuccess,
      pagesQuery.isSuccess,
      schoolTeamsQuery.isSuccess,
      teamUsageQuery.isSuccess,
      t,
    ]
  )

  return {
    isLoading:
      overviewQuery.isLoading ||
      pagesQuery.isLoading ||
      appUserMetricsQuery.isLoading ||
      teamUsageQuery.isLoading ||
      schoolTeamsQuery.isLoading,
    metricCards,
    productMix,
    range,
    schoolTeams,
    statusRows,
    topPages,
    trend,
    usageAlerts,
  }
}
