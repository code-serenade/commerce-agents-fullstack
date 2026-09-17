import { useTranslation } from "react-i18next"
import { Button, Progress, Switch } from "@workspace/ui-components"
import { DonutChart } from "@workspace/ui-components/stable/chart"
import type {
  LearningEvidenceEntry,
  LearningEvidenceRange,
  LearningProfileResponse,
} from "@/api"
import type { LearningDomainView, LearningProgressViewModel } from "../types"
import { LearningAssessmentTimeline } from "../charts/learning-assessment-timeline"
import { LearningDomainGrid } from "./learning-domain-grid"
import { LearningEvidenceHistory } from "./learning-evidence-history"
import { LearningProgressSummary } from "./learning-progress-summary"

export function LearningOverviewPanel({
  domains,
  evidence,
  evidenceError,
  evidenceLoading,
  evidenceRange,
  model,
  onContinueChat,
  onEvidenceRangeChange,
  onOpenConversation,
  onToggleTracking,
  profile,
  trackingPending,
}: {
  domains: LearningDomainView[]
  evidence?: LearningEvidenceEntry[]
  evidenceError: boolean
  evidenceLoading: boolean
  evidenceRange: LearningEvidenceRange
  model: LearningProgressViewModel
  onContinueChat: () => void
  onEvidenceRangeChange: (range: LearningEvidenceRange) => void
  onOpenConversation: (conversationId: string) => void
  onToggleTracking: (enabled: boolean) => void
  profile?: LearningProfileResponse
  trackingPending: boolean
}) {
  const { t } = useTranslation()
  const observedDomains = domains.filter((domain) => domain.observed)

  return (
    <div className="space-y-4">
      <section className="rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) p-4 shadow-sm backdrop-blur-xl">
        <div className="flex items-center justify-between gap-4">
          <div className="min-w-0">
            <h2 className="text-sm font-semibold text-foreground">
              {t("learningProgress.settingsTitle")}
            </h2>
            <p className="mt-1 text-xs/5 text-muted-foreground">
              {t("learningProgress.settingsDescription")}
            </p>
          </div>
          <div className="flex shrink-0 items-center gap-2">
            <span className="text-xs text-muted-foreground">
              {profile?.enabled
                ? t("learningProgress.trackingOn")
                : t("learningProgress.trackingOff")}
            </span>
            <Switch
              checked={profile?.enabled ?? false}
              disabled={trackingPending}
              onCheckedChange={onToggleTracking}
            />
          </div>
        </div>
      </section>

      <LearningProgressSummary
        assessedCount={model.assessedCount}
        coverageRate={model.coverageRate}
        demonstratedCount={model.demonstratedCount}
        lastAnalyzedAt={profile?.lastAnalyzedAt}
        multiAngleCount={model.multiAngleCount}
        observedCount={model.observedCount}
        retainedCount={model.retainedCount}
        totalDomains={domains.length}
      />
      <LearningAssessmentTimeline timeline={profile?.assessmentTimeline} />
      <section className="rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) p-4 shadow-sm backdrop-blur-xl">
        <div className="flex items-center justify-between gap-4">
          <h2 className="text-sm font-semibold text-foreground">
            {t("learningProgress.coverageSection")}
          </h2>
          <span className="text-lg font-semibold text-foreground">
            {model.coverageRate}%
          </span>
        </div>
        <div className="mt-3 grid items-center gap-4 sm:grid-cols-[auto_1fr]">
          <div className="w-56 shrink-0">
            <DonutChart
              data={[
                {
                  key: "covered",
                  label: t("learningProgress.covered"),
                  value: model.coverageRate,
                  color: "var(--primary)",
                },
                {
                  key: "remaining",
                  label: t("learningProgress.remaining"),
                  value: Math.max(0, 100 - model.coverageRate),
                  color: "var(--agent-chat-soft-border)",
                },
              ]}
              height={200}
            />
          </div>
          <div className="min-w-0">
            <Progress value={model.coverageRate} />
            <p className="mt-2 text-xs/5 text-muted-foreground">
              {t("learningProgress.coverageDescription")}
            </p>
          </div>
        </div>
      </section>
      <section className="rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) p-4 shadow-sm backdrop-blur-xl">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
          <div className="min-w-0">
            <h2 className="text-sm font-semibold text-foreground">
              {t("learningProgress.dynamicsTitle")}
            </h2>
            <p className="mt-1 text-xs/5 text-muted-foreground">
              {t("learningProgress.dynamicsDescription")}
            </p>
            {observedDomains.length > 0 ? (
              <div className="mt-3 flex flex-wrap gap-2">
                {observedDomains.map((domain) => (
                  <span
                    key={domain.code}
                    className="rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-accent-soft) px-2.5 py-1 text-xs text-foreground"
                  >
                    {domain.label}
                  </span>
                ))}
              </div>
            ) : (
              <p className="mt-3 text-sm/5 text-muted-foreground">
                {t("learningProgress.dynamicsEmpty")}
              </p>
            )}
            <p className="mt-3 text-xs/5 text-muted-foreground">
              {t("learningProgress.dynamicsStats", {
                evidence: String(model.evidenceCount),
                messages: String(model.analyzedMessages),
              })}
            </p>
          </div>
          <Button size="sm" onClick={onContinueChat}>
            {t("learningProgress.continueChat")}
          </Button>
        </div>
      </section>
      <LearningDomainGrid
        domains={domains}
        onOpenConversation={onOpenConversation}
      />
      <LearningEvidenceHistory
        evidence={evidence}
        evidenceError={evidenceError}
        evidenceLoading={evidenceLoading}
        evidenceRange={evidenceRange}
        onEvidenceRangeChange={onEvidenceRangeChange}
        onOpenConversation={onOpenConversation}
      />
    </div>
  )
}
