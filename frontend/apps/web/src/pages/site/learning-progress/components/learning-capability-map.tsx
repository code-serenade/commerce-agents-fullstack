import { Grid2X2Check, MessageSquareText } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Badge, Button, Progress } from "@workspace/ui-components"
import type { LearningCapabilityRow } from "../types"

export function LearningCapabilityMap({
  rows,
  onContinueChat,
}: {
  rows: LearningCapabilityRow[]
  onContinueChat: () => void
}) {
  const { t } = useTranslation()
  const assessedCount = rows.filter((row) => row.assessmentCount > 0).length
  const multiAngleCount = rows.filter(
    (row) => row.assessmentAngleCount >= 2
  ).length

  return (
    <section className="rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) p-4 shadow-sm backdrop-blur-xl">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <Grid2X2Check aria-hidden="true" className="size-4 text-primary" />
            <h2 className="text-sm font-semibold text-foreground">
              {t("learningProgress.capabilityMap.title")}
            </h2>
          </div>
          <p className="mt-1 text-xs/5 text-muted-foreground">
            {t("learningProgress.capabilityMap.description")}
          </p>
        </div>
        <Button size="sm" onClick={onContinueChat}>
          <MessageSquareText aria-hidden="true" className="size-4" />
          {t("learningProgress.capabilityMap.continueChat")}
        </Button>
      </div>

      <dl className="mt-4 grid grid-cols-2 divide-x divide-(--agent-chat-soft-border) border-y border-(--agent-chat-soft-border) py-3 sm:max-w-md">
        <MapMetric
          label={t("learningProgress.capabilityMap.assessedDomains")}
          value={t("learningProgress.assessedValue", {
            count: assessedCount,
            total: rows.length,
          })}
        />
        <MapMetric
          label={t("learningProgress.capabilityMap.multiAngleDomains")}
          value={t("learningProgress.assessedValue", {
            count: multiAngleCount,
            total: rows.length,
          })}
        />
      </dl>

      <div className="mt-5 grid gap-3 lg:grid-cols-2">
        {rows.map((row) => {
          const angleProgress = Math.min(100, row.assessmentAngleCount * 50)
          return (
            <article
              key={row.knowledgeCode}
              className="rounded-lg border border-(--agent-chat-soft-border) p-4"
            >
              <div className="flex flex-wrap items-center justify-between gap-2">
                <h3 className="text-sm font-semibold text-foreground">
                  {t(`learningProgress.domains.${row.knowledgeCode}`)}
                </h3>
                <Badge variant="outline">
                  {t(`learningProgress.domainStatus.${row.statusCode}`)}
                </Badge>
              </div>
              <div className="mt-4 flex items-center justify-between gap-3 text-xs">
                <span className="font-medium text-foreground">
                  {t("learningProgress.capabilityMap.angleProgress", {
                    count: row.assessmentAngleCount,
                  })}
                </span>
                <span className="text-muted-foreground">
                  {t("learningProgress.capabilityMap.answerCount", {
                    count: row.assessmentCount,
                  })}
                </span>
              </div>
              <div className="mt-2">
                <Progress value={angleProgress} />
              </div>
              <dl className="mt-4 grid grid-cols-4 divide-x divide-(--agent-chat-soft-border)">
                <CompactMetric
                  label={t("learningProgress.capabilityMap.questions")}
                  value={row.distinctQuestionCount}
                />
                <CompactMetric
                  label={t("learningProgress.capabilityMap.correct")}
                  value={row.correctCount}
                />
                <CompactMetric
                  label={t("learningProgress.capabilityMap.partial")}
                  value={row.partialCount}
                />
                <CompactMetric
                  label={t("learningProgress.capabilityMap.incorrect")}
                  value={row.incorrectCount}
                />
              </dl>
              <p className="mt-3 text-[11px]/4 text-muted-foreground">
                {row.latestAssessedAt
                  ? t("learningProgress.capabilityMap.latest", {
                      date: new Date(row.latestAssessedAt).toLocaleString(),
                    })
                  : t("learningProgress.capabilityMap.notAssessed")}
              </p>
            </article>
          )
        })}
      </div>
    </section>
  )
}

function MapMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="px-3 first:pl-0 last:pr-0">
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd className="mt-1 text-lg font-semibold text-foreground">{value}</dd>
    </div>
  )
}

function CompactMetric({ label, value }: { label: string; value: number }) {
  return (
    <div className="px-2 first:pl-0 last:pr-0">
      <dt className="truncate text-[11px] text-muted-foreground">{label}</dt>
      <dd className="mt-1 text-sm font-semibold text-foreground">{value}</dd>
    </div>
  )
}
