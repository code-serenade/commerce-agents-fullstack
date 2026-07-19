import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import {
  CommercePanel,
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { StatusTone } from "../../components/commerce-ui"
import type { WorkspacesPageState } from "../use-workspaces-page"

export function WorkspaceQualityPanel({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.workspaces.panels.qualityLoop")}
      description={t("commerce.workspaces.panels.qualityLoopDesc")}
    >
      <WorkspaceQualityContent page={page} />
    </CommercePanel>
  )
}

export function WorkspaceQualityContent({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()
  const latestQuality = page.qualityChecks[0]
  const latestTrust = page.trustScores[0]
  const latestLearningRecords = useMemo(
    () => page.learningRecords.slice(0, 4),
    [page.learningRecords]
  )

  return (
    <div>
      <div className="grid gap-3 md:grid-cols-3">
        <QualityMetric
          label={t("commerce.workspaces.quality.latestCheck")}
          badgeLabel={
            latestQuality
              ? t(
                  latestQuality.passed
                    ? "commerce.workspaces.quality.statusPassed"
                    : "commerce.workspaces.quality.statusAttention"
                )
              : t("commerce.workspaces.quality.statusEmpty")
          }
          tone={latestQuality?.passed ? "success" : latestQuality ? "warning" : "muted"}
          value={
            latestQuality
              ? t("commerce.workspaces.quality.score", {
                  score: Math.round(latestQuality.score * 100),
                })
              : t("commerce.common.none")
          }
          detail={
            latestQuality
              ? latestQuality.source || latestQuality.createdAt || "-"
              : t("commerce.workspaces.empty.qualityChecks")
          }
        />
        <QualityMetric
          label={t("commerce.workspaces.quality.trustScore")}
          badgeLabel={latestTrust?.trustLevel || t("commerce.workspaces.quality.statusEmpty")}
          tone={toneForStatus(latestTrust?.trustLevel)}
          value={
            latestTrust
              ? t("commerce.workspaces.quality.score", {
                  score: Math.round(latestTrust.score * 100),
                })
              : t("commerce.common.none")
          }
          detail={latestTrust?.trustLevel || t("commerce.workspaces.empty.trustScores")}
        />
        <QualityMetric
          label={t("commerce.workspaces.quality.learningRecords")}
          badgeLabel={
            page.learningRecords.length
              ? t("commerce.workspaces.quality.statusSaved")
              : t("commerce.workspaces.quality.statusEmpty")
          }
          tone={page.learningRecords.length ? "success" : "muted"}
          value={String(page.learningRecords.length)}
          detail={t("commerce.workspaces.quality.learningRecordsDesc")}
        />
      </div>
      <div className="mt-3 grid gap-3">
        <div className="rounded-lg border border-(--app-border) bg-background p-3">
          <div className="mb-2 text-sm font-medium">
            {t("commerce.workspaces.quality.issues")}
          </div>
          {latestQuality?.issues?.length ? (
            <ul className="space-y-1 text-sm text-muted-foreground">
              {latestQuality.issues.slice(0, 3).map((issue, index) => (
                <li key={`${index}-${String(issue).slice(0, 16)}`}>
                  {String(issue)}
                </li>
              ))}
            </ul>
          ) : (
            <MutedText>{t("commerce.workspaces.empty.qualityIssues")}</MutedText>
          )}
        </div>
        <div className="rounded-lg border border-(--app-border) bg-background p-3">
          <div className="mb-2 text-sm font-medium">
            {t("commerce.workspaces.quality.learningSamples")}
          </div>
          {latestLearningRecords.length ? (
            <div className="space-y-2">
              {latestLearningRecords.map((record) => (
                <div key={record.id} className="min-w-0">
                  <div className="truncate text-sm font-medium">
                    {record.category}
                  </div>
                  <MutedText>{record.content}</MutedText>
                </div>
              ))}
            </div>
          ) : (
            <MutedText>{t("commerce.workspaces.empty.learningRecords")}</MutedText>
          )}
        </div>
      </div>
    </div>
  )
}

function QualityMetric({
  badgeLabel,
  detail,
  label,
  tone,
  value,
}: {
  badgeLabel: string
  detail: string
  label: string
  tone: StatusTone
  value: string
}) {
  return (
    <div className="min-w-0 rounded-lg border border-(--app-border) bg-background p-3">
      <MutedText>{label}</MutedText>
      <div className="mt-2 flex min-w-0 items-center justify-between gap-2">
        <span className="truncate text-lg font-semibold">{value}</span>
        <StatusBadge tone={tone}>{badgeLabel}</StatusBadge>
      </div>
      <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">{detail}</p>
    </div>
  )
}
