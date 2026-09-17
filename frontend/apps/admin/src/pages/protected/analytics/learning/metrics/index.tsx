import {
  BadgeCheck,
  BookOpenCheck,
  CircleCheckBig,
  CircleSlash2,
  ClipboardCheck,
  SendHorizontal,
  Timer,
  TriangleAlert,
  UsersRound,
  UserRoundCheck,
} from "lucide-react"
import type { MetricCardsItem } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { LearningStudentMetricsResponse } from "@/api"

export function buildLearningMetricCards(
  metrics: LearningStudentMetricsResponse | undefined,
  t: TFunction
): MetricCardsItem[] {
  return [
    {
      key: "total",
      label: t("admin.analytics.learning.metrics.total"),
      value: metrics ? String(metrics.total) : "-",
      icon: <UsersRound className="size-4" />,
    },
    {
      key: "enabled",
      label: t("admin.analytics.learning.metrics.enabled"),
      value: metrics ? String(metrics.enabled) : "-",
      icon: <UserRoundCheck className="size-4" />,
      variant: "success",
    },
    {
      key: "observed",
      label: t("admin.analytics.learning.metrics.observed"),
      value: metrics ? String(metrics.observed) : "-",
      icon: <BookOpenCheck className="size-4" />,
      variant: "accent",
      tail: metrics
        ? t("admin.analytics.learning.metrics.avgCoverageTail", {
            rate: Math.round(metrics.avgCoverageRate * 100),
          })
        : undefined,
    },
    {
      key: "assessed",
      label: t("admin.analytics.learning.metrics.assessed"),
      value: metrics ? String(metrics.assessed ?? 0) : "-",
      icon: <ClipboardCheck className="size-4" />,
    },
    {
      key: "demonstrated",
      label: t("admin.analytics.learning.metrics.demonstrated"),
      value: metrics ? String(metrics.demonstrated ?? 0) : "-",
      icon: <BadgeCheck className="size-4" />,
      variant: "success",
      tail: metrics
        ? t("admin.analytics.learning.metrics.retainedTail", {
            count: metrics.retained ?? 0,
          })
        : undefined,
    },
    {
      key: "needsAttention",
      label: t("admin.analytics.learning.metrics.needsAttention"),
      value: metrics ? String(metrics.needsAttention ?? 0) : "-",
      icon: <TriangleAlert className="size-4" />,
      variant: "warning",
    },
  ]
}

export function buildLearningPilotMetricCards(
  metrics: LearningStudentMetricsResponse | undefined,
  t: TFunction
): MetricCardsItem[] {
  const pilot = metrics?.probePilot
  const loaded = Boolean(metrics)
  const value = (count: number | undefined) =>
    loaded ? String(count ?? 0) : "-"
  const percent = (rate: number | undefined) =>
    Math.round(Math.min(1, Math.max(0, rate ?? 0)) * 100)

  return [
    {
      key: "issuedProbes",
      label: t("admin.analytics.learning.pilotMetrics.issued"),
      value: value(pilot?.issuedCount),
      icon: <SendHorizontal className="size-4" />,
      tail: loaded
        ? t("admin.analytics.learning.pilotMetrics.issuedTail", {
            active: pilot?.activeCount ?? 0,
            superseded: pilot?.supersededCount ?? 0,
          })
        : undefined,
    },
    {
      key: "answeredProbes",
      label: t("admin.analytics.learning.pilotMetrics.answered"),
      value: value(pilot?.answeredCount),
      icon: <Timer className="size-4" />,
      variant: "accent",
      tail: loaded
        ? t("admin.analytics.learning.pilotMetrics.answeredTail", {
            rate: percent(pilot?.answerRate),
            seconds: Math.max(0, Math.round(pilot?.averageAnswerSeconds ?? 0)),
          })
        : undefined,
    },
    {
      key: "skippedProbes",
      label: t("admin.analytics.learning.pilotMetrics.skipped"),
      value: value(pilot?.skippedCount),
      icon: <CircleSlash2 className="size-4" />,
      variant: "warning",
      tail: loaded
        ? t("admin.analytics.learning.pilotMetrics.skippedTail", {
            rate: percent(pilot?.skipRate),
            expired: pilot?.expiredCount ?? 0,
          })
        : undefined,
    },
    {
      key: "correctProbes",
      label: t("admin.analytics.learning.pilotMetrics.correct"),
      value: value(pilot?.correctCount),
      icon: <CircleCheckBig className="size-4" />,
      variant: "success",
      tail: loaded
        ? t("admin.analytics.learning.pilotMetrics.correctTail", {
            rate: percent(pilot?.correctRate),
            continuations: pilot?.continuationCount ?? 0,
          })
        : undefined,
    },
  ]
}

export { buildLearningQuestionGovernanceMetricCards } from "./components/question-governance-metrics"
