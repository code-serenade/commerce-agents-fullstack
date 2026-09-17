import { BadgeCheck, BarChart3, BookOpenCheck, Clock3 } from "lucide-react"
import { useTranslation } from "react-i18next"

export function LearningProgressSummary({
  assessedCount,
  coverageRate,
  demonstratedCount,
  lastAnalyzedAt,
  multiAngleCount,
  observedCount,
  retainedCount,
  totalDomains,
}: {
  assessedCount: number
  coverageRate: number
  demonstratedCount: number
  lastAnalyzedAt?: string | null
  multiAngleCount: number
  observedCount: number
  retainedCount: number
  totalDomains: number
}) {
  const { t } = useTranslation()
  const cards = [
    {
      icon: BookOpenCheck,
      label: t("learningProgress.coverage"),
      value: `${coverageRate}%`,
    },
    {
      icon: BarChart3,
      label: t("learningProgress.observed"),
      value: String(observedCount),
    },
    {
      icon: BadgeCheck,
      label: t("learningProgress.assessed"),
      value: t("learningProgress.assessedValue", {
        count: assessedCount,
        total: totalDomains,
      }),
      details: [
        t("learningProgress.multiAngleAssessed", {
          count: multiAngleCount,
          total: totalDomains,
        }),
        t("learningProgress.demonstrated", {
          count: demonstratedCount,
          total: totalDomains,
        }),
        t("learningProgress.retained", {
          count: retainedCount,
          total: totalDomains,
        }),
      ],
    },
    {
      icon: Clock3,
      label: t("learningProgress.updated"),
      value: lastAnalyzedAt
        ? new Date(lastAnalyzedAt).toLocaleString()
        : t("learningProgress.pending"),
    },
  ]

  return (
    <div className="grid grid-cols-2 gap-3 xl:grid-cols-4">
      {cards.map(({ details, icon: Icon, label, value }) => (
        <section
          key={label}
          className="rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) p-4 shadow-sm backdrop-blur-xl"
        >
          <Icon className="size-4 text-primary" />
          <p className="mt-3 text-xs text-muted-foreground">{label}</p>
          <p className="mt-1 text-lg font-semibold text-foreground">{value}</p>
          {details?.map((detail) => (
            <p key={detail} className="mt-1 text-xs/5 text-muted-foreground">
              {detail}
            </p>
          ))}
        </section>
      ))}
    </div>
  )
}
