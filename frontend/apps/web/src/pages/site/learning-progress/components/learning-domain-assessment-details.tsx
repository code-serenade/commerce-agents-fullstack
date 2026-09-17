import type { TFunction } from "i18next"
import { MessageSquareText } from "lucide-react"
import { Button, Separator } from "@workspace/ui-components"
import type { LearningDomainAssessment } from "@/api"
import { translateLearningCode } from "../domain/learning-code-label"
import { LearningDomainRuleCheck } from "./learning-domain-rule-check"

export function LearningDomainAssessmentDetails({
  assessment,
  observed,
  onOpenConversation,
  t,
}: {
  assessment?: LearningDomainAssessment
  observed: boolean
  onOpenConversation: (conversationId: string) => void
  t: TFunction
}) {
  if (!assessment) {
    return (
      <p className="pb-3 text-sm/6 text-muted-foreground">
        {t(
          observed
            ? "learningProgress.domainLegacyObserved"
            : "learningProgress.domainLegacyUnobserved"
        )}
      </p>
    )
  }

  return (
    <div className="space-y-4 pb-4">
      <div>
        <p className="text-xs font-semibold text-foreground">
          {t("learningProgress.domainWhy")}
        </p>
        <p className="mt-1 text-sm/6 text-muted-foreground">
          {translateLearningCode(
            t,
            "learningProgress.domainReasons",
            assessment.reasonCode
          )}
        </p>
      </div>

      <LearningDomainRuleCheck assessment={assessment} t={t} />

      <div>
        <p className="text-xs font-semibold text-foreground">
          {t("learningProgress.domainEvidenceBasis")}
        </p>
        <dl className="mt-2 grid grid-cols-2 gap-x-4 gap-y-3 sm:grid-cols-3 lg:grid-cols-6">
          <DomainMetric
            label={t("learningProgress.domainMetrics.answers")}
            value={assessment.assessmentCount}
          />
          <DomainMetric
            label={t("learningProgress.domainMetrics.questions")}
            value={assessment.distinctQuestionCount}
          />
          <DomainMetric
            label={t("learningProgress.domainMetrics.angles")}
            value={assessment.assessmentAngleCount}
          />
          <DomainMetric
            label={t("learningProgress.domainMetrics.correct")}
            value={assessment.correctCount}
          />
          <DomainMetric
            label={t("learningProgress.domainMetrics.partial")}
            value={assessment.partialCount}
          />
          <DomainMetric
            label={t("learningProgress.domainMetrics.incorrect")}
            value={assessment.incorrectCount}
          />
        </dl>
        <p className="mt-2 text-xs/5 text-muted-foreground">
          {assessment.latestOutcome && assessment.latestAssessedAt
            ? t("learningProgress.domainLatestAssessment", {
                date: new Date(assessment.latestAssessedAt).toLocaleString(),
                outcome: assessmentOutcomeLabel(t, assessment.latestOutcome),
              })
            : t("learningProgress.domainNoAssessment")}
        </p>
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        <div>
          <p className="text-xs font-semibold text-foreground">
            {t("learningProgress.domainMissing")}
          </p>
          {assessment.missingEvidenceCodes.length ? (
            <ul className="mt-1 space-y-1 text-sm/6 text-muted-foreground">
              {assessment.missingEvidenceCodes.map((code) => (
                <li key={code} className="flex items-start gap-2">
                  <span className="mt-2 size-1 shrink-0 rounded-full bg-primary" />
                  <span>
                    {translateLearningCode(
                      t,
                      "learningProgress.domainMissingEvidence",
                      code
                    )}
                  </span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="mt-1 text-sm/6 text-muted-foreground">
              {t("learningProgress.domainMissingNone")}
            </p>
          )}
        </div>
        <div>
          <p className="text-xs font-semibold text-foreground">
            {t("learningProgress.domainNext")}
          </p>
          <p className="mt-1 text-sm/6 text-muted-foreground">
            {translateLearningCode(
              t,
              "learningProgress.domainNextActions",
              assessment.nextActionCode
            )}
          </p>
        </div>
      </div>

      {assessment.recentEvidence.length ? (
        <div>
          <Separator />
          <p className="mt-3 text-xs font-semibold text-foreground">
            {t("learningProgress.domainRecentEvidence")}
          </p>
          <div className="mt-1 divide-y divide-(--agent-chat-soft-border)">
            {assessment.recentEvidence.map((item) => (
              <div
                key={item.evidenceId}
                className="flex flex-col gap-2 py-2.5 sm:flex-row sm:items-start sm:justify-between"
              >
                <div className="min-w-0">
                  <p className="text-sm/5 text-foreground">
                    {item.questionText ??
                      t("learningProgress.domainHistoricalAssessment")}
                  </p>
                  {item.selectedChoiceText ? (
                    <p className="mt-1 text-xs/5 text-muted-foreground">
                      {t("learningProgress.domainEvidenceChoice", {
                        choice: item.selectedChoiceText,
                      })}
                    </p>
                  ) : null}
                  <p className="mt-1 text-xs/5 text-muted-foreground">
                    {assessmentOutcomeLabel(t, item.outcome)} ·{" "}
                    {new Date(item.createdAt).toLocaleString()}
                  </p>
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-7 shrink-0 self-start px-2 text-xs"
                  onClick={() => onOpenConversation(item.conversationId)}
                >
                  <MessageSquareText className="size-3.5" />
                  {t("learningProgress.viewRelatedConversation")}
                </Button>
              </div>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  )
}

function DomainMetric({ label, value }: { label: string; value: number }) {
  return (
    <div>
      <dt className="text-[11px] text-muted-foreground">{label}</dt>
      <dd className="mt-0.5 text-base font-semibold text-foreground">
        {value}
      </dd>
    </div>
  )
}

function assessmentOutcomeLabel(t: TFunction, outcome: string) {
  return translateLearningCode(
    t,
    "learningProgress.assessmentOutcomes",
    outcome
  )
}
