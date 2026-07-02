import type { TFunction } from "i18next"
import type { MetricCardsItem } from "@workspace/app-kit"
import type {
  AnalyticsOverviewResponse,
  AnalyticsPagesResponse,
  AnalyticsSourcesResponse,
  AnalyticsSummary,
} from "@/api"
import { resolveRouteDisplayInfo } from "../../route-labels"
import {
  formatAnalyticsDateLabel,
  formatDuration,
  formatInteger,
  formatPercent,
} from "../format"

export function buildMetricCards(
  summary: AnalyticsSummary | undefined,
  t: TFunction
): MetricCardsItem[] {
  return [
    {
      key: "pv",
      label: t("admin.analytics.metrics.pv.label"),
      value: formatInteger(summary?.pv ?? 0),
      tail: t("admin.analytics.metrics.pv.tail"),
    },
    {
      key: "uv",
      label: t("admin.analytics.metrics.uv.label"),
      value: formatInteger(summary?.uv ?? 0),
      tail: t("admin.analytics.metrics.uv.tail"),
    },
    {
      key: "questions",
      label: t("admin.analytics.metrics.questions.label"),
      value: formatInteger(summary?.agentQuestions ?? 0),
      tail: t("admin.analytics.metrics.questions.tail", {
        tokens: formatInteger(summary?.tokenUsed ?? 0),
      }),
      variant: "accent",
    },
    {
      key: "quality",
      label: t("admin.analytics.metrics.quality.label"),
      value: formatDuration(summary?.avgSessionDurationSec ?? 0),
      tail: t("admin.analytics.metrics.quality.tail", {
        bounce: formatPercent(summary?.bounceRate ?? 0),
        errors: formatInteger(
          (summary?.frontendErrors ?? 0) + (summary?.apiErrors ?? 0)
        ),
      }),
      variant:
        (summary?.frontendErrors ?? 0) + (summary?.apiErrors ?? 0) > 0
          ? "warning"
          : "success",
    },
  ]
}

export function buildTrafficTrend(
  overview: AnalyticsOverviewResponse | undefined
) {
  return (
    overview?.dailyTraffic.map((row) => ({
      date: formatAnalyticsDateLabel(row.date),
      pv: row.pv,
      uv: row.uv,
      sessions: row.sessions,
      questions: row.agentQuestions,
    })) ?? []
  )
}

export function buildSourceKindChart(
  sources: AnalyticsSourcesResponse | undefined,
  t: TFunction
) {
  return (
    sources?.sourceKinds.slice(0, 8).map((row) => ({
      source: translateSourceKind(row.key, t),
      pv: row.pv,
    })) ?? []
  )
}

export function buildPageChart(
  pages: AnalyticsPagesResponse | undefined,
  t: TFunction
) {
  return (
    pages?.items.slice(0, 8).map((row) => ({
      page: resolveRouteDisplayInfo(row.routePath, t).label,
      pv: row.pv,
    })) ?? []
  )
}

function translateSourceKind(value: string, t: TFunction) {
  const key = `admin.analytics.sources.kinds.${value}`
  const translated = t(key)
  return translated === key ? value : translated
}
