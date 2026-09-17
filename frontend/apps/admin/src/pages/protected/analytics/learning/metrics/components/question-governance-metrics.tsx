import {
  BookOpenCheck,
  CircleCheckBig,
  ClipboardCheck,
  Layers3,
  Timer,
  TriangleAlert,
  Unplug,
} from "lucide-react"
import type { MetricCardsItem } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { LearningStudentMetricsResponse } from "@/api"

function shortQuestionBankVersion(version: string | undefined) {
  const normalized = version?.trim()
  if (!normalized) return "-"
  const match = /^ecommerce_diagnostic_(v\d+)$/.exec(normalized)
  return match?.[1] ?? normalized
}

export function buildLearningQuestionGovernanceMetricCards(
  metrics: LearningStudentMetricsResponse | undefined,
  t: TFunction
): MetricCardsItem[] {
  const pilot = metrics?.probePilot
  const loaded = typeof pilot?.questionCount === "number"
  const value = (count: number | undefined) =>
    loaded ? String(count ?? 0) : "-"
  const approvedCount = pilot?.productionApprovedCount ?? 0
  const pendingCount = pilot?.curriculumReviewPendingCount ?? 0
  const questionCount = pilot?.questionCount ?? 0
  const angleDepthLoaded =
    typeof pilot?.assessmentAngleCount === "number" &&
    typeof pilot?.singleQuestionAngleCount === "number"
  const singleQuestionAngleCount = pilot?.singleQuestionAngleCount ?? 0
  const topGapDomain = pilot?.coverageGapDomainMetrics?.[0]

  return [
    {
      key: "questionBankVersion",
      label: t("admin.analytics.learning.questionGovernance.version"),
      value: shortQuestionBankVersion(pilot?.questionBankVersion),
      icon: <BookOpenCheck className="size-4" />,
      tail: t("admin.analytics.learning.questionGovernance.versionTail"),
    },
    {
      key: "questionCount",
      label: t("admin.analytics.learning.questionGovernance.total"),
      value: value(pilot?.questionCount),
      icon: <ClipboardCheck className="size-4" />,
      tail: t("admin.analytics.learning.questionGovernance.totalTail"),
    },
    {
      key: "assessmentAngleCount",
      label: t("admin.analytics.learning.questionGovernance.angles"),
      value: angleDepthLoaded ? String(pilot?.assessmentAngleCount ?? 0) : "-",
      icon: <Layers3 className="size-4" />,
      variant:
        angleDepthLoaded && singleQuestionAngleCount === 0 ? "success" : "warning",
      tail: angleDepthLoaded
        ? t("admin.analytics.learning.questionGovernance.anglesTail", {
            count: singleQuestionAngleCount,
          })
        : t("admin.analytics.learning.questionGovernance.anglesUnavailableTail"),
    },
    {
      key: "productionApprovedCount",
      label: t("admin.analytics.learning.questionGovernance.approved"),
      value: value(pilot?.productionApprovedCount),
      icon: <CircleCheckBig className="size-4" />,
      variant:
        loaded && questionCount > 0 && approvedCount === questionCount
          ? "success"
          : "warning",
      tail: t("admin.analytics.learning.questionGovernance.approvedTail"),
    },
    {
      key: "curriculumReviewPendingCount",
      label: t("admin.analytics.learning.questionGovernance.pending"),
      value: value(pilot?.curriculumReviewPendingCount),
      icon: <TriangleAlert className="size-4" />,
      variant: loaded && pendingCount === 0 ? "success" : "warning",
      tail: t("admin.analytics.learning.questionGovernance.pendingTail"),
    },
    {
      key: "earliestSourceValidThrough",
      label: t("admin.analytics.learning.questionGovernance.validThrough"),
      value: loaded
        ? pilot?.earliestSourceValidThrough?.trim() || "-"
        : "-",
      icon: <Timer className="size-4" />,
      variant: "accent",
      tail: t("admin.analytics.learning.questionGovernance.validThroughTail"),
    },
    {
      key: "coverageGapCount",
      label: t("admin.analytics.learning.questionGovernance.coverageGaps"),
      value: value(pilot?.coverageGapCount),
      icon: <Unplug className="size-4" />,
      variant:
        loaded && (pilot?.coverageGapCount ?? 0) === 0
          ? "success"
          : "warning",
      tail:
        loaded && topGapDomain
          ? t("admin.analytics.learning.questionGovernance.coverageGapsTail", {
              domain: topGapDomain.code,
              count: topGapDomain.count,
            })
          : t(
              "admin.analytics.learning.questionGovernance.coverageGapsEmptyTail"
            ),
    },
  ]
}
